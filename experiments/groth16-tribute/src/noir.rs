//! Shared, deterministic inputs and the experiment-local ACVM adapter.
use std::{fs, io::Read, path::Path};

use acir::{
    circuit::Program,
    native_types::{Witness, WitnessMap, WitnessStack},
    FieldElement,
};
use acvm::pwg::{ACVMStatus, ACVM};
use ark_ff::{BigInteger, PrimeField, Zero};
use ark_relations::gr1cs::{
    ConstraintSynthesizer, ConstraintSystem, ConstraintSystemRef, OptimizationGoal, SynthesisError,
    SynthesisMode,
};
use ark_std::{
    rand::{rngs::StdRng, RngCore, SeedableRng},
    UniformRand,
};
use base64::Engine;
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use flate2::read::GzDecoder;
use outbe_protocol::{
    primitive::{curve::Scalar, hash, signature},
    protocol::{imt::Imt, zk::Circuit},
    Fr,
};
use outbe_zk_canonical::{
    demo_tribute::demo_tribute_domain,
    emit_mint::{hash as emit_hash, Pool as EmitPool},
    noir::{demo_tribute, emit_mint, ownership_proof, EmbeddedCurvePoint},
};

use crate::{
    circuit::{DemoTributeCircuit, FlatAggregationCircuit},
    emit::EmitCircuit,
    Result,
};

pub const CAPACITIES: [usize; 7] = [1, 2, 4, 8, 16, 32, 64];
pub type Slot = (ownership_proof::Witness, ownership_proof::PublicInputs);

#[derive(Clone)]
pub enum Fixture {
    Demo(DemoTributeCircuit),
    Flat(FlatAggregationCircuit),
    Emit(EmitCircuit),
}

impl ConstraintSynthesizer<Fr> for Fixture {
    fn generate_constraints(
        self,
        cs: ConstraintSystemRef<Fr>,
    ) -> std::result::Result<(), SynthesisError> {
        match self {
            Self::Demo(c) => c.generate_constraints(cs),
            Self::Flat(c) => c.generate_constraints(cs),
            Self::Emit(c) => c.generate_constraints(cs),
        }
    }
}

pub fn validate(fixture: &Fixture) -> Result<()> {
    if let Fixture::Flat(c) = fixture {
        if !CAPACITIES.contains(&c.slots.len()) {
            return Err(format!("unsupported flat capacity {}", c.slots.len()).into());
        }
        if c.slots
            .iter()
            .any(|(_, p)| p.binding_hash != c.binding_hash)
        {
            return Err("slot binding differs from shared binding".into());
        }
    }
    Ok(())
}

fn slot(rng: &mut StdRng, binding: Fr, padding: bool) -> Result<Slot> {
    let (sk, pk) = signature::keypair(rng);
    let nonce = if padding { Fr::zero() } else { Fr::rand(rng) };
    let nft_hash = if padding {
        Fr::zero()
    } else {
        loop {
            let x = Fr::rand(rng);
            if !x.is_zero() {
                break x;
            }
        }
    };
    let owner = hash::derive_owner(&pk, nonce)?;
    let payload = hash::signing_payload(nft_hash, nonce, binding)?;
    let signature = signature::sign(rng, &sk, payload)?;
    let (x, y) = signature::public_coords(&pk)?;
    Ok((
        ownership_proof::Witness {
            pk: EmbeddedCurvePoint { x, y },
            signature,
            nonce,
        },
        ownership_proof::PublicInputs {
            owner,
            nft_hash,
            binding_hash: binding,
        },
    ))
}

