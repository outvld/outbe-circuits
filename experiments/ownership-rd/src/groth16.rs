//! Experimental, single-party Groth16 setup and resident proving keys.
//!
//! Key reads/fetches and process/worker lifetime are deliberately outside this engine.
//! Repeated `Session::run` calls retain the decoded PK and prepared VK, but generate
//! a new fixture, witness and OS-entropy-blinded proof each time. No warmup is discarded.

use ark_bn254::{Bn254, Fr};
use ark_ff::UniformRand;
use ark_groth16::{
    prepare_verifying_key, Groth16, PreparedVerifyingKey, Proof, ProvingKey, VerifyingKey,
};
use ark_relations::gr1cs::{
    ConstraintSynthesizer, ConstraintSystem, OptimizationGoal, SynthesisMode, R1CS_PREDICATE_LABEL,
};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::rand::{rngs::OsRng, rngs::StdRng, SeedableRng};
use serde_json::{json, Value};

use crate::statement::{circuit_fixture, CircuitKind, Scheme};
use crate::{measured, Result};

pub struct KeyMaterial {
    pub pk: Vec<u8>,
    pub vk: Vec<u8>,
    pub report: Value,
}

/// Local single-party random parameters, NOT a production trusted setup ceremony.
/// Serialization is timed separately and no key I/O occurs here.
pub fn prepare(scheme: Scheme, capacity: usize) -> Result<KeyMaterial> {
    prepare_circuit(CircuitKind::FlatAggregation, scheme, capacity)
}

pub fn prepare_demo(scheme: Scheme) -> Result<KeyMaterial> {
    prepare_circuit(CircuitKind::DemoTribute, scheme, 1)
}

fn prepare_circuit(kind: CircuitKind, scheme: Scheme, capacity: usize) -> Result<KeyMaterial> {
    let mut timings = Vec::new();
    let batch = measured(&mut timings, "fixture", || {
        circuit_fixture(kind, scheme, capacity, 0)
    })?;
    let mut rng = StdRng::from_rng(OsRng)?;
    let pk = measured(&mut timings, "setup", || {
        Ok(Groth16::<Bn254>::generate_random_parameters_with_reduction(
            batch, &mut rng,
        )?)
    })?;
    let pk_bytes = measured(&mut timings, "pk_encode", || encode(&pk))?;
    let vk_bytes = measured(&mut timings, "vk_encode", || encode(&pk.vk))?;
    let report = json!({
        "engine": "groth16",
        "circuit": kind.name(),
        "scheme": scheme.name(),
        "capacity": capacity,
        "single_party_test_setup": true,
        "timings": timings,
        "sizes": {
            "pk_bytes": pk_bytes.len(),
            "vk_bytes": vk_bytes.len(),
            "instance_variables": pk.vk.gamma_abc_g1.len(),
            "witness_variables": pk.l_query.len()
        }
    });
    Ok(KeyMaterial {
        pk: pk_bytes,
        vk: vk_bytes,
        report,
    })
}

/// Holds one decoded proving key and one prepared verifying key across runs.
pub struct Session {
    kind: CircuitKind,
    scheme: Scheme,
    capacity: usize,
    pk: ProvingKey<Bn254>,
    pvk: PreparedVerifyingKey<Bn254>,
    pk_bytes: usize,
    vk_bytes: usize,
    load_timings: Vec<Value>,
}

impl Session {
    /// Checked compressed decoding includes curve/subgroup validation. Both inputs
    /// must contain exactly one key, and the supplied VK must match the PK's VK.
    pub fn load(scheme: Scheme, capacity: usize, pk: &[u8], vk: &[u8]) -> Result<Self> {
        Self::load_circuit(CircuitKind::FlatAggregation, scheme, capacity, pk, vk)
    }

    pub fn load_demo(scheme: Scheme, pk: &[u8], vk: &[u8]) -> Result<Self> {
        Self::load_circuit(CircuitKind::DemoTribute, scheme, 1, pk, vk)
    }

