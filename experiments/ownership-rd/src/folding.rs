//! Experimental Nova IVC over BN254/Grumpkin with Spartan IPA compression.
//!
//! Every step imports the *constrained* arkworks ownership statement, including
//! its signature gadget. This is neither a Groth16 proof nor a production setup.
//! Parameters are reusable for any supported step count, but are scheme-specific.

use std::sync::Arc;

use ark_bn254::Fr;
use ark_ff::{AdditiveGroup, Field, PrimeField};
use ark_r1cs_std::{alloc::AllocVar, eq::EqGadget, fields::fp::FpVar};
use ark_relations::{
    gr1cs::{ConstraintSystem as ArkCS, OptimizationGoal, SynthesisMode, R1CS_PREDICATE_LABEL},
    utils::matrix::Matrix,
};
use ff::PrimeField as NovaPrimeField;
use nova_snark::{
    frontend::{num::AllocatedNum, ConstraintSystem, LinearCombination, SynthesisError, Variable},
    nova::{CompressedSNARK, ProverKey, PublicParams, RecursiveSNARK, VerifierKey},
    provider::{Bn256EngineIPA, GrumpkinEngine},
    traits::{circuit::StepCircuit, snark::RelaxedR1CSSNARKTrait, Engine},
};
use outbe_protocol::{codec::field_to_be_bytes, primitive::hash};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    measured,
    poseidon2::poseidon2_hash3,
    statement::{enforce_slot, fixture, Batch, Scheme, Slot},
    Result,
};

type E1 = Bn256EngineIPA;
type E2 = GrumpkinEngine;
type Scalar = <E1 as Engine>::Scalar;
type S1 = nova_snark::spartan::snark::RelaxedR1CSSNARK<
    E1,
    nova_snark::provider::ipa_pc::EvaluationEngine<E1>,
>;
type S2 = nova_snark::spartan::snark::RelaxedR1CSSNARK<
    E2,
    nova_snark::provider::ipa_pc::EvaluationEngine<E2>,
>;
type PP = PublicParams<E1, E2, OwnershipStep>;
type PK = ProverKey<E1, E2, OwnershipStep, S1, S2>;
type VK = VerifierKey<E1, E2, OwnershipStep, S1, S2>;
type Proof = CompressedSNARK<E1, E2, OwnershipStep, S1, S2>;
type Recursive = RecursiveSNARK<E1, E2, OwnershipStep>;

const FORMAT: u32 = 1;
// Both the input slice and bincode's accounted decoded allocations are bounded.
// These are intentionally finite even on native: parameters are substantial,
// and a mobile browser may still exhaust its practical memory below this ceiling.
const PARAM_LIMIT: usize = 512 * 1024 * 1024;
const PROOF_LIMIT: usize = 16 * 1024 * 1024;

fn decode<T: serde::de::DeserializeOwned, const LIMIT: usize>(bytes: &[u8]) -> Result<T> {
    if bytes.len() > LIMIT {
        return Err("Nova artifact exceeds decoding size limit".into());
    }
    let (value, consumed) = bincode::serde::decode_from_slice(
        bytes,
        bincode::config::standard().with_limit::<LIMIT>(),
    )?;
    if consumed != bytes.len() {
        return Err("trailing bytes in Nova artifact".into());
    }
    Ok(value)
}

fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    Ok(bincode::serde::encode_to_vec(
        value,
        bincode::config::standard(),
    )?)
}

/// No modular reduction is permitted at this bridge. The Nova provider's
/// scalar representation is little-endian and is checked by `from_repr`.
fn to_nova(value: Fr) -> Result<Scalar> {
    let mut repr = <Scalar as NovaPrimeField>::Repr::default();
    let bytes = field_to_be_bytes(&value);
    if repr.as_ref().len() != bytes.len() {
        return Err("unexpected Nova scalar width".into());
    }
    for (dst, src) in repr.as_mut().iter_mut().zip(bytes.iter().rev()) {
        *dst = *src;
    }
    Option::<Scalar>::from(Scalar::from_repr(repr))
        .ok_or_else(|| "ark/Nova scalar modulus mismatch".into())
}

fn initial(binding: Fr) -> [Fr; 3] {
    [
        Fr::ZERO,
        Fr::from_be_bytes_mod_order(b"OUTBE_RD_BATCH_V1"),
        binding,
    ]
}