/// Zero-valued padding is a fully signed ownership statement, not an inactive slot.
pub fn pad_slots(
    mut slots: Vec<Slot>,
    capacity: usize,
    binding: Fr,
    rng: &mut StdRng,
) -> Result<FlatAggregationCircuit> {
    if !CAPACITIES.contains(&capacity) || slots.len() > capacity {
        return Err(format!(
            "invalid padding capacity {capacity} for {} slots",
            slots.len()
        )
        .into());
    }
    if slots.iter().any(|(_, p)| p.binding_hash != binding) {
        return Err("slot binding differs from shared binding".into());
    }
    while slots.len() < capacity {
        slots.push(slot(rng, binding, true)?);
    }
    Ok(FlatAggregationCircuit {
        slots,
        binding_hash: binding,
    })
}

const EMIT_RADIX: u128 = 1u128 << 120;
const EMIT_NOTE: [u128; 3] = [5, 7, 2];
const EMIT_MINT: [u128; 3] = [9, 8, 1];

fn emit_owner(rng: &mut StdRng) -> Fr {
    let mut bytes = [0; 20];
    rng.fill_bytes(&mut bytes);
    Fr::from_be_bytes_mod_order(&bytes)
}

fn emit_spend_key(rng: &mut StdRng) -> Fr {
    loop {
        let key = Fr::rand(rng);
        if !key.is_zero() {
            return key;
        }
    }
}

fn emit_serial(owner: Fr, key: Fr) -> Result<Fr> {
    let bytes = be32(owner.into_bigint());
    if bytes[..12].iter().all(|b| *b == 0) {
        return Ok(emit_hash::note_sn_raw(bytes[12..].try_into()?, key)?);
    }
    // Only the owner-range rejection fixture takes this path: preserve the
    // tagged native formula while recomputing its otherwise-valid relations.
    Ok(EmitPool::hash_multi(
        emit_hash::tag_note_sn()?,
        &[owner, key],
    )?)
}

fn emit_remaining(note: [u128; 3], mint: [u128; 3]) -> Result<[u128; 3]> {
    let mut remaining = [0; 3];
    let mut borrow = 0;
    for i in 0..3 {
        let subtrahend = mint[i] + borrow;
        borrow = u128::from(note[i] < subtrahend);
        remaining[i] = note[i] + borrow * EMIT_RADIX - subtrahend;
    }
    if borrow != 0 {
        return Err("Emit fixture mint exceeds note".into());
    }
    Ok(remaining)
}

/// Recompute all native relations, including an actual index-5 insertion.
/// The range-negative checks also use this to avoid testing only stale hashes.
fn refresh_emit(c: &mut EmitCircuit, rng: &mut StdRng) -> Result<()> {
    let chain = c.public.chain_id;
    let serial = emit_serial(c.public.note_owner, c.witness.note_spend_key)?;
    let commitment =
        emit_hash::note_commitment_raw(chain, serial, c.witness.note_amount.map(Fr::from))?;
    c.public.nullifier = emit_hash::nullifier(commitment, c.witness.note_spend_key)?;
    let remaining = emit_remaining(c.witness.note_amount, c.public.mint_units)?;
    c.public.change_commitment = if remaining == [0; 3] {
        Fr::zero()
    } else {
        let next_key = emit_hash::change_key(c.witness.note_spend_key, c.public.nullifier)?;
        let next_serial = emit_serial(c.public.note_owner, next_key)?;
        let next_commitment =
            emit_hash::note_commitment_raw(chain, next_serial, remaining.map(Fr::from))?;
        if next_key.is_zero()
            || next_serial.is_zero()
            || next_commitment.is_zero()
            || emit_hash::nullifier(next_commitment, next_key)?.is_zero()
        {
            return Err("degenerate Emit change fixture".into());
        }
        next_commitment
    };
    if serial.is_zero() || commitment.is_zero() || c.public.nullifier.is_zero() {
        return Err("degenerate Emit note fixture".into());
    }
    let mut tree = Imt::new(emit_hash::emit_domain(), emit_hash::empty_leaf(chain)?, 32)?;
    for index in 0..8 {
        let leaf = if index == 5 {
            commitment
        } else {
            let owner = emit_owner(rng);
            let key = emit_spend_key(rng);
            let amount = [
                rng.next_u64() as u128,
                rng.next_u64() as u128,
                (rng.next_u64() & 0xffff) as u128,
            ];
            emit_hash::note_commitment_raw(chain, emit_serial(owner, key)?, amount.map(Fr::from))?
        };
        tree.append(leaf)?;
    }
    let path = tree.inclusion_path(5)?;
    if path.root(commitment)? != tree.root() {
        return Err("Emit inserted leaf path mismatch".into());
    }
    c.witness.leaf_index = 5;
    c.witness.auth_path = path
        .siblings
        .try_into()
        .map_err(|_| "Emit sibling length")?;
    c.public.root = tree.root();
    Ok(())
}