    fn load_circuit(
        kind: CircuitKind,
        scheme: Scheme,
        capacity: usize,
        pk: &[u8],
        vk: &[u8],
    ) -> Result<Self> {
        if !matches!(capacity, 1 | 2 | 4 | 8 | 16 | 32 | 64) {
            return Err(format!("unsupported ownership capacity: {capacity}").into());
        }
        let mut timings = Vec::new();
        let decoded_pk: ProvingKey<Bn254> =
            measured(&mut timings, "pk_decode", || decode_exact(pk))?;
        let decoded_vk: VerifyingKey<Bn254> =
            measured(&mut timings, "vk_decode", || decode_exact(vk))?;
        let instances = match kind {
            CircuitKind::FlatAggregation => 2 * capacity + 2,
            CircuitKind::DemoTribute => 5,
        }; // Includes the constant-one slot.
        if decoded_vk.gamma_abc_g1.len() != instances
            || decoded_pk.vk.gamma_abc_g1.len() != instances
        {
            return Err("Groth16 key public-input arity does not match capacity".into());
        }
        if decoded_pk.vk != decoded_vk {
            return Err("Groth16 proving and verifying keys do not match".into());
        }
        let variables = instances
            .checked_add(decoded_pk.l_query.len())
            .ok_or("Groth16 key variable count overflow")?;
        if decoded_pk.a_query.len() != variables
            || decoded_pk.b_g1_query.len() != variables
            || decoded_pk.b_g2_query.len() != variables
            || decoded_pk.h_query.is_empty()
        {
            return Err("inconsistent Groth16 proving-key query lengths".into());
        }
        let pvk = measured(&mut timings, "vk_prepare", || {
            Ok(prepare_verifying_key(&decoded_vk))
        })?;
        Ok(Self {
            kind,
            scheme,
            capacity,
            pk: decoded_pk,
            pvk,
            pk_bytes: pk.len(),
            vk_bytes: vk.len(),
            load_timings: timings,
        })
    }

    pub fn load_report(&self) -> Value {
        json!({
            "engine": "groth16",
            "circuit": self.kind.name(),
            "scheme": self.scheme.name(),
            "capacity": self.capacity,
            "timings": self.load_timings,
            "sizes": { "pk_bytes": self.pk_bytes, "vk_bytes": self.vk_bytes }
        })
    }

    pub fn run(&self, sample: u64) -> Result<Value> {
        Ok(self.execute(sample)?.report)
    }

