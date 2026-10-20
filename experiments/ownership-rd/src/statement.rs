//! Isolated ownership statements and public deterministic test fixtures.
//!
//! `Poseidon2` is a NEW, unaudited Schnorr authorization scheme. It is not
//! compatible with canonical Outbe signatures and must not authorize production
//! assets. Its nonce relies on the supplied cryptographic RNG (unlike canonical
//! signing's deterministic, auxiliary-entropy-hedged nonce). Fixture keys and
//! deterministic signing RNGs below are public test data, never production keys.

use ark_ec::{AffineRepr, CurveGroup, PrimeGroup};
use ark_ff::{BigInteger, Field, PrimeField, Zero};
use ark_grumpkin::{constraints::GVar, Affine, Fr as Scalar, Projective};
use ark_r1cs_std::{fields::fp::FpVar, prelude::*};
use ark_relations::gr1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_std::{
    rand::{rngs::StdRng, CryptoRng, RngCore, SeedableRng},
    UniformRand,
};
use outbe_protocol::{
    codec::field_to_be_bytes,
    primitive::{curve::base_to_scalar, hash, signature},
    protocol::{imt::Imt, zk::Circuit},
    Fr,
};
use outbe_zk_canonical::{
    demo_tribute::demo_tribute_domain,
    noir::{demo_tribute, ownership_proof, EmbeddedCurvePoint},
};

use crate::{baseline, poseidon2::poseidon2_hash3, Result};

pub const CAPACITIES: [usize; 7] = [1, 2, 4, 8, 16, 32, 64];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CircuitKind {
    FlatAggregation,
    DemoTribute,
}

impl CircuitKind {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "flat_aggregation" => Ok(Self::FlatAggregation),
            "demo_tribute" => Ok(Self::DemoTribute),
            _ => Err(format!("unknown ownership circuit {value:?}").into()),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::FlatAggregation => "flat_aggregation",
            Self::DemoTribute => "demo_tribute",
        }
    }
}

#[derive(Clone)]
pub enum CircuitFixture {
    Flat(Batch),
    Demo(DemoTributeCircuit),
}

impl CircuitFixture {
    pub fn public_inputs(&self) -> Vec<Fr> {
        match self {
            Self::Flat(batch) => batch.public_inputs(),
            Self::Demo(circuit) => demo_tribute::DemoTribute::public_inputs(&circuit.public),
        }
    }

    pub fn statement_digest(&self) -> String {
        match self {
            Self::Flat(batch) => batch.statement_digest(),
            Self::Demo(_) => digest_fields(self.public_inputs()),
        }
    }

    pub fn witness_digest(&self) -> String {
        match self {
            Self::Flat(batch) => batch.witness_digest(),
            Self::Demo(circuit) => digest_fields(demo_tribute::DemoTribute::witness_inputs(
                &circuit.witness,
                &circuit.public,
            )),
        }
    }
}

impl ConstraintSynthesizer<Fr> for CircuitFixture {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        match self {
            Self::Flat(batch) => batch.generate_constraints(cs),
            Self::Demo(circuit) => circuit.generate_constraints(cs),
        }
    }
}

#[derive(Clone)]
pub struct DemoTributeCircuit {
    pub scheme: Scheme,
    pub witness: demo_tribute::Witness,
    pub public: demo_tribute::PublicInputs,
}

impl ConstraintSynthesizer<Fr> for DemoTributeCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        if self.scheme == Scheme::Canonical {
            // Preserve the frozen experimental keys' exact allocation/constraint order.
            return baseline::DemoTributeCircuit {
                witness: self.witness,
                public: self.public,
            }
            .generate_constraints(cs);
        }
        let owner = FpVar::new_input(cs.clone(), || Ok(self.public.derived_owner))?;
        let nft_hash = FpVar::new_input(cs.clone(), || Ok(self.public.nft_hash))?;
        let binding = FpVar::new_input(cs.clone(), || Ok(self.public.binding_hash))?;
        let root = FpVar::new_input(cs.clone(), || Ok(self.public.merkle_root))?;
        let ownership = ownership_proof::Witness {
            pk: self.witness.pk.clone(),
            signature: self.witness.signature,
            nonce: self.witness.nonce,
        };
        enforce_slot(
            cs.clone(),
            self.scheme,
            &ownership,
            &owner,
            &nft_hash,
            &binding,
        )?;
        baseline::enforce_demo_inclusion(cs, &self.witness, nft_hash, &root)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scheme {
    Canonical,
    Poseidon2,
}

impl Scheme {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "canonical" => Ok(Self::Canonical),
            "poseidon2" => Ok(Self::Poseidon2),
            _ => Err(format!("unknown ownership scheme {value:?}").into()),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Canonical => "canonical",
            Self::Poseidon2 => "poseidon2",
        }
    }
}

pub type Slot = (ownership_proof::Witness, ownership_proof::PublicInputs);