fn emit_fixture(
    rng: &mut StdRng,
    note_amount: [u128; 3],
    mint_units: [u128; 3],
) -> Result<EmitCircuit> {
    let mut c = EmitCircuit {
        witness: emit_mint::Witness {
            note_amount,
            note_spend_key: emit_spend_key(rng),
            leaf_index: 5,
            auth_path: [Fr::zero(); 32],
        },
        public: emit_mint::PublicInputs {
            chain_id: 31_337,
            root: Fr::zero(),
            nullifier: Fr::zero(),
            note_owner: emit_owner(rng),
            mint_units,
            change_commitment: Fr::zero(),
        },
    };
    refresh_emit(&mut c, rng)?;
    Ok(c)
}

pub(crate) fn emit_fixture_with_amounts(
    note_amount: [u128; 3],
    mint_units: [u128; 3],
    sample: u64,
) -> Result<EmitCircuit> {
    let mut rng = StdRng::seed_from_u64(0x7472696275746500 ^ (8u64 << 32) ^ sample);
    emit_fixture(&mut rng, note_amount, mint_units)
}

pub fn fixture(case: &str, sample: u64) -> Result<Fixture> {
    let index = crate::CASES
        .iter()
        .position(|name| *name == case)
        .ok_or("unknown circuit case")?;
    if index == 8 {
        return Ok(Fixture::Emit(emit_fixture_with_amounts(
            EMIT_NOTE, EMIT_MINT, sample,
        )?));
    }
    let mut rng = StdRng::seed_from_u64(0x7472696275746500 ^ ((index as u64) << 32) ^ sample);
    let binding = Fr::from(7u64);
    if index != 0 {
        let slots = (0..CAPACITIES[index - 1])
            .map(|_| slot(&mut rng, binding, false))
            .collect::<Result<Vec<_>>>()?;
        return Ok(Fixture::Flat(FlatAggregationCircuit {
            slots,
            binding_hash: binding,
        }));
    }
    let (w, p) = slot(&mut rng, binding, false)?;
    let mut tree = Imt::new(demo_tribute_domain(), Fr::zero(), 32)?;
    for i in 0..8 {
        let leaf = if i == 5 {
            p.nft_hash
        } else {
            slot(&mut rng, binding, false)?.1.nft_hash
        };
        tree.append(leaf)?;
    }
    let path = tree.inclusion_path(5)?;
    let indices: [bool; 32] = path
        .circuit_indices()?
        .try_into()
        .map_err(|_| "direction length")?;
    if !indices.iter().any(|b| *b) || !indices.iter().any(|b| !b) {
        return Err("fixture lacks both Merkle branches".into());
    }
    if path.root(p.nft_hash)? != tree.root() {
        return Err("inserted leaf path mismatch".into());
    }
    Ok(Fixture::Demo(DemoTributeCircuit {
        witness: demo_tribute::Witness {
            pk: w.pk,
            signature: w.signature,
            nonce: w.nonce,
            merkle_path_siblings: path.siblings.try_into().map_err(|_| "sibling length")?,
            merkle_path_indices: indices,
        },
        public: demo_tribute::PublicInputs {
            derived_owner: p.owner,
            nft_hash: p.nft_hash,
            binding_hash: binding,
            merkle_root: tree.root(),
        },
    }))
}