    fn execute(&self, sample: u64) -> Result<CompletedProof> {
        let mut timings = Vec::new();
        let batch = measured(&mut timings, "fixture", || {
            circuit_fixture(self.kind, self.scheme, self.capacity, sample)
        })?;
        // Digests and public extraction are reporting work, not witness synthesis.
        let statement_digest = batch.statement_digest();
        let witness_digest = batch.witness_digest();
        let public = batch.public_inputs();
        let (matrices, assignment, instances, constraints, witnesses) =
            measured(&mut timings, "witness", || {
                let cs = ConstraintSystem::new_ref();
                cs.set_optimization_goal(OptimizationGoal::Constraints);
                cs.set_mode(SynthesisMode::Prove {
                    construct_matrices: true,
                    generate_lc_assignments: false,
                });
                batch.generate_constraints(cs.clone())?;
                cs.finalize();
                let matrices = cs
                    .to_matrices()?
                    .remove(R1CS_PREDICATE_LABEL)
                    .ok_or("missing ownership R1CS matrices")?;
                let inner = cs.borrow().ok_or("missing ownership constraint system")?;
                let constraints = inner
                    .get_predicate_num_constraints(R1CS_PREDICATE_LABEL)
                    .ok_or("missing ownership R1CS constraints")?;
                let mut assignment =
                    Vec::with_capacity(inner.num_instance_variables + inner.num_witness_variables);
                assignment.extend_from_slice(inner.instance_assignment()?);
                assignment.extend_from_slice(inner.witness_assignment()?);
                Ok((
                    matrices,
                    assignment,
                    inner.num_instance_variables,
                    constraints,
                    inner.num_witness_variables,
                ))
            })?;
        if instances != public.len() + 1
            || assignment.first() != Some(&Fr::from(1u64))
            || assignment.get(1..instances) != Some(public.as_slice())
        {
            return Err("Groth16 synthesized public assignment mismatch".into());
        }
        let domain = constraints
            .checked_add(instances)
            .and_then(usize::checked_next_power_of_two)
            .ok_or("Groth16 evaluation domain overflow")?;
        if self.pk.a_query.len() != assignment.len()
            || self.pk.l_query.len() != witnesses
            || self.pk.vk.gamma_abc_g1.len() != instances
            || self.pk.h_query.len() != domain - 1
        {
            return Err("Groth16 proving key does not match synthesized circuit dimensions".into());
        }
        // Seed independently on every run; fixture seeds never supply proof blinding.
        let mut rng = StdRng::from_rng(OsRng)?;
        let proof = measured(&mut timings, "prove", || {
            Ok(Groth16::<Bn254>::create_proof_with_reduction_and_matrices(
                &self.pk,
                Fr::rand(&mut rng),
                Fr::rand(&mut rng),
                &matrices,
                instances,
                constraints,
                &assignment,
            )?)
        })?;
        let assignment_fields = assignment.len();
        drop(matrices);
        drop(assignment);
        let encoded = measured(&mut timings, "proof_encode", || encode(&proof))?;
        let decoded: Proof<Bn254> =
            measured(&mut timings, "proof_decode", || decode_exact(&encoded))?;
        let verified = measured(&mut timings, "verify", || {
            verify(&self.pvk, &decoded, &public)
        })?;
        if !verified {
            return Err(format!(
                "Groth16 verification failed: {}/n{}/sample{sample}",
                self.scheme.name(),
                self.capacity,
            )
            .into());
        }
        let report = json!({
            "engine": "groth16",
            "circuit": self.kind.name(),
            "scheme": self.scheme.name(),
            "capacity": self.capacity,
            "sample": sample,
            "verified": true,
            "statement_digest": statement_digest,
            "witness_digest": witness_digest,
            "timings": timings,
            "sizes": {
                "pk_bytes": self.pk_bytes,
                "vk_bytes": self.vk_bytes,
                "proof_bytes": encoded.len(),
                "public_bytes": public.len() * 32,
                "constraints": constraints,
                "instance_variables": instances,
                "witness_variables": witnesses,
                "assignment_fields": assignment_fields,
                "assignment_bytes": assignment_fields * 32
            }
        });
        Ok(CompletedProof {
            report,
            #[cfg(test)]
            encoded,
            #[cfg(test)]
            public,
        })
    }
}

// Retain proof bytes/public words for behavioral tests without a second proving path.
struct CompletedProof {
    report: Value,
    #[cfg(test)]
    encoded: Vec<u8>,
    #[cfg(test)]
    public: Vec<Fr>,
}

fn encode<T: CanonicalSerialize>(value: &T) -> Result<Vec<u8>> {
    let mut bytes = Vec::with_capacity(value.compressed_size());
    value.serialize_compressed(&mut bytes)?;
    Ok(bytes)
}

fn decode_exact<T: CanonicalDeserialize>(mut bytes: &[u8]) -> Result<T> {
    let value = T::deserialize_compressed(&mut bytes)?;
    if !bytes.is_empty() {
        return Err("trailing bytes after compressed Groth16 encoding".into());
    }
    Ok(value)
}