fn advance(state: [Fr; 3], slot: &Slot) -> Result<[Fr; 3]> {
    let count = state[0] + Fr::ONE;
    let t = hash::poseidon2(&[state[1], slot.1.owner, slot.1.nft_hash])?;
    Ok([count, hash::poseidon2(&[t, state[2], count])?, state[2]])
}

fn expected(batch: &Batch) -> Result<[Fr; 3]> {
    batch
        .slots
        .iter()
        .try_fold(initial(batch.binding_hash), advance)
}

fn nova_state(state: [Fr; 3]) -> Result<Vec<Scalar>> {
    state.into_iter().map(to_nova).collect()
}

/// The adapter stores one step's matrices, never a batch-sized circuit. All
/// host assignments are advice; their validity follows only from matrix rows.
#[derive(Clone)]
struct OwnershipStep {
    data: Arc<StepData>,
}

struct StepData {
    matrices: Vec<Matrix<Scalar>>,
    assignment: Vec<Scalar>,
}

fn ark_step(
    scheme: Scheme,
    slot: &Slot,
    state: [Fr; 3],
) -> Result<ark_relations::gr1cs::ConstraintSystemRef<Fr>> {
    let cs = ArkCS::new_ref();
    cs.set_optimization_goal(OptimizationGoal::Constraints);
    cs.set_mode(SynthesisMode::Prove {
        construct_matrices: true,
        generate_lc_assignments: false,
    });
    let next = advance(state, slot)?;
    // Layout is always [1, in_count,in_digest,in_binding,
    // out_count,out_digest,out_binding, private ownership variables...].
    let input = state
        .into_iter()
        .map(|v| FpVar::new_input(cs.clone(), || Ok(v)))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let output = next
        .into_iter()
        .map(|v| FpVar::new_input(cs.clone(), || Ok(v)))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let owner = FpVar::new_witness(cs.clone(), || Ok(slot.1.owner))?;
    let nft = FpVar::new_witness(cs.clone(), || Ok(slot.1.nft_hash))?;
    enforce_slot(cs.clone(), scheme, &slot.0, &owner, &nft, &input[2])?;
    output[0].enforce_equal(&(input[0].clone() + Fr::ONE))?;
    let t = poseidon2_hash3([input[1].clone(), owner, nft])?;
    poseidon2_hash3([t, input[2].clone(), output[0].clone()])?.enforce_equal(&output[1])?;
    output[2].enforce_equal(&input[2])?;
    cs.finalize();
    Ok(cs)
}

impl OwnershipStep {
    fn new(scheme: Scheme, slot: &Slot, state: [Fr; 3]) -> Result<Self> {
        let cs = ark_step(scheme, slot, state)?;
        let matrices = cs
            .to_matrices()?
            .remove(R1CS_PREDICATE_LABEL)
            .ok_or("missing ownership R1CS matrices")?;
        if matrices.len() != 3 {
            return Err("ownership adapter requires exactly A/B/C matrices".into());
        }
        let inner = cs.borrow().ok_or("missing ownership constraint system")?;
        if inner.num_instance_variables != 7 {
            return Err("unexpected ownership state layout".into());
        }
        let assignment = inner
            .instance_assignment()?
            .iter()
            .chain(inner.witness_assignment()?.iter())
            .copied()
            .map(to_nova)
            .collect::<Result<Vec<_>>>()?;
        let matrices = matrices
            .into_iter()
            .map(|matrix| {
                matrix
                    .into_iter()
                    .map(|row| {
                        row.into_iter()
                            .map(|(c, i)| Ok((to_nova(c)?, i)))
                            .collect::<Result<Vec<_>>>()
                    })
                    .collect::<Result<Vec<_>>>()
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            data: Arc::new(StepData {
                matrices,
                assignment,
            }),
        })
    }
}

fn linear(row: &[(Scalar, usize)], variables: &[Variable]) -> LinearCombination<Scalar> {
    row.iter()
        .fold(LinearCombination::zero(), |lc, (coefficient, index)| {
            lc + (*coefficient, variables[*index])
        })
}

impl StepCircuit<Scalar> for OwnershipStep {
    fn arity(&self) -> usize {
        3
    }