pub fn public_inputs(f: &Fixture) -> Vec<Fr> {
    match f {
        Fixture::Demo(c) => demo_tribute::DemoTribute::public_inputs(&c.public),
        Fixture::Emit(c) => emit_mint::EmitMint::public_inputs(&c.public),
        Fixture::Flat(c) => {
            let mut fields = Vec::with_capacity(2 * c.slots.len() + 1);
            for (_, p) in &c.slots {
                fields.extend([p.owner, p.nft_hash]);
            }
            fields.push(c.binding_hash);
            fields
        }
    }
}

pub fn input_fields(f: &Fixture) -> Vec<Fr> {
    match f {
        Fixture::Demo(c) => demo_tribute::DemoTribute::witness_inputs(&c.witness, &c.public),
        Fixture::Emit(c) => emit_mint::EmitMint::witness_inputs(&c.witness, &c.public),
        Fixture::Flat(c) => {
            // Historical n2 ABI at a9fe83ed: pk[N], signature[N][64], nonce[N], public_inputs[2N+1].
            let mut fields = Vec::with_capacity(69 * c.slots.len() + 1);
            for (w, _) in &c.slots {
                fields.extend([w.pk.x, w.pk.y]);
            }
            for (w, _) in &c.slots {
                fields.extend(w.signature.iter().map(|b| Fr::from(*b as u64)));
            }
            fields.extend(c.slots.iter().map(|(w, _)| w.nonce));
            fields.extend(public_inputs(f));
            fields
        }
    }
}

pub fn solve(
    program: &Program<FieldElement>,
    fields: &[Fr],
    expected_public: &[Fr],
) -> Result<WitnessStack<FieldElement>> {
    let circuit = program
        .functions
        .first()
        .ok_or("ACIR has no entry function")?;
    let mut map = WitnessMap::new();
    for (i, f) in fields.iter().enumerate() {
        map.insert(Witness(i as u32), FieldElement::from_repr(*f));
    }
    let solver = Bn254BlackBoxSolver;
    let mut acvm = ACVM::new(
        &solver,
        &circuit.opcodes,
        map,
        &program.unconstrained_functions,
        &circuit.assert_messages,
    );
    match acvm.solve() {
        ACVMStatus::Solved => {}
        status => return Err(format!("ACVM solve: {status}").into()),
    }
    let solved = acvm.finalize();
    let public = circuit.public_inputs();
    if public.0.len() != expected_public.len() {
        return Err("ACIR public arity differs from fixture".into());
    }
    for (index, expected) in public.0.iter().zip(expected_public) {
        if solved.get(index) != Some(&FieldElement::from_repr(*expected)) {
            return Err(format!("ACIR public value differs at witness {}", index.0).into());
        }
    }
    let mut stack = WitnessStack::default();
    stack.push(0, solved);
    Ok(stack)
}

pub fn encode_witness(stack: &WitnessStack<FieldElement>) -> Result<Vec<u8>> {
    let compressed = stack.serialize()?;
    let mut bytes = Vec::new();
    GzDecoder::new(compressed.as_slice()).read_to_end(&mut bytes)?;
    Ok(bytes)
}

pub fn load_program(out: &Path, case: &str) -> Result<Program<FieldElement>> {
    let encoded = fs::read_to_string(out.join("artifacts").join(case).join("bytecode.b64"))?;
    let bytes = base64::engine::general_purpose::STANDARD.decode(encoded.trim())?;
    Ok(Program::deserialize_program(&bytes)?)
}

fn synthesize(f: &Fixture) -> Result<ConstraintSystemRef<Fr>> {
    validate(f)?;
    let cs = ConstraintSystem::new_ref();
    cs.set_optimization_goal(OptimizationGoal::Constraints);
    cs.set_mode(SynthesisMode::Prove {
        construct_matrices: true,
        generate_lc_assignments: false,
    });
    f.clone().generate_constraints(cs.clone())?;
    cs.finalize();
    Ok(cs)
}