fn verify(pvk: &PreparedVerifyingKey<Bn254>, proof: &Proof<Bn254>, public: &[Fr]) -> Result<bool> {
    // ark-groth16 0.6 prepare_inputs uses zip and does not check arity itself.
    if public.len().checked_add(1) != Some(pvk.vk.gamma_abc_g1.len()) {
        return Err("Groth16 verification public-input arity mismatch".into());
    }
    Ok(Groth16::<Bn254>::verify_proof(pvk, proof, public)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proof_behavior(kind: CircuitKind, scheme: Scheme) -> Result<()> {
        let keys = match kind {
            CircuitKind::FlatAggregation => prepare(scheme, 1)?,
            CircuitKind::DemoTribute => prepare_demo(scheme)?,
        };
        let load = |pk: &[u8], vk: &[u8]| match kind {
            CircuitKind::FlatAggregation => Session::load(scheme, 1, pk, vk),
            CircuitKind::DemoTribute => Session::load_demo(scheme, pk, vk),
        };
        let session = load(&keys.pk, &keys.vk)?;
        assert_eq!(keys.report["circuit"], kind.name());
        assert_eq!(session.load_report()["circuit"], kind.name());
        let first = session.execute(1)?;
        assert_eq!(first.report["circuit"], kind.name());
        if kind == CircuitKind::DemoTribute {
            assert_eq!(first.public.len(), 4);
            assert_eq!(first.report["scheme"], scheme.name());
            assert_eq!(first.report["capacity"], 1);
            assert_eq!(first.report["sizes"]["instance_variables"], 5);
        }
        let decoded: Proof<Bn254> = decode_exact(&first.encoded)?;
        assert!(verify(&session.pvk, &decoded, &first.public)?);
        for index in 0..first.public.len() {
            let mut altered = first.public.clone();
            altered[index] += Fr::from(1u64);
            assert!(!verify(&session.pvk, &decoded, &altered)?);
        }
        assert!(verify(
            &session.pvk,
            &decoded,
            &first.public[..first.public.len() - 1]
        )
        .is_err());
        let mut extra_public = first.public.clone();
        extra_public.push(Fr::from(0u64));
        assert!(verify(&session.pvk, &decoded, &extra_public).is_err());
        assert!(decode_exact::<Proof<Bn254>>(&first.encoded[..first.encoded.len() - 1]).is_err());
        let mut trailing_proof = first.encoded.clone();
        trailing_proof.push(0);
        assert!(decode_exact::<Proof<Bn254>>(&trailing_proof).is_err());
        let mut corrupted = first.encoded.clone();
        corrupted[0] ^= 1;
        if let Ok(proof) = decode_exact::<Proof<Bn254>>(&corrupted) {
            assert!(!verify(&session.pvk, &proof, &first.public)?);
        }
        let wrong = prepare_circuit(kind, scheme, 1)?;
        let wrong_vk: VerifyingKey<Bn254> = decode_exact(&wrong.vk)?;
        assert!(!verify(
            &prepare_verifying_key(&wrong_vk),
            &decoded,
            &first.public
        )?);
        assert!(load(&keys.pk, &wrong.vk).is_err());
        assert!(Session::load(scheme, 2, &keys.pk, &keys.vk).is_err());
        assert!(load(&keys.pk[..keys.pk.len() - 1], &keys.vk).is_err());
        let mut trailing_pk = keys.pk.clone();
        trailing_pk.push(0);
        assert!(load(&trailing_pk, &keys.vk).is_err());
        let mut trailing_vk = keys.vk.clone();
        trailing_vk.push(0);
        assert!(load(&keys.pk, &trailing_vk).is_err());
        let mut truncated_query: ProvingKey<Bn254> = decode_exact(&keys.pk)?;
        truncated_query.a_query.pop();
        assert!(load(&encode(&truncated_query)?, &keys.vk).is_err());
        if kind == CircuitKind::DemoTribute {
            assert!(Session::load(scheme, 1, &keys.pk, &keys.vk).is_err());
        } else {
            assert!(Session::load_demo(scheme, &keys.pk, &keys.vk).is_err());
        }
        // A second independent fixture/proof must work with the very same held PK.
        let second = session.execute(2)?;
        let second_proof: Proof<Bn254> = decode_exact(&second.encoded)?;
        assert!(verify(&session.pvk, &second_proof, &second.public)?);
        assert_ne!(first.public, second.public);
        assert_ne!(first.encoded, second.encoded);
        assert!(!verify(&session.pvk, &decoded, &second.public)?);
        Ok(())
    }

    #[test]
    fn canonical_n1_real_proofs_reject_changed_statements_and_keys() -> Result<()> {
        proof_behavior(CircuitKind::FlatAggregation, Scheme::Canonical)
    }

    #[test]
    fn poseidon2_n1_real_proofs_reject_changed_statements_and_keys() -> Result<()> {
        proof_behavior(CircuitKind::FlatAggregation, Scheme::Poseidon2)
    }

    #[test]
    fn demo_real_proofs_reject_changed_statements_and_keys() -> Result<()> {
        proof_behavior(CircuitKind::DemoTribute, Scheme::Canonical)
    }

    #[test]
    fn poseidon2_demo_real_proofs_reject_changed_statements_and_keys() -> Result<()> {
        proof_behavior(CircuitKind::DemoTribute, Scheme::Poseidon2)
    }
}