    fn synthesize<CS: ConstraintSystem<Scalar>>(
        &self,
        cs: &mut CS,
        z: &[AllocatedNum<Scalar>],
    ) -> std::result::Result<Vec<AllocatedNum<Scalar>>, SynthesisError> {
        if z.len() != 3 {
            return Err(SynthesisError::Unsatisfiable(
                "ownership state arity must be three".into(),
            ));
        }
        let mut variables = Vec::with_capacity(self.data.assignment.len());
        variables.push(CS::one());
        variables.extend(z.iter().map(AllocatedNum::get_variable));
        let mut output = Vec::with_capacity(3);
        for (i, value) in self.data.assignment.iter().enumerate().skip(4) {
            let allocated =
                AllocatedNum::alloc(cs.namespace(|| format!("ark_variable_{i}")), || Ok(*value))?;
            variables.push(allocated.get_variable());
            if i < 7 {
                output.push(allocated);
            }
        }
        let [a, b, c] = self.data.matrices.as_slice() else {
            return Err(SynthesisError::Unsatisfiable(
                "missing ownership A/B/C matrices".into(),
            ));
        };
        if a.len() != b.len() || a.len() != c.len() {
            return Err(SynthesisError::Unsatisfiable(
                "ownership matrix row counts differ".into(),
            ));
        }
        for i in 0..a.len() {
            cs.enforce(
                || format!("ark_ownership_row_{i}"),
                |_| linear(&a[i], &variables),
                |_| linear(&b[i], &variables),
                |_| linear(&c[i], &variables),
            );
        }
        Ok(output)
    }
}

#[derive(Serialize, Deserialize)]
struct Parameters {
    format: u32,
    scheme: String,
    pp: PP,
    pk: PK,
    vk: VK,
}

pub struct KeyMaterial {
    pub params: Vec<u8>,
    pub report: Value,
}

pub fn prepare(scheme: Scheme) -> Result<KeyMaterial> {
    let mut timings = Vec::new();
    let batch = fixture(scheme, 1, 0)?;
    let step = measured(&mut timings, "step_synthesis", || {
        OwnershipStep::new(scheme, &batch.slots[0], initial(batch.binding_hash))
    })?;
    let pp = measured(&mut timings, "pp_setup", || {
        Ok(PP::setup(&step, &*S1::ck_floor(), &*S2::ck_floor())?)
    })?;
    let (pk, vk) = measured(&mut timings, "compression_setup", || Ok(Proof::setup(&pp)?))?;
    let constraints = pp.num_constraints();
    let variables = pp.num_variables();
    let parameters = Parameters {
        format: FORMAT,
        scheme: scheme.name().into(),
        pp,
        pk,
        vk,
    };
    let params = measured(&mut timings, "params_encode", || encode(&parameters))?;
    if params.len() > PARAM_LIMIT {
        return Err("generated Nova parameters exceed decoding size limit".into());
    }
    // Exercise the exact bounded format used by browser sessions, outside setup.
    measured(&mut timings, "params_decode", || {
        let _: Parameters = decode::<_, PARAM_LIMIT>(&params)?;
        Ok(())
    })?;
    Ok(KeyMaterial {
        report: json!({
            "engine":"nova", "circuit":"flat_aggregation", "scheme":scheme.name(), "timings":timings,
            "sizes": {"params_bytes":params.len(),
                "primary_constraints":constraints.0,"secondary_constraints":constraints.1,
                "primary_variables":variables.0,"secondary_variables":variables.1},
            "parameter_format":"ownership-rd Nova v1; bincode2 serde standard (PP + Spartan IPA PK/VK)",
            "step_size":1, "parameters_independent_of_total_steps":true,
            "warning":"Experimental Nova/Spartan IPA, not Groth16, not production-ready; trusted parameter provenance required"
        }),
        params,
    })
}

pub struct Session {
    scheme: Scheme,
    parameters: Parameters,
    params_bytes: usize,
    load: Value,
}

impl Session {
    pub fn load(scheme: Scheme, params: &[u8]) -> Result<Self> {
        let mut timings = Vec::new();
        let parameters: Parameters = measured(&mut timings, "params_decode", || {
            decode::<_, PARAM_LIMIT>(params)
        })?;
        if parameters.format != FORMAT || parameters.scheme != scheme.name() {
            return Err("Nova parameter format or signature scheme mismatch".into());
        }
        Ok(Self {
            scheme,
            parameters,
            params_bytes: params.len(),
            load: json!({
                "engine":"nova", "circuit":"flat_aggregation", "scheme":scheme.name(), "timings":timings,
                "sizes":{"params_bytes":params.len()},
                "parameter_format":"bincode2 serde standard; Nova 0.76 BN254/Grumpkin PP and Spartan IPA PK/VK"
            }),
        })
    }