fn accepted(f: &Fixture) -> Result<bool> {
    match synthesize(f) {
        Ok(cs) => Ok(cs.is_satisfied()?),
        Err(_) => Ok(false),
    }
}

fn parity(program: &Program<FieldElement>, f: &Fixture, expected: bool, label: &str) -> Result<()> {
    // Deliberately do not catch panics: a panic aborts the check, never counts as rejection.
    let r1cs = accepted(f)?;
    let acvm = solve(program, &input_fields(f), &public_inputs(f)).is_ok();
    if r1cs != expected || acvm != expected {
        return Err(format!("{label}: expected {expected}, R1CS={r1cs}, ACVM={acvm}").into());
    }
    Ok(())
}

fn with_slot(
    f: &mut Fixture,
    index: usize,
    change: impl FnOnce(&mut ownership_proof::Witness, &mut ownership_proof::PublicInputs),
) {
    match f {
        Fixture::Flat(c) => {
            let (w, p) = &mut c.slots[index];
            change(w, p);
        }
        Fixture::Demo(c) => {
            let mut w = ownership_proof::Witness {
                pk: c.witness.pk.clone(),
                signature: c.witness.signature,
                nonce: c.witness.nonce,
            };
            let mut p = ownership_proof::PublicInputs {
                owner: c.public.derived_owner,
                nft_hash: c.public.nft_hash,
                binding_hash: c.public.binding_hash,
            };
            change(&mut w, &mut p);
            c.witness.pk = w.pk;
            c.witness.signature = w.signature;
            c.witness.nonce = w.nonce;
            c.public.derived_owner = p.owner;
            c.public.nft_hash = p.nft_hash;
            c.public.binding_hash = p.binding_hash;
        }
        Fixture::Emit(_) => unreachable!("Emit has no ownership slot"),
    }
}

fn be32(n: impl BigInteger) -> [u8; 32] {
    let bytes = n.to_bytes_be();
    let mut out = [0; 32];
    out[32 - bytes.len()..].copy_from_slice(&bytes);
    out
}

fn add_be(a: &[u8], b: &[u8]) -> Option<[u8; 32]> {
    let mut out = [0; 32];
    let mut carry = 0u16;
    for i in (0..32).rev() {
        let sum = a[i] as u16 + b[i] as u16 + carry;
        out[i] = sum as u8;
        carry = sum >> 8;
    }
    (carry == 0).then_some(out)
}

