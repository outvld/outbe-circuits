//! Experimental R1CS port of the frozen Tribute/flat-ownership statements.

use ark_bn254::Fr;
use ark_crypto_primitives::prf::blake2s::constraints::evaluate_blake2s;
use ark_ec::PrimeGroup;
use ark_ff::{Field, MontFp, PrimeField};
use ark_grumpkin::constraints::GVar;
use ark_r1cs_std::{fields::fp::FpVar, prelude::*};
use ark_relations::gr1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use outbe_zk_canonical::noir;

use crate::poseidon2::poseidon2_hash3;

#[derive(Clone)]
pub struct DemoTributeCircuit {
    pub witness: noir::demo_tribute::Witness,
    pub public: noir::demo_tribute::PublicInputs,
}

#[derive(Clone)]
pub struct FlatAggregationCircuit {
    pub slots: Vec<(
        noir::ownership_proof::Witness,
        noir::ownership_proof::PublicInputs,
    )>,
    pub binding_hash: Fr,
}

impl ConstraintSynthesizer<Fr> for DemoTributeCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        // Public allocation order is the original Noir ABI, not traversal order.
        let owner = FpVar::new_input(cs.clone(), || Ok(self.public.derived_owner))?;
        let nft_hash = FpVar::new_input(cs.clone(), || Ok(self.public.nft_hash))?;
        let binding = FpVar::new_input(cs.clone(), || Ok(self.public.binding_hash))?;
        let root = FpVar::new_input(cs.clone(), || Ok(self.public.merkle_root))?;
        let ownership = noir::ownership_proof::Witness {
            pk: self.witness.pk.clone(),
            signature: self.witness.signature,
            nonce: self.witness.nonce,
        };
        enforce_ownership(cs.clone(), &ownership, &owner, &nft_hash, &binding)?;

        enforce_demo_inclusion(cs, &self.witness, nft_hash, &root)
    }
}

/// The frozen Demo's full depth-32 inclusion, shared by signature experiments.
pub fn enforce_demo_inclusion(
    cs: ConstraintSystemRef<Fr>,
    witness: &noir::demo_tribute::Witness,
    nft_hash: FpVar<Fr>,
    root: &FpVar<Fr>,
) -> Result<(), SynthesisError> {
    let domain = FpVar::Constant(outbe_zk_canonical::demo_tribute::demo_tribute_domain());
    let mut node = nft_hash;
    for (sibling, direction) in witness
        .merkle_path_siblings
        .iter()
        .zip(witness.merkle_path_indices.iter())
    {
        let sibling = FpVar::new_witness(cs.clone(), || Ok(*sibling))?;
        let direction = Boolean::new_witness(cs.clone(), || Ok(*direction))?;
        // Noir's true direction means the current node is on the left.
        let left = direction.select(&node, &sibling)?;
        let right = direction.select(&sibling, &node)?;
        node = poseidon2_hash3([domain.clone(), left, right])?;
    }
    node.enforce_equal(root)
}

impl ConstraintSynthesizer<Fr> for FlatAggregationCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        if !matches!(self.slots.len(), 1 | 2 | 4 | 8 | 16 | 32 | 64)
            || self
                .slots
                .iter()
                .any(|(_, public)| public.binding_hash != self.binding_hash)
        {
            return Err(SynthesisError::Unsatisfiable);
        }
        let mut public_vars = Vec::with_capacity(self.slots.len());
        for (_, public) in &self.slots {
            let owner = FpVar::new_input(cs.clone(), || Ok(public.owner))?;
            let nft_hash = FpVar::new_input(cs.clone(), || Ok(public.nft_hash))?;
            public_vars.push((owner, nft_hash));
        }
        let binding = FpVar::new_input(cs.clone(), || Ok(self.binding_hash))?;
        for ((witness, _), (owner, nft_hash)) in self.slots.iter().zip(&public_vars) {
            enforce_ownership(cs.clone(), witness, owner, nft_hash, &binding)?;
        }
        Ok(())
    }
}

/// Canonical, zero-padded big-endian field encoding. `to_bytes_le` constrains
/// the unique decomposition (< Fr::MODULUS), rather than just its field residue.
pub fn field_to_be32(value: &FpVar<Fr>) -> Result<[UInt8<Fr>; 32], SynthesisError> {
    let mut bytes = value.to_bytes_le()?;
    bytes.reverse();
    bytes.try_into().map_err(|_| SynthesisError::Unsatisfiable)
}