    pub fn load_report(&self) -> Value {
        self.load.clone()
    }

    pub fn run(&self, capacity: usize, sample: u64) -> Result<Value> {
        let mut timings = Vec::new();
        let batch = measured(&mut timings, "fixture", || {
            fixture(self.scheme, capacity, sample)
        })?;
        let z0 = nova_state(initial(batch.binding_hash))?;
        let expected_fields = expected(&batch)?;
        let expected = nova_state(expected_fields)?;
        let recursive = measured(&mut timings, "folding", || self.fold(&batch))?;
        measured(&mut timings, "recursive_verify", || {
            let actual = recursive.verify(&self.parameters.pp, capacity, &z0)?;
            if actual != expected {
                return Err("Nova recursive final state mismatch".into());
            }
            Ok(())
        })?;
        let proof = measured(&mut timings, "compress", || {
            Ok(Proof::prove(
                &self.parameters.pp,
                &self.parameters.pk,
                &recursive,
            )?)
        })?;
        let bytes = measured(&mut timings, "proof_encode", || encode(&proof))?;
        let decoded: Proof = measured(&mut timings, "proof_decode", || {
            decode::<_, PROOF_LIMIT>(&bytes)
        })?;
        measured(&mut timings, "verify", || {
            self.verify(&decoded, capacity, &z0, &expected)
        })?;
        let constraints = self.parameters.pp.num_constraints();
        let variables = self.parameters.pp.num_variables();
        Ok(json!({
            "engine":"nova", "circuit":"flat_aggregation", "scheme":self.scheme.name(), "capacity":capacity,
            "sample":sample, "verified":true,
            "statement_digest":batch.statement_digest(), "witness_digest":batch.witness_digest(),
            "timings":timings,
            "sizes":{"params_bytes":self.params_bytes,"proof_bytes":bytes.len(),"public_bytes":96,
                "primary_constraints_per_step":constraints.0,"secondary_constraints_per_step":constraints.1,
                "primary_variables_per_step":variables.0,"secondary_variables_per_step":variables.1,
                "ownership_steps":capacity},
            "state_commitment":{"count":capacity,
                "digest_be32":field_to_be_bytes(&expected_fields[1]),
                "binding_be32":field_to_be_bytes(&batch.binding_hash)},
            "proof_format":"bincode2 serde standard; Nova compressed SNARK with Spartan IPA on both curves",
            "public_encoding":"public_bytes=96 is the semantic final [count,digest,binding], already included in proof_bytes; do not add it as wire overhead. Verification additionally takes initial state and step count",
            "folding_includes":"ark ownership synthesis, matrix materialization/bridge, Nova recursive proving",
            "verification_context":"verification after folding/compression; resident decoded PP and Spartan keys",
            "warning":"Unaudited R&D; not Groth16 or EVM calldata; no nullifier/spent-state tracking; native and WASM share pure-Rust code"
        }))
    }

    fn fold(&self, batch: &Batch) -> Result<Recursive> {
        if batch.scheme != self.scheme
            || batch.slots.is_empty()
            || batch
                .slots
                .iter()
                .any(|slot| slot.1.binding_hash != batch.binding_hash)
        {
            return Err("invalid Nova batch scheme, binding or step count".into());
        }
        let mut state = initial(batch.binding_hash);
        let first = OwnershipStep::new(self.scheme, &batch.slots[0], state)?;
        let z0 = nova_state(state)?;
        let mut recursive = Recursive::new(&self.parameters.pp, &first, &z0)?;
        recursive.prove_step(&self.parameters.pp, &first)?;
        state = advance(state, &batch.slots[0])?;
        // Only one step's ark matrices/advice remain live at a time.
        drop(first);
        for slot in batch.slots.iter().skip(1) {
            let step = OwnershipStep::new(self.scheme, slot, state)?;
            recursive.prove_step(&self.parameters.pp, &step)?;
            state = advance(state, slot)?;
        }
        Ok(recursive)
    }