#[derive(Clone)]
pub struct Batch {
    pub scheme: Scheme,
    pub slots: Vec<Slot>,
    pub binding_hash: Fr,
}

impl Batch {
    /// Original interleaved public ABI, without the R1CS constant-one slot.
    pub fn public_inputs(&self) -> Vec<Fr> {
        let mut fields = Vec::with_capacity(2 * self.slots.len() + 1);
        for (_, public) in &self.slots {
            fields.extend([public.owner, public.nft_hash]);
        }
        fields.push(self.binding_hash);
        fields
    }

    /// SHA-256 of consecutive canonical 32-byte BE public field words. No scheme
    /// tag: identical public statements must have identical digests across schemes.
    pub fn statement_digest(&self) -> String {
        digest_fields(self.public_inputs())
    }

    /// SHA-256 of the full original Noir ABI flattening: all pk coordinate pairs,
    /// then all signatures (each raw byte represented by one field word), then
    /// all nonces, then `public_inputs()`. Every word is canonical 32-byte BE.
    /// This is a comparison digest, not a compact witness serialization format.
    pub fn witness_digest(&self) -> String {
        let mut fields = Vec::with_capacity(69 * self.slots.len() + 1);
        for (witness, _) in &self.slots {
            fields.extend([witness.pk.x, witness.pk.y]);
        }
        for (witness, _) in &self.slots {
            fields.extend(witness.signature.iter().map(|byte| Fr::from(*byte)));
        }
        fields.extend(self.slots.iter().map(|(witness, _)| witness.nonce));
        fields.extend(self.public_inputs());
        digest_fields(fields)
    }
}

fn digest_fields(fields: Vec<Fr>) -> String {
    let mut bytes = Vec::with_capacity(32 * fields.len());
    for field in fields {
        bytes.extend_from_slice(&field_to_be_bytes(&field));
    }
    crate::sha(&bytes)
}

impl ConstraintSynthesizer<Fr> for Batch {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        if self.scheme == Scheme::Canonical {
            return baseline::FlatAggregationCircuit {
                slots: self.slots,
                binding_hash: self.binding_hash,
            }
            .generate_constraints(cs);
        }
        if !CAPACITIES.contains(&self.slots.len())
            || self
                .slots
                .iter()
                .any(|(_, p)| p.binding_hash != self.binding_hash)
        {
            return Err(SynthesisError::Unsatisfiable);
        }
        // Allocate every public field before any private variable, exactly as
        // the canonical baseline does.
        let mut public_vars = Vec::with_capacity(self.slots.len());
        for (_, public) in &self.slots {
            public_vars.push((
                FpVar::new_input(cs.clone(), || Ok(public.owner))?,
                FpVar::new_input(cs.clone(), || Ok(public.nft_hash))?,
            ));
        }
        let binding = FpVar::new_input(cs.clone(), || Ok(self.binding_hash))?;
        for ((witness, _), (owner, nft)) in self.slots.iter().zip(&public_vars) {
            enforce_slot(cs.clone(), self.scheme, witness, owner, nft, &binding)?;
        }
        Ok(())
    }
}

/// Data/key RNG seed: 0x6f776e6572736869 ^ (capacity << 32) ^ sample.
/// Per-slot signing RNG seed: data_seed ^ 0x7369676e61747572 ^ (index + 1).
/// Separate RNGs ensure signing internals never perturb later keys/nonces/NFTs.
/// These reproducible public fixtures are unrelated to OS-entropy key setup and
/// proof blinding, which are the proving engines' responsibility.
pub fn fixture(scheme: Scheme, capacity: usize, sample: u64) -> Result<Batch> {
    if !CAPACITIES.contains(&capacity) {
        return Err(format!("unsupported ownership capacity {capacity}").into());
    }
    let seed = 0x6f77_6e65_7273_6869 ^ ((capacity as u64) << 32) ^ sample;
    let mut data_rng = StdRng::seed_from_u64(seed);
    let binding_hash = Fr::from(7u64);
    let mut slots = Vec::with_capacity(capacity);
    for index in 0..capacity {
        let (sk, pk) = loop {
            let pair = signature::keypair(&mut data_rng);
            if !pair.0.is_zero() {
                break pair;
            }
        };
        let nonce = Fr::rand(&mut data_rng);
        let nft_hash = loop {
            let value = Fr::rand(&mut data_rng);
            if !value.is_zero() {
                break value;
            }
        };
        let owner = hash::derive_owner(&pk, nonce)?;
        let message = hash::signing_payload(nft_hash, nonce, binding_hash)?;
        let mut signing_rng =
            StdRng::seed_from_u64(seed ^ 0x7369_676e_6174_7572 ^ (index as u64 + 1));
        let signature = match scheme {
            Scheme::Canonical => signature::sign(&mut signing_rng, &sk, message)?,
            Scheme::Poseidon2 => sign_poseidon2(&mut signing_rng, &sk, message)?,
        };
        let (x, y) = signature::public_coords(&pk)?;
        slots.push((
            ownership_proof::Witness {
                pk: EmbeddedCurvePoint { x, y },
                signature,
                nonce,
            },
            ownership_proof::PublicInputs {
                owner,
                nft_hash,
                binding_hash,
            },
        ));
    }
    Ok(Batch {
        scheme,
        slots,
        binding_hash,
    })
}