pub fn enforce_ownership(
    cs: ConstraintSystemRef<Fr>,
    witness: &noir::ownership_proof::Witness,
    owner: &FpVar<Fr>,
    nft_hash: &FpVar<Fr>,
    binding: &FpVar<Fr>,
) -> Result<(), SynthesisError> {
    let x = FpVar::new_witness(cs.clone(), || Ok(witness.pk.x))?;
    let y = FpVar::new_witness(cs.clone(), || Ok(witness.pk.y))?;
    let nonce = FpVar::new_witness(cs.clone(), || Ok(witness.nonce))?;
    y.square()?
        .enforce_equal(&(x.square()? * &x - Fr::from(17u64)))?;
    (x.is_zero()? & y.is_zero()?).enforce_equal(&Boolean::FALSE)?;
    // Grumpkin has cofactor one. Coordinates are already constrained on-curve;
    // constructing GVar directly avoids native affine validation/panics on bad pk.
    let pk = GVar::new(x.clone(), y.clone(), FpVar::one());
    poseidon2_hash3([x, y, nonce.clone()])?.enforce_equal(owner)?;
    let message = field_to_be32(&poseidon2_hash3([
        nft_hash.clone(),
        nonce,
        binding.clone(),
    ])?)?;
    let signature = UInt8::new_witness_vec(cs, &witness.signature)?
        .try_into()
        .map_err(|_| SynthesisError::Unsatisfiable)?;
    enforce_schnorr(&pk, &signature, &message)
}

fn bytes_be_to_bits_le(bytes: &[UInt8<Fr>]) -> Result<Vec<Boolean<Fr>>, SynthesisError> {
    let mut bits = Vec::with_capacity(bytes.len() * 8);
    for byte in bytes.iter().rev() {
        bits.extend(byte.to_bits_le()?);
    }
    Ok(bits)
}

/// Signature bytes are raw big-endian s || e. In particular, e is not first
/// reduced into a field variable: all 256 bits participate in multiplication
/// and the final digest comparison.
pub fn enforce_schnorr(
    pk: &GVar,
    signature: &[UInt8<Fr>; 64],
    message: &[UInt8<Fr>],
) -> Result<(), SynthesisError> {
    let s = bytes_be_to_bits_le(&signature[..32])?;
    let e = bytes_be_to_bits_le(&signature[32..])?;
    Boolean::kary_or(&s)?.enforce_equal(&Boolean::TRUE)?;
    Boolean::kary_or(&e)?.enforce_equal(&Boolean::TRUE)?;
    let q_minus_one = (-ark_grumpkin::Fr::ONE).into_bigint();
    Boolean::enforce_smaller_or_equal_than_le(&s, q_minus_one)?;

    let generator = GVar::constant(ark_grumpkin::Projective::generator());
    let r = generator.scalar_mul_le(s.iter())? + pk.scalar_mul_le(e.iter())?;
    r.is_zero()?.enforce_equal(&Boolean::FALSE)?;
    let r = r.to_affine()?;
    let pk = pk.to_affine()?;
    let challenge_x = pedersen_hash3([r.x, pk.x, pk.y])?;
    let challenge_bytes = field_to_be32(&challenge_x)?;
    let mut input_bits = Vec::with_capacity((32 + message.len()) * 8);
    for byte in challenge_bytes.iter().chain(message) {
        // Blake2s packs little-endian bits into each input byte/word.
        input_bits.extend(byte.to_bits_le()?);
    }
    let digest = evaluate_blake2s(&input_bits)?;
    for (word, expected) in digest.iter().zip(signature[32..].chunks_exact(4)) {
        let bytes = word.to_bytes_le()?;
        for (actual, expected) in bytes.iter().zip(expected) {
            actual.enforce_equal(expected)?;
        }
    }
    Ok(())
}

// Exact generators from outbe-protocol/src/primitive/signature.rs. Keep them
// local: the experiment must not widen the production primitive's visibility.
const PEDERSEN_POINTS: [(Fr, Fr); 4] = [
    (
        MontFp!("3728882899078719075161482178784387565366481897740339799480980287259621149274"),
        MontFp!("11985179162806396554955778097047359550576116765952231747549784474790041064285"),
    ),
    (
        MontFp!("2393473289045184898987089634332637236754766663897650125720167164137088869378"),
        MontFp!("14752839959415467457196082350231122454649853219840744672802853620609001898278"),
    ),
    (
        MontFp!("12786580035164724895989409532878568950405964521921739189071475352889503270728"),
        MontFp!("1658946642478826263901298755938807527017017780224674388532518006781615929512"),
    ),
    (
        MontFp!("20793534830995666052889629834383221590246524226953098604058514977776739732941"),
        MontFp!("21169538964657307934625131027308026551296100558246394809295742894776804020355"),
    ),
];