    fn verify(
        &self,
        proof: &Proof,
        count: usize,
        z0: &[Scalar],
        expected: &[Scalar],
    ) -> Result<()> {
        let actual = proof.verify(&self.parameters.vk, count, z0)?;
        if actual != expected {
            return Err("Nova compressed proof final state mismatch".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ownership_matrix_topology_is_state_and_witness_independent() -> Result<()> {
        for scheme in [Scheme::Canonical, Scheme::Poseidon2] {
            let a = fixture(scheme, 2, 21)?;
            let b = fixture(scheme, 2, 22)?;
            let state = advance(initial(b.binding_hash), &b.slots[0])?;
            let ca = ark_step(scheme, &a.slots[0], initial(a.binding_hash))?;
            let cb = ark_step(scheme, &b.slots[1], state)?;
            assert!(ca.is_satisfied()?);
            assert!(cb.is_satisfied()?);
            assert_eq!(ca.to_matrices()?, cb.to_matrices()?);
            assert_eq!(to_nova(-Fr::ONE)?.to_repr().as_ref().len(), 32);
            assert_eq!(to_nova(-Fr::ONE)? + to_nova(Fr::ONE)?, to_nova(Fr::ZERO)?);
            let mut invalid = a.slots[0].clone();
            invalid.0.signature[0] ^= 1;
            if let Ok(cs) = ark_step(scheme, &invalid, initial(a.binding_hash)) {
                assert!(!cs.is_satisfied()?);
            }
        }
        Ok(())
    }

    #[test]
    fn real_compressed_ownership_one_and_two_steps_and_rejections() -> Result<()> {
        for scheme in [Scheme::Canonical, Scheme::Poseidon2] {
            let keys = prepare(scheme)?;
            let session = Session::load(scheme, &keys.params)?;
            let mut trailing = keys.params.clone();
            trailing.push(0);
            assert!(Session::load(scheme, &trailing).is_err());
            assert!(Session::load(
                if scheme == Scheme::Canonical {
                    Scheme::Poseidon2
                } else {
                    Scheme::Canonical
                },
                &keys.params
            )
            .is_err());
            for count in [1, 2] {
                let batch = fixture(scheme, count, 41)?;
                let z0 = nova_state(initial(batch.binding_hash))?;
                let expected = nova_state(expected(&batch)?)?;
                let recursive = session.fold(&batch)?;
                assert_eq!(
                    recursive.verify(&session.parameters.pp, count, &z0)?,
                    expected
                );
                let proof =
                    Proof::prove(&session.parameters.pp, &session.parameters.pk, &recursive)?;
                let bytes = encode(&proof)?;
                let proof: Proof = decode::<_, PROOF_LIMIT>(&bytes)?;
                session.verify(&proof, count, &z0, &expected)?;
                assert!(session.verify(&proof, count + 1, &z0, &expected).is_err());
                let mut wrong_initial = z0.clone();
                wrong_initial[2] += to_nova(Fr::ONE)?;
                assert!(session
                    .verify(&proof, count, &wrong_initial, &expected)
                    .is_err());
                let mut wrong_final = expected.clone();
                wrong_final[1] += to_nova(Fr::ONE)?;
                assert!(session.verify(&proof, count, &z0, &wrong_final).is_err());
                wrong_final = expected.clone();
                wrong_final[0] += to_nova(Fr::ONE)?;
                assert!(session.verify(&proof, count, &z0, &wrong_final).is_err());
                if count == 2 {
                    let mut reversed = batch.clone();
                    reversed.slots.reverse();
                    assert!(session
                        .verify(
                            &proof,
                            count,
                            &z0,
                            &nova_state(super::expected(&reversed)?)?
                        )
                        .is_err());
                }
                assert!(decode::<Proof, PROOF_LIMIT>(&bytes[..bytes.len() / 2]).is_err());
                let mut trailing = bytes.clone();
                trailing.push(0);
                assert!(decode::<Proof, PROOF_LIMIT>(&trailing).is_err());
                let mut corrupt = bytes;
                let last = corrupt.len() - 1;
                corrupt[last] ^= 0x80;
                match decode::<Proof, PROOF_LIMIT>(&corrupt) {
                    Err(_) => {}
                    Ok(p) => assert!(session.verify(&p, count, &z0, &expected).is_err()),
                }
                let mut invalid = batch.clone();
                invalid.slots[count - 1].0.signature[0] ^= 1;
                match session.fold(&invalid) {
                    Err(_) => {}
                    Ok(p) => assert!(p.verify(&session.parameters.pp, count, &z0).is_err()),
                }
            }
        }
        Ok(())
    }
}