/// Demo uses the selected ownership fixture unchanged, plus an actual depth-32
/// tree with eight nonzero leaves and the NFT at index 5 (both path directions).
pub fn circuit_fixture(
    kind: CircuitKind,
    scheme: Scheme,
    capacity: usize,
    sample: u64,
) -> Result<CircuitFixture> {
    if kind == CircuitKind::FlatAggregation {
        return Ok(CircuitFixture::Flat(fixture(scheme, capacity, sample)?));
    }
    if capacity != 1 {
        return Err("Demo Tribute requires capacity 1".into());
    }
    let batch = fixture(scheme, 1, sample)?;
    let (ownership, public) = batch
        .slots
        .into_iter()
        .next()
        .ok_or("missing ownership slot")?;
    // Keep the path RNG independent of the ownership fixture stream.
    let mut rng = StdRng::seed_from_u64(0x6465_6d6f_7061_7468 ^ sample);
    let mut tree = Imt::new(demo_tribute_domain(), Fr::zero(), 32)?;
    for index in 0..8 {
        let leaf = if index == 5 {
            public.nft_hash
        } else {
            loop {
                let leaf = Fr::rand(&mut rng);
                if !leaf.is_zero() {
                    break leaf;
                }
            }
        };
        tree.append(leaf)?;
    }
    let path = tree.inclusion_path(5)?;
    let indices = path.circuit_indices()?;
    Ok(CircuitFixture::Demo(DemoTributeCircuit {
        scheme,
        witness: demo_tribute::Witness {
            pk: ownership.pk,
            signature: ownership.signature,
            nonce: ownership.nonce,
            merkle_path_siblings: path
                .siblings
                .try_into()
                .map_err(|_| "invalid Demo path depth")?,
            merkle_path_indices: indices.try_into().map_err(|_| "invalid Demo index depth")?,
        },
        public: demo_tribute::PublicInputs {
            derived_owner: public.owner,
            nft_hash: public.nft_hash,
            binding_hash: public.binding_hash,
            merkle_root: tree.root(),
        },
    }))
}

fn signature_domain() -> Fr {
    // The 19-byte ASCII integer is strictly smaller than the BN254 modulus.
    Fr::from_be_bytes_mod_order(b"OUTBE_RD_SCHNORR_V1")
}

fn native_challenge(pk: &Affine, r: &Affine, message: Fr) -> Result<Fr> {
    let h0 = hash::poseidon2(&[signature_domain(), pk.x, pk.y])?;
    let h1 = hash::poseidon2(&[h0, r.x, r.y])?;
    Ok(hash::poseidon2(&[h1, message, Fr::from(6u64)])?)
}

/// Experimental Schnorr signer. `s` is a canonical Grumpkin scalar; `e` is a
/// canonical BN254 Fr challenge. Both are nonzero and encoded as BE s || e.
/// A repeated/broken RNG can repeat k and disclose the secret key: this is NOT
/// the hedged canonical signer and is intentionally confined to this R&D crate.
pub fn sign_poseidon2<R: RngCore + CryptoRng>(
    rng: &mut R,
    sk: &Scalar,
    message: Fr,
) -> Result<[u8; 64]> {
    if sk.is_zero() {
        return Err("experimental signing key must be nonzero".into());
    }
    let pk = signature::public_key(sk)?;
    loop {
        let k = Scalar::rand(rng);
        if k.is_zero() {
            continue;
        }
        let r = (Projective::generator() * k).into_affine();
        let e = native_challenge(&pk, &r, message)?;
        if e.is_zero() {
            continue;
        }
        let s = k - base_to_scalar(&e) * sk;
        if s.is_zero() {
            continue;
        }
        let mut bytes = [0u8; 64];
        bytes[..32].copy_from_slice(&s.into_bigint().to_bytes_be());
        bytes[32..].copy_from_slice(&field_to_be_bytes(&e));
        return Ok(bytes);
    }
}

fn canonical_nonzero<F: PrimeField>(bytes: &[u8]) -> Option<F> {
    let value = F::from_be_bytes_mod_order(bytes);
    (!value.is_zero() && value.into_bigint().to_bytes_be() == bytes).then_some(value)
}

/// Strict native oracle for the experimental verifier, including raw scalar
/// canonicality and invalid/identity public-key rejection, before curve arithmetic.
pub fn verify_poseidon2(pk: &Affine, message: Fr, bytes: &[u8; 64]) -> bool {
    if pk.is_zero() || !pk.is_on_curve() || !pk.is_in_correct_subgroup_assuming_on_curve() {
        return false;
    }
    let Some(s) = canonical_nonzero::<Scalar>(&bytes[..32]) else {
        return false;
    };
    let Some(e) = canonical_nonzero::<Fr>(&bytes[32..]) else {
        return false;
    };
    let r = (Projective::generator() * s + pk.mul_bigint(base_to_scalar(&e).into_bigint()))
        .into_affine();
    if r.is_zero() {
        return false;
    }
    native_challenge(pk, &r, message).is_ok_and(|actual| field_to_be_bytes(&actual) == bytes[32..])
}