fn check_emit_constraints(program: &Program<FieldElement>, valid: &EmitCircuit) -> Result<()> {
    let mut rng = StdRng::seed_from_u64(0x656d69745f63686b);
    let full = emit_fixture(&mut rng, EMIT_NOTE, EMIT_NOTE)?;
    // Radix 2^120, deliberately including the frozen bignum's 17th top bit.
    // This ports its relation, not a tightened mathematical uint256 type.
    for (label, note, mint) in [
        ("full mint", EMIT_NOTE, EMIT_NOTE),
        ("cross-120 borrow", [1, 4, 2], [2, 1, 1]),
        ("cross-240 borrow", [1, 0, 2], [2, 0, 1]),
        (
            "maximum uint256",
            [EMIT_RADIX - 1, EMIT_RADIX - 1, 65_535],
            EMIT_MINT,
        ),
        ("bit-256 full mint", [0, 0, 65_536], [0, 0, 65_536]),
        ("top-17 partial mint", [5, 7, 131_071], EMIT_MINT),
        (
            "top-17 full mint",
            [EMIT_RADIX - 1, EMIT_RADIX - 1, 131_071],
            [EMIT_RADIX - 1, EMIT_RADIX - 1, 131_071],
        ),
    ] {
        let c = emit_fixture(&mut rng, note, mint)?;
        parity(program, &Fixture::Emit(c), true, &format!("Emit {label}"))?;
    }
    if emit_remaining(EMIT_NOTE, EMIT_MINT)? != [EMIT_RADIX - 4, EMIT_RADIX - 2, 0]
        || valid.public.change_commitment.is_zero()
        || !full.public.change_commitment.is_zero()
    {
        return Err("Emit fixture borrow/change semantics differ".into());
    }

    for public_index in 0..8 {
        let mut c = valid.clone();
        match public_index {
            0 => c.public.chain_id += 1,
            1 => c.public.root += Fr::from(1u64),
            2 => c.public.nullifier += Fr::from(1u64),
            3 => c.public.note_owner += Fr::from(1u64),
            4..=6 => c.public.mint_units[public_index - 4] += 1,
            7 => c.public.change_commitment += Fr::from(1u64),
            _ => unreachable!(),
        }
        parity(
            program,
            &Fixture::Emit(c),
            false,
            &format!("Emit public field {public_index}"),
        )?;
    }
    for limb in 0..3 {
        let mut c = valid.clone();
        c.witness.note_amount[limb] += 1;
        parity(
            program,
            &Fixture::Emit(c),
            false,
            &format!("Emit note amount limb {limb}"),
        )?;
    }
    for (mutation, label) in [
        "zero spend key",
        "changed spend key",
        "changed leaf direction",
        "changed leaf index high bit",
        "changed first sibling",
        "changed last sibling",
        "reordered authentication path",
        "zero mint",
        "mint exceeds note",
        "zero partial change",
        "zero nullifier",
    ]
    .iter()
    .enumerate()
    {
        let mut c = valid.clone();
        match mutation {
            0 => c.witness.note_spend_key = Fr::zero(),
            1 => c.witness.note_spend_key += Fr::from(1u64),
            2 => c.witness.leaf_index ^= 1,
            3 => c.witness.leaf_index ^= 1 << 31,
            4 => c.witness.auth_path[0] += Fr::from(1u64),
            5 => c.witness.auth_path[31] += Fr::from(1u64),
            6 => c.witness.auth_path.swap(0, 1),
            7 => c.public.mint_units = [0; 3],
            8 => c.public.mint_units = [6, 7, 2],
            9 => c.public.change_commitment = Fr::zero(),
            10 => c.public.nullifier = Fr::zero(),
            _ => unreachable!(),
        }
        parity(program, &Fixture::Emit(c), false, &format!("Emit {label}"))?;
    }
    for limb in 0..3 {
        let invalid = if limb == 2 { 1u128 << 17 } else { EMIT_RADIX };
        for mint in [false, true] {
            let mut c = valid.clone();
            if mint {
                c.public.mint_units[limb] = invalid;
            } else {
                c.witness.note_amount[limb] = invalid;
            }
            parity(
                program,
                &Fixture::Emit(c),
                false,
                &format!(
                    "Emit noncanonical {} limb {limb}",
                    if mint { "mint" } else { "note" }
                ),
            )?;
        }
        // Recompute the commitment, tree and nullifier for an equal-value full
        // mint: only the noncanonical amounts should prevent acceptance.
        let mut c = full.clone();
        c.witness.note_amount[limb] = invalid;
        c.public.mint_units = c.witness.note_amount;
        refresh_emit(&mut c, &mut rng)?;
        parity(
            program,
            &Fixture::Emit(c),
            false,
            &format!("Emit recomputed noncanonical full mint limb {limb}"),
        )?;
    }
    for change in [Fr::from(1u64), valid.public.change_commitment] {
        let mut c = full.clone();
        c.public.change_commitment = change;
        parity(
            program,
            &Fixture::Emit(c),
            false,
            "Emit nonzero full-mint change",
        )?;
    }
    let mut invalid_owner = full.clone();
    let mut high_bit = [0; 32];
    high_bit[11] = 1; // 2^160, just outside EthAddress.
    invalid_owner.public.note_owner += Fr::from_be_bytes_mod_order(&high_bit);
    refresh_emit(&mut invalid_owner, &mut rng)?;
    parity(
        program,
        &Fixture::Emit(invalid_owner),
        false,
        "Emit owner >= 2^160 with recomputed relations",
    )?;
    for zero_key in [false, true] {
        let mut c = valid.clone();
        let label = if zero_key {
            c.witness.note_spend_key = Fr::zero();
            "Emit zero spend key with recomputed relations"
        } else {
            c.public.mint_units = [0; 3];
            "Emit zero mint with recomputed relations"
        };
        refresh_emit(&mut c, &mut rng)?;
        parity(program, &Fixture::Emit(c), false, label)?;
    }
    let partial_matrices = synthesize(&Fixture::Emit(valid.clone()))?.to_matrices()?;
    let full_matrices = synthesize(&Fixture::Emit(full))?.to_matrices()?;
    if partial_matrices != full_matrices {
        return Err("Emit full/partial witness-dependent matrix topology".into());
    }
    Ok(())
}