pub fn pedersen_hash3(inputs: [FpVar<Fr>; 3]) -> Result<FpVar<Fr>, SynthesisError> {
    let points = PEDERSEN_POINTS
        .map(|(x, y)| GVar::constant(ark_grumpkin::Affine::new_unchecked(x, y).into()));
    let length_bits = [Boolean::TRUE, Boolean::TRUE];
    let mut accumulator = points[3].scalar_mul_le(length_bits.iter())?;
    for (point, input) in points[..3].iter().zip(&inputs) {
        accumulator += point.scalar_mul_le(input.to_bits_le()?.iter())?;
    }
    let affine = accumulator.to_affine()?;
    // Noir's identity encoding is x=0; identity is not an extra rejection here.
    affine.infinity.select(&FpVar::zero(), &affine.x)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_ff::{AdditiveGroup, BigInteger};
    use ark_relations::gr1cs::ConstraintSystem;
    use ark_std::rand::{rngs::StdRng, SeedableRng};
    use ark_std::UniformRand;
    use outbe_protocol::primitive::{hash, signature};

    const KAT_SIGNATURE: [u8; 64] = [
        1, 13, 119, 112, 212, 39, 233, 41, 84, 235, 255, 93, 245, 172, 186, 83, 157, 253, 76, 77,
        33, 128, 178, 15, 214, 67, 105, 107, 177, 234, 77, 48, 27, 237, 155, 84, 39, 84, 247, 27,
        22, 8, 176, 230, 24, 115, 145, 220, 254, 122, 135, 179, 171, 4, 214, 202, 64, 199, 19, 84,
        239, 138, 124, 12,
    ];

    fn kat_pk() -> ark_grumpkin::Affine {
        ark_grumpkin::Affine::new_unchecked(
            MontFp!("2124416763957513755957069320378814719427254224313784354193701269410464905578"),
            MontFp!(
                "21635190314466406102464795369176917324283837527799356152433238205601767715223"
            ),
        )
    }

    fn schnorr_satisfied(pk: ark_grumpkin::Affine, sig: [u8; 64], msg: &[u8]) -> bool {
        let cs = ConstraintSystem::<Fr>::new_ref();
        // No native curve constructor is invoked on attacker-supplied coordinates.
        let x = FpVar::new_witness(cs.clone(), || Ok(pk.x)).unwrap();
        let y = FpVar::new_witness(cs.clone(), || Ok(pk.y)).unwrap();
        let pk = GVar::new(x, y, FpVar::one());
        let signature: [UInt8<Fr>; 64] = UInt8::new_witness_vec(cs.clone(), &sig)
            .unwrap()
            .try_into()
            .unwrap();
        let message = UInt8::new_witness_vec(cs.clone(), msg).unwrap();
        match enforce_schnorr(&pk, &signature, &message) {
            Ok(()) => cs.is_satisfied().unwrap(),
            Err(_) => false,
        }
    }

    #[test]
    fn schnorr_canonical_known_answer() {
        assert!(schnorr_satisfied(
            kat_pk(),
            KAT_SIGNATURE,
            &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
        ));
    }

    #[test]
    fn pedersen_known_answer_is_constrained() {
        let cs = ConstraintSystem::<Fr>::new_ref();
        let inputs =
            [1u64, 2, 3].map(|n| FpVar::new_witness(cs.clone(), || Ok(Fr::from(n))).unwrap());
        let actual = pedersen_hash3(inputs).unwrap();
        let expected = FpVar::new_input(cs.clone(), || {
            Ok(MontFp!(
                "5487336162142086062598263185110674681975491452797383739382055710245001264198"
            ))
        })
        .unwrap();
        actual.enforce_equal(&expected).unwrap();
        assert!(cs.is_satisfied().unwrap());
    }

    #[test]
    fn canonical_field_bytes_preserve_leading_zeroes() {
        for value in [Fr::ZERO, Fr::ONE, Fr::from(256u64), -Fr::ONE] {
            let cs = ConstraintSystem::<Fr>::new_ref();
            let input = FpVar::new_witness(cs.clone(), || Ok(value)).unwrap();
            let bytes = field_to_be32(&input).unwrap();
            let expected = value.into_bigint().to_bytes_be();
            assert_eq!(expected.len(), 32);
            for (byte, expected) in bytes.iter().zip(expected) {
                byte.enforce_equal(&UInt8::constant(expected)).unwrap();
            }
            assert!(cs.is_satisfied().unwrap());
        }
    }

    #[test]
    fn canonical_field_bytes_reject_modulus_alias() {
        let cs = ConstraintSystem::<Fr>::new_ref();
        let input = FpVar::new_witness(cs.clone(), || Ok(Fr::ONE)).unwrap();
        let bytes = field_to_be32(&input).unwrap();
        let mut alias = Fr::MODULUS;
        assert!(!alias.add_with_carry(&Fr::ONE.into_bigint()));
        for (byte, expected) in bytes.iter().zip(alias.to_bytes_be()) {
            byte.enforce_equal(&UInt8::constant(expected)).unwrap();
        }
        assert!(!cs.is_satisfied().unwrap());
    }

    #[test]
    fn schnorr_rejects_raw_scalar_boundaries_and_tampering() {
        let message = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let mut variants = Vec::new();
        let mut zero_s = KAT_SIGNATURE;
        zero_s[..32].fill(0);
        variants.push(zero_s);
        let mut zero_e = KAT_SIGNATURE;
        zero_e[32..].fill(0);
        variants.push(zero_e);
        let mut changed_e = KAT_SIGNATURE;
        changed_e[63] ^= 1;
        variants.push(changed_e);
        let mut boundary = KAT_SIGNATURE;
        boundary[..32].copy_from_slice(&ark_grumpkin::Fr::MODULUS.to_bytes_be());
        variants.push(boundary);
        let mut noncanonical = KAT_SIGNATURE;
        let mut s = ark_grumpkin::Fr::from_be_bytes_mod_order(&KAT_SIGNATURE[..32]).into_bigint();
        assert!(!s.add_with_carry(&ark_grumpkin::Fr::MODULUS));
        noncanonical[..32].copy_from_slice(&s.to_bytes_be());
        // Native verification reduces s. This intentionally differs from ACVM
        // and our range-constrained gadget for the noncanonical representation.
        assert!(signature::verify_bytes(&kat_pk(), &noncanonical, &message));
        variants.push(noncanonical);
        for signature in variants {
            assert!(!schnorr_satisfied(kat_pk(), signature, &message));
        }
        let mut changed_message = message;
        changed_message[0] ^= 1;
        assert!(!schnorr_satisfied(
            kat_pk(),
            KAT_SIGNATURE,
            &changed_message
        ));
    }

    #[test]
    fn ownership_accepts_native_signature_with_raw_e_above_scalar_modulus() {
        let mut rng = StdRng::seed_from_u64(0x7472696275746500);
        let mut selected = None;
        let q = ark_grumpkin::Fr::MODULUS.to_bytes_be();
        for _ in 0..64 {
            let (sk, pk) = signature::keypair(&mut rng);
            let nonce = Fr::rand(&mut rng);
            let nft_hash = Fr::rand(&mut rng);
            let binding = Fr::from(7u64);
            let payload = hash::signing_payload(nft_hash, nonce, binding).unwrap();
            let signature = signature::sign(&mut rng, &sk, payload).unwrap();
            if signature[32..] >= q[..] {
                let (x, y) = signature::public_coords(&pk).unwrap();
                selected = Some((
                    noir::ownership_proof::Witness {
                        pk: noir::EmbeddedCurvePoint { x, y },
                        signature,
                        nonce,
                    },
                    noir::ownership_proof::PublicInputs {
                        owner: hash::derive_owner(&pk, nonce).unwrap(),
                        nft_hash,
                        binding_hash: binding,
                    },
                ));
                break;
            }
        }
        let slot = selected.expect("fixed seed must find an unreduced 256-bit challenge");
        let cs = ConstraintSystem::<Fr>::new_ref();
        FlatAggregationCircuit {
            binding_hash: slot.1.binding_hash,
            slots: vec![slot],
        }
        .generate_constraints(cs.clone())
        .unwrap();
        assert!(cs.is_satisfied().unwrap());
    }
}