/// Native statement oracle. Canonical s is checked here explicitly because the
/// production native signature helper reduces it, whereas the baseline R1CS
/// verifier (and pinned Noir implementation) requires a canonical scalar.
pub fn verify_slot(
    scheme: Scheme,
    witness: &ownership_proof::Witness,
    public: &ownership_proof::PublicInputs,
    binding: Fr,
) -> bool {
    let pk = Affine::new_unchecked(witness.pk.x, witness.pk.y);
    if public.binding_hash != binding
        || pk.is_zero()
        || !pk.is_on_curve()
        || !pk.is_in_correct_subgroup_assuming_on_curve()
        || canonical_nonzero::<Scalar>(&witness.signature[..32]).is_none()
    {
        return false;
    }
    let Ok(owner) = hash::derive_owner(&pk, witness.nonce) else {
        return false;
    };
    if owner != public.owner {
        return false;
    }
    let Ok(message) = hash::signing_payload(public.nft_hash, witness.nonce, binding) else {
        return false;
    };
    match scheme {
        Scheme::Canonical => signature::verify(&pk, message, &witness.signature),
        Scheme::Poseidon2 => verify_poseidon2(&pk, message, &witness.signature),
    }
}

pub fn enforce_slot(
    cs: ConstraintSystemRef<Fr>,
    scheme: Scheme,
    witness: &ownership_proof::Witness,
    owner: &FpVar<Fr>,
    nft: &FpVar<Fr>,
    binding: &FpVar<Fr>,
) -> Result<(), SynthesisError> {
    if scheme == Scheme::Canonical {
        return baseline::enforce_ownership(cs, witness, owner, nft, binding);
    }
    let x = FpVar::new_witness(cs.clone(), || Ok(witness.pk.x))?;
    let y = FpVar::new_witness(cs.clone(), || Ok(witness.pk.y))?;
    let nonce = FpVar::new_witness(cs.clone(), || Ok(witness.nonce))?;
    y.square()?
        .enforce_equal(&(x.square()? * &x - Fr::from(17u64)))?;
    (x.is_zero()? & y.is_zero()?).enforce_equal(&Boolean::FALSE)?;
    // Grumpkin has cofactor one; direct construction avoids native validation
    // panics on malicious coordinates, which are instead rejected by constraints.
    let pk = GVar::new(x.clone(), y.clone(), FpVar::one());
    poseidon2_hash3([x, y, nonce.clone()])?.enforce_equal(owner)?;
    let message = poseidon2_hash3([nft.clone(), nonce, binding.clone()])?;
    let bytes: [UInt8<Fr>; 64] = UInt8::new_witness_vec(cs, &witness.signature)?
        .try_into()
        .map_err(|_| SynthesisError::Unsatisfiable)?;
    enforce_poseidon2_signature(&pk, &bytes, &message)
}

fn signature_bits(bytes: &[UInt8<Fr>]) -> Result<Vec<Boolean<Fr>>, SynthesisError> {
    let mut bits = Vec::with_capacity(bytes.len() * 8);
    for byte in bytes.iter().rev() {
        bits.extend(byte.to_bits_le()?);
    }
    Ok(bits)
}

fn challenge_var(pk: &GVar, r: &GVar, message: &FpVar<Fr>) -> Result<FpVar<Fr>, SynthesisError> {
    let pk = pk.to_affine()?;
    let r = r.to_affine()?;
    let h0 = poseidon2_hash3([FpVar::Constant(signature_domain()), pk.x, pk.y])?;
    let h1 = poseidon2_hash3([h0, r.x, r.y])?;
    poseidon2_hash3([h1, message.clone(), FpVar::Constant(Fr::from(6u64))])
}