/// Differential statement checks only. Proof and serialization attacks live in main.
pub fn check_constraints(out: &Path) -> Result<()> {
    for case in crate::CASES {
        let program = load_program(out, case)?;
        let valid = fixture(case, 1)?;
        parity(&program, &valid, true, &format!("{case} valid"))?;
        if let Fixture::Emit(c) = &valid {
            check_emit_constraints(&program, c)?;
            continue;
        }
        for mutation in 0..11 {
            let mut bad = valid.clone();
            with_slot(&mut bad, 0, |w, p| match mutation {
                0 => p.owner += Fr::from(1u64),
                1 => p.nft_hash += Fr::from(1u64),
                2 => p.binding_hash += Fr::from(1u64),
                3 => w.nonce += Fr::from(1u64),
                4 => w.signature[0] ^= 1,
                5 => {
                    w.pk.x = Fr::zero();
                    w.pk.y = Fr::zero();
                }
                6 => w.pk.y += Fr::from(1u64),
                7 => w.signature[..32].fill(0),
                8 => w.signature[32..].fill(0),
                9 => w.signature[63] ^= 1,
                10 => w.signature[..32].copy_from_slice(&be32(Scalar::MODULUS)),
                _ => unreachable!(),
            });
            // The circuit has a single shared binding; mutate it as well as the slot's host metadata.
            if mutation == 2 {
                if let Fixture::Flat(c) = &mut bad {
                    c.binding_hash += Fr::from(1u64);
                    for (_, p) in &mut c.slots {
                        p.binding_hash = c.binding_hash;
                    }
                }
            }
            parity(
                &program,
                &bad,
                false,
                &format!("{case} mutation {mutation}"),
            )?;
        }
    }
    let demo_program = load_program(out, "demo_tribute")?;
    let demo = fixture("demo_tribute", 1)?;
    for mutation in 0..3 {
        let mut bad = demo.clone();
        if let Fixture::Demo(c) = &mut bad {
            match mutation {
                0 => c.public.merkle_root += Fr::from(1u64),
                1 => c.witness.merkle_path_siblings[0] += Fr::from(1u64),
                _ => c.witness.merkle_path_indices[0] = !c.witness.merkle_path_indices[0],
            }
        }
        parity(&demo_program, &bad, false, "Demo Merkle mutation")?;
    }
    // Private Demo ABI: pk(2), signature(64), nonce(1), siblings(32), directions(32).
    let mut raw = input_fields(&demo);
    raw[99] = Fr::from(2u64);
    if solve(&demo_program, &raw, &public_inputs(&demo)).is_ok() {
        return Err("raw Noir direction 2 accepted".into());
    }
    // The corresponding representable Boolean flips the direction and must fail.
    let mut flipped = demo.clone();
    if let Fixture::Demo(c) = &mut flipped {
        c.witness.merkle_path_indices[0] = !c.witness.merkle_path_indices[0];
    }
    if accepted(&flipped)? {
        return Err("Boolean direction mutation accepted".into());
    }

    let n1_program = load_program(out, "flat_aggregation_n1")?;
    let q = be32(Scalar::MODULUS);
    let mut high_e = None;
    let mut noncanonical = None;
    // Fixed-seed search, with explicit failure rather than silently omitting a boundary.
    for sample in 0..256 {
        let f = fixture("flat_aggregation_n1", sample)?;
        if let Fixture::Flat(c) = &f {
            let sig = &c.slots[0].0.signature;
            if high_e.is_none() && sig[32..] >= q[..] {
                high_e = Some(f.clone());
            }
            if noncanonical.is_none() {
                if let Some(sum) = add_be(&sig[..32], &q) {
                    let mut bad = f.clone();
                    with_slot(&mut bad, 0, |w, _| w.signature[..32].copy_from_slice(&sum));
                    noncanonical = Some(bad);
                }
            }
        }
        if high_e.is_some() && noncanonical.is_some() {
            break;
        }
    }
    parity(
        &n1_program,
        &high_e.ok_or("no fixed-seed e >= q fixture")?,
        true,
        "raw e >= q",
    )?;
    parity(
        &n1_program,
        &noncanonical.ok_or("no nonoverflowing s+q fixture")?,
        false,
        "noncanonical s+q",
    )?;

    let mut rng = StdRng::seed_from_u64(0x70616464696e67);
    let binding = Fr::from(7u64);
    let real = if let Fixture::Flat(c) = fixture("flat_aggregation_n4", 2)? {
        c.slots[..3].to_vec()
    } else {
        unreachable!()
    };
    let padded = Fixture::Flat(pad_slots(real.clone(), 4, binding, &mut rng)?);
    let n4_program = load_program(out, "flat_aggregation_n4")?;
    parity(&n4_program, &padded, true, "three real plus one padding")?;
    for signature_mutation in [false, true] {
        let mut bad = padded.clone();
        with_slot(&mut bad, 3, |w, p| {
            if signature_mutation {
                w.signature[63] ^= 1
            } else {
                p.owner += Fr::from(1u64)
            }
        });
        parity(&n4_program, &bad, false, "padding mutation")?;
    }
    parity(
        &n1_program,
        &Fixture::Flat(pad_slots(Vec::new(), 1, binding, &mut rng)?),
        true,
        "entirely padded n1",
    )?;
    if pad_slots(real.clone(), 2, binding, &mut rng).is_ok()
        || pad_slots(Vec::new(), 3, binding, &mut rng).is_ok()
    {
        return Err("padding overflow/unsupported capacity accepted".into());
    }
    if pad_slots(real, 4, binding + Fr::from(1u64), &mut rng).is_ok() {
        return Err("inconsistent binding accepted".into());
    }
    for capacity in [0, 3, 65] {
        let invalid = Fixture::Flat(FlatAggregationCircuit {
            slots: Vec::new(),
            binding_hash: binding,
        });
        if fixture(&format!("flat_aggregation_n{capacity}"), 0).is_ok()
            || validate(&invalid).is_ok()
        {
            return Err("invalid shape accepted".into());
        }
    }
    let mut inconsistent = fixture("flat_aggregation_n2", 0)?;
    with_slot(&mut inconsistent, 1, |_, p| {
        p.binding_hash += Fr::from(1u64)
    });
    if validate(&inconsistent).is_ok() {
        return Err("inconsistent fixture binding accepted".into());
    }
    let mut tail = fixture("flat_aggregation_n64", 1)?;
    with_slot(&mut tail, 63, |_, p| p.owner += Fr::from(1u64));
    parity(
        &load_program(out, "flat_aggregation_n64")?,
        &tail,
        false,
        "last n64 slot",
    )?;
    for case in ["demo_tribute", "flat_aggregation_n2", "emit_mint"] {
        let a = synthesize(&fixture(case, 10)?)?;
        let b = synthesize(&fixture(case, 11)?)?;
        if a.to_matrices()? != b.to_matrices()? {
            return Err(format!("{case}: witness-dependent matrix topology").into());
        }
    }
    Ok(())
}