fn enforce_poseidon2_signature(
    pk: &GVar,
    bytes: &[UInt8<Fr>; 64],
    message: &FpVar<Fr>,
) -> Result<(), SynthesisError> {
    let s = signature_bits(&bytes[..32])?;
    let e = signature_bits(&bytes[32..])?;
    Boolean::kary_or(&s)?.enforce_equal(&Boolean::TRUE)?;
    Boolean::kary_or(&e)?.enforce_equal(&Boolean::TRUE)?;
    Boolean::enforce_smaller_or_equal_than_le(&s, (-Scalar::ONE).into_bigint())?;
    Boolean::enforce_smaller_or_equal_than_le(&e, (-Fr::ONE).into_bigint())?;
    // Keep all 256 raw bits, never silently reduce/truncate the signature bytes.
    let r = GVar::constant(Projective::generator()).scalar_mul_le(s.iter())?
        + pk.scalar_mul_le(e.iter())?;
    r.is_zero()?.enforce_equal(&Boolean::FALSE)?;
    let challenge = challenge_var(pk, &r, message)?;
    for (actual, expected) in baseline::field_to_be32(&challenge)?
        .iter()
        .zip(&bytes[32..])
    {
        actual.enforce_equal(expected)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_ff::AdditiveGroup;
    use ark_relations::gr1cs::{ConstraintSystem, OptimizationGoal, SynthesisMode};

    fn synthesize(
        batch: impl ConstraintSynthesizer<Fr>,
    ) -> Result<ConstraintSystemRef<Fr>, SynthesisError> {
        let cs = ConstraintSystem::<Fr>::new_ref();
        cs.set_optimization_goal(OptimizationGoal::Constraints);
        cs.set_mode(SynthesisMode::Prove {
            construct_matrices: true,
            generate_lc_assignments: false,
        });
        batch.generate_constraints(cs.clone())?;
        cs.finalize();
        Ok(cs)
    }

    fn satisfied(batch: impl ConstraintSynthesizer<Fr>) -> bool {
        match synthesize(batch) {
            Ok(cs) => cs.is_satisfied().unwrap(),
            Err(_) => false,
        }
    }

    fn native(batch: &Batch) -> bool {
        batch
            .slots
            .iter()
            .all(|(witness, public)| verify_slot(batch.scheme, witness, public, batch.binding_hash))
    }

    fn reject(batch: Batch) {
        assert!(!native(&batch), "native accepted invalid statement");
        assert!(!satisfied(batch), "constraints accepted invalid statement");
    }

    #[test]
    fn deterministic_common_fixtures_and_digest_encodings() {
        let canonical = fixture(Scheme::Canonical, 2, 11).unwrap();
        let experimental = fixture(Scheme::Poseidon2, 2, 11).unwrap();
        assert_eq!(canonical.public_inputs(), experimental.public_inputs());
        assert_eq!(
            canonical.statement_digest(),
            experimental.statement_digest()
        );
        assert_ne!(canonical.witness_digest(), experimental.witness_digest());
        assert_eq!(
            experimental.witness_digest(),
            fixture(Scheme::Poseidon2, 2, 11).unwrap().witness_digest()
        );
        for ((a, ap), (b, bp)) in canonical.slots.iter().zip(&experimental.slots) {
            assert_eq!((a.pk.x, a.pk.y, a.nonce), (b.pk.x, b.pk.y, b.nonce));
            assert_eq!(
                (ap.owner, ap.nft_hash, ap.binding_hash),
                (bp.owner, bp.nft_hash, bp.binding_hash)
            );
            assert!(!ap.nft_hash.is_zero());
        }
        let public_bytes: Vec<_> = experimental
            .public_inputs()
            .iter()
            .flat_map(field_to_be_bytes)
            .collect();
        assert_eq!(public_bytes.len(), 5 * 32);
        assert_eq!(experimental.statement_digest(), crate::sha(&public_bytes));
        // Independent manual flattening checks byte order and signature-byte expansion.
        let mut witness_bytes = Vec::new();
        for (w, _) in &experimental.slots {
            witness_bytes.extend(field_to_be_bytes(&w.pk.x));
            witness_bytes.extend(field_to_be_bytes(&w.pk.y));
        }
        for (w, _) in &experimental.slots {
            for byte in w.signature {
                witness_bytes.extend([0u8; 31]);
                witness_bytes.push(byte);
            }
        }
        for (w, _) in &experimental.slots {
            witness_bytes.extend(field_to_be_bytes(&w.nonce));
        }
        witness_bytes.extend(public_bytes);
        assert_eq!(witness_bytes.len(), (69 * 2 + 1) * 32);
        assert_eq!(experimental.witness_digest(), crate::sha(&witness_bytes));
    }

    #[test]
    fn original_and_new_schemes_verify_and_cross_reject() {
        for scheme in [Scheme::Canonical, Scheme::Poseidon2] {
            let batch = fixture(scheme, 1, 17).unwrap();
            assert!(native(&batch));
            assert!(satisfied(batch.clone()));
            let mut wrong_scheme = batch;
            wrong_scheme.scheme = if scheme == Scheme::Canonical {
                Scheme::Poseidon2
            } else {
                Scheme::Canonical
            };
            reject(wrong_scheme);
        }
    }

    #[test]
    fn native_and_constrained_challenges_agree() {
        let mut rng = StdRng::seed_from_u64(771);
        let (_, pk) = signature::keypair(&mut rng);
        let (_, r) = signature::keypair(&mut rng);
        for message in [Fr::ZERO, Fr::ONE, -Fr::ONE] {
            let cs = ConstraintSystem::<Fr>::new_ref();
            let pk_var = GVar::new_witness(cs.clone(), || Ok(pk.into_group())).unwrap();
            let r_var = GVar::new_witness(cs.clone(), || Ok(r.into_group())).unwrap();
            let message_var = FpVar::new_witness(cs.clone(), || Ok(message)).unwrap();
            let expected = FpVar::new_input(cs.clone(), || {
                native_challenge(&pk, &r, message).map_err(|_| SynthesisError::Unsatisfiable)
            })
            .unwrap();
            challenge_var(&pk_var, &r_var, &message_var)
                .unwrap()
                .enforce_equal(&expected)
                .unwrap();
            assert!(cs.is_satisfied().unwrap());
        }
    }

    #[test]
    fn rejects_binding_owner_message_nonce_and_signature_tampering() {
        for scheme in [Scheme::Canonical, Scheme::Poseidon2] {
            let original = fixture(scheme, 1, 23).unwrap();
            let mut owner = original.clone();
            owner.slots[0].1.owner += Fr::ONE;
            reject(owner);
            let mut nft = original.clone();
            nft.slots[0].1.nft_hash += Fr::ONE;
            reject(nft);
            let mut binding = original.clone();
            binding.binding_hash += Fr::ONE;
            binding.slots[0].1.binding_hash = binding.binding_hash;
            reject(binding);
            let mut nonce = original.clone();
            nonce.slots[0].0.nonce += Fr::ONE;
            // Keep ownership valid so failure specifically exercises the signed message.
            let pk = Affine::new_unchecked(nonce.slots[0].0.pk.x, nonce.slots[0].0.pk.y);
            nonce.slots[0].1.owner = hash::derive_owner(&pk, nonce.slots[0].0.nonce).unwrap();
            reject(nonce);
            let mut signature = original;
            signature.slots[0].0.signature[63] ^= 1;
            reject(signature);
        }
    }

    #[test]
    fn rejects_raw_scalar_boundaries_and_modulus_aliases() {
        for scheme in [Scheme::Canonical, Scheme::Poseidon2] {
            let original = fixture(scheme, 1, 31).unwrap();
            let mut zero_s = original.clone();
            zero_s.slots[0].0.signature[..32].fill(0);
            reject(zero_s);
            let mut zero_e = original.clone();
            zero_e.slots[0].0.signature[32..].fill(0);
            reject(zero_e);
            let mut at_q = original.clone();
            at_q.slots[0].0.signature[..32].copy_from_slice(&Scalar::MODULUS.to_bytes_be());
            reject(at_q);
            let mut alias_s = original.clone();
            let mut s =
                Scalar::from_be_bytes_mod_order(&alias_s.slots[0].0.signature[..32]).into_bigint();
            assert!(!s.add_with_carry(&Scalar::MODULUS));
            alias_s.slots[0].0.signature[..32].copy_from_slice(&s.to_bytes_be());
            reject(alias_s);
            if scheme == Scheme::Poseidon2 {
                let mut at_p = original.clone();
                at_p.slots[0].0.signature[32..].copy_from_slice(&Fr::MODULUS.to_bytes_be());
                reject(at_p);
                let mut alias_e = original;
                let mut e =
                    Fr::from_be_bytes_mod_order(&alias_e.slots[0].0.signature[32..]).into_bigint();
                assert!(!e.add_with_carry(&Fr::MODULUS));
                alias_e.slots[0].0.signature[32..].copy_from_slice(&e.to_bytes_be());
                reject(alias_e);
            }
        }
    }

    #[test]
    fn rejects_invalid_public_keys_and_identity_nonce_point() {
        for scheme in [Scheme::Canonical, Scheme::Poseidon2] {
            let original = fixture(scheme, 1, 37).unwrap();
            for (x, y) in [(Fr::ZERO, Fr::ZERO), (Fr::ONE, Fr::ONE)] {
                let mut bad = original.clone();
                bad.slots[0].0.pk = EmbeddedCurvePoint { x, y };
                reject(bad);
            }
            // s=e=1 and pk=-G make R exactly identity. The owner is correct,
            // isolating nonce-point rejection from owner/on-curve checks.
            let mut identity_r = original;
            let pk = (-Projective::generator()).into_affine();
            identity_r.slots[0].0.pk = EmbeddedCurvePoint { x: pk.x, y: pk.y };
            identity_r.slots[0].1.owner =
                hash::derive_owner(&pk, identity_r.slots[0].0.nonce).unwrap();
            identity_r.slots[0].0.signature[..32]
                .copy_from_slice(&Scalar::ONE.into_bigint().to_bytes_be());
            identity_r.slots[0].0.signature[32..].copy_from_slice(&field_to_be_bytes(&Fr::ONE));
            reject(identity_r);
        }
        let mut rng = StdRng::seed_from_u64(1);
        assert!(sign_poseidon2(&mut rng, &Scalar::ZERO, Fr::ONE).is_err());
    }

    #[test]
    fn public_order_capacity_binding_validation_and_tail_constraints() {
        assert_eq!(Scheme::parse("canonical").unwrap().name(), "canonical");
        assert_eq!(Scheme::parse("poseidon2").unwrap().name(), "poseidon2");
        assert!(Scheme::parse("Poseidon2").is_err());
        for scheme in [Scheme::Canonical, Scheme::Poseidon2] {
            let batch = fixture(scheme, 2, 41).unwrap();
            for capacity in [0, 3, 65] {
                assert!(fixture(scheme, capacity, 0).is_err());
                assert!(!satisfied(Batch {
                    scheme,
                    slots: vec![batch.slots[0].clone(); capacity],
                    binding_hash: batch.binding_hash,
                }));
            }
            let expected = batch.public_inputs();
            let cs = synthesize(batch.clone()).unwrap();
            assert_eq!(
                &cs.borrow().unwrap().instance_assignment().unwrap()[1..],
                expected.as_slice()
            );
            assert!(cs.is_satisfied().unwrap());
            let mut late = batch.clone();
            late.slots[1].1.owner += Fr::ONE;
            reject(late);
            let mut inconsistent = batch;
            inconsistent.slots[1].1.binding_hash += Fr::ONE;
            reject(inconsistent);
        }
    }

    #[test]
    fn demo_fixture_uses_inserted_index_five_and_canonical_flattening() -> Result<()> {
        let kind = CircuitKind::DemoTribute;
        assert_eq!(CircuitKind::parse(kind.name())?, kind);
        assert_eq!(
            CircuitKind::parse("flat_aggregation")?,
            CircuitKind::FlatAggregation
        );
        assert!(CircuitKind::parse("full_proof").is_err());
        for capacity in [0, 2, 64] {
            assert!(circuit_fixture(kind, Scheme::Canonical, capacity, 0).is_err());
        }
        let sample = 53;
        let demo = circuit_fixture(kind, Scheme::Canonical, 1, sample)?;
        let CircuitFixture::Demo(circuit) = &demo else {
            unreachable!()
        };
        let flat = fixture(Scheme::Canonical, 1, sample)?;
        let (ownership, public) = &flat.slots[0];
        assert_eq!(circuit.witness.signature, ownership.signature);
        assert_eq!(circuit.witness.nonce, ownership.nonce);
        assert_eq!(
            (circuit.witness.pk.x, circuit.witness.pk.y),
            (ownership.pk.x, ownership.pk.y)
        );
        assert_eq!(
            demo.public_inputs(),
            vec![
                public.owner,
                public.nft_hash,
                public.binding_hash,
                circuit.public.merkle_root,
            ]
        );
        let mut rng = StdRng::seed_from_u64(0x6465_6d6f_7061_7468 ^ sample);
        let mut tree = Imt::new(demo_tribute_domain(), Fr::ZERO, 32)?;
        for index in 0..8 {
            let leaf = if index == 5 {
                public.nft_hash
            } else {
                loop {
                    let leaf = Fr::rand(&mut rng);
                    if !leaf.is_zero() {
                        break leaf;
                    }
                }
            };
            tree.append(leaf)?;
        }
        let path = tree.inclusion_path(5)?;
        assert_eq!(path.root(public.nft_hash)?, tree.root());
        assert_eq!(circuit.public.merkle_root, tree.root());
        assert_eq!(
            circuit.witness.merkle_path_siblings.as_slice(),
            path.siblings.as_slice()
        );
        assert_eq!(
            circuit.witness.merkle_path_indices.as_slice(),
            path.circuit_indices()?.as_slice()
        );
        assert_eq!(
            &circuit.witness.merkle_path_indices[..3],
            &[false, true, false]
        );
        assert!(circuit.witness.merkle_path_indices[3..]
            .iter()
            .all(|bit| *bit));
        let public_bytes: Vec<_> = demo
            .public_inputs()
            .iter()
            .flat_map(field_to_be_bytes)
            .collect();
        let witness_bytes: Vec<_> =
            demo_tribute::DemoTribute::witness_inputs(&circuit.witness, &circuit.public)
                .iter()
                .flat_map(field_to_be_bytes)
                .collect();
        assert_eq!(demo.statement_digest(), crate::sha(&public_bytes));
        assert_eq!(demo.witness_digest(), crate::sha(&witness_bytes));
        assert_eq!(
            demo.witness_digest(),
            circuit_fixture(kind, Scheme::Canonical, 1, sample)?.witness_digest()
        );
        assert_ne!(
            demo.witness_digest(),
            circuit_fixture(kind, Scheme::Canonical, 1, sample + 1)?.witness_digest()
        );
        for scheme in [Scheme::Canonical, Scheme::Poseidon2] {
            let original = fixture(scheme, 2, sample)?;
            let dispatched = circuit_fixture(CircuitKind::FlatAggregation, scheme, 2, sample)?;
            assert_eq!(dispatched.statement_digest(), original.statement_digest());
            assert_eq!(dispatched.witness_digest(), original.witness_digest());
        }
        Ok(())
    }

    #[test]
    fn demo_constraints_reject_merkle_public_and_signature_mutations() -> Result<()> {
        for scheme in [Scheme::Canonical, Scheme::Poseidon2] {
            let CircuitFixture::Demo(original) =
                circuit_fixture(CircuitKind::DemoTribute, scheme, 1, 59)?
            else {
                unreachable!()
            };
            assert!(satisfied(original.clone()));
            for index in 0..4 {
                let mut bad = original.clone();
                match index {
                    0 => bad.public.derived_owner += Fr::ONE,
                    1 => bad.public.nft_hash += Fr::ONE,
                    2 => bad.public.binding_hash += Fr::ONE,
                    _ => bad.public.merkle_root += Fr::ONE,
                }
                assert!(!satisfied(bad), "accepted changed public field {index}");
            }
            // Exercise both directions and the otherwise-empty high levels.
            for level in [0, 1, 2, 31] {
                let mut sibling = original.clone();
                sibling.witness.merkle_path_siblings[level] += Fr::ONE;
                assert!(!satisfied(sibling), "accepted changed sibling {level}");
                let mut direction = original.clone();
                direction.witness.merkle_path_indices[level] ^= true;
                assert!(!satisfied(direction), "accepted changed direction {level}");
            }
            for byte in [0, 32, 63] {
                let mut bad = original.clone();
                bad.witness.signature[byte] ^= 1;
                assert!(!satisfied(bad), "accepted changed signature byte {byte}");
            }
        }
        Ok(())
    }

    #[test]
    fn demo_topology_and_public_allocation_are_value_independent() -> Result<()> {
        for scheme in [Scheme::Canonical, Scheme::Poseidon2] {
            let a_fixture = circuit_fixture(CircuitKind::DemoTribute, scheme, 1, 61)?;
            let public = a_fixture.public_inputs();
            let a = synthesize(a_fixture)?;
            let b = synthesize(circuit_fixture(CircuitKind::DemoTribute, scheme, 1, 67)?)?;
            assert!(a.is_satisfied()?);
            assert!(b.is_satisfied()?);
            assert_eq!(
                &a.borrow().unwrap().instance_assignment()?[1..],
                public.as_slice()
            );
            assert_eq!(a.to_matrices()?, b.to_matrices()?);
        }
        Ok(())
    }

    #[test]
    fn demo_signatures_cross_reject_the_same_public_statement() -> Result<()> {
        let canonical = circuit_fixture(CircuitKind::DemoTribute, Scheme::Canonical, 1, 71)?;
        let poseidon = circuit_fixture(CircuitKind::DemoTribute, Scheme::Poseidon2, 1, 71)?;
        assert_eq!(canonical.public_inputs(), poseidon.public_inputs());
        assert_eq!(canonical.statement_digest(), poseidon.statement_digest());
        assert_ne!(canonical.witness_digest(), poseidon.witness_digest());
        let (CircuitFixture::Demo(mut canonical), CircuitFixture::Demo(mut poseidon)) =
            (canonical, poseidon)
        else {
            unreachable!()
        };
        assert_eq!(
            canonical.witness.merkle_path_siblings,
            poseidon.witness.merkle_path_siblings
        );
        assert_eq!(
            canonical.witness.merkle_path_indices,
            poseidon.witness.merkle_path_indices
        );
        assert_eq!(canonical.witness.nonce, poseidon.witness.nonce);
        assert_eq!(
            (canonical.witness.pk.x, canonical.witness.pk.y),
            (poseidon.witness.pk.x, poseidon.witness.pk.y)
        );
        std::mem::swap(
            &mut canonical.witness.signature,
            &mut poseidon.witness.signature,
        );
        assert!(!satisfied(canonical));
        assert!(!satisfied(poseidon));
        Ok(())
    }

    #[test]
    fn canonical_demo_keeps_baseline_constraint_matrices() -> Result<()> {
        let CircuitFixture::Demo(circuit) =
            circuit_fixture(CircuitKind::DemoTribute, Scheme::Canonical, 1, 73)?
        else {
            unreachable!()
        };
        let baseline = synthesize(baseline::DemoTributeCircuit {
            witness: circuit.witness.clone(),
            public: circuit.public.clone(),
        })?;
        let selected = synthesize(circuit)?;
        assert!(baseline.is_satisfied()?);
        assert!(selected.is_satisfied()?);
        assert_eq!(baseline.to_matrices()?, selected.to_matrices()?);
        Ok(())
    }

    #[test]
    fn matrix_shape_is_independent_of_private_and_public_values() {
        for scheme in [Scheme::Canonical, Scheme::Poseidon2] {
            let a = synthesize(fixture(scheme, 2, 43).unwrap()).unwrap();
            let b = synthesize(fixture(scheme, 2, 47).unwrap()).unwrap();
            assert!(a.is_satisfied().unwrap());
            assert!(b.is_satisfied().unwrap());
            assert_eq!(a.to_matrices().unwrap(), b.to_matrices().unwrap());
        }
    }
}
