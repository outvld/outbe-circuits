//! Experimental R1CS port of frozen Emit Mint, including its pinned bignum relation.

use ark_bn254::Fr;
use ark_r1cs_std::{fields::fp::FpVar, prelude::*};
use ark_relations::gr1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use outbe_zk_canonical::{emit_mint::hash, noir::emit_mint};

use crate::poseidon2::{poseidon2_hash2, poseidon2_hash3};

#[derive(Clone)]
pub struct EmitCircuit {
    pub witness: emit_mint::Witness,
    pub public: emit_mint::PublicInputs,
}

const RADIX: u128 = 1u128 << 120;

fn range_amount(amount: &[FpVar<Fr>; 3]) -> Result<(), SynthesisError> {
    for (limb, bits) in amount.iter().zip([120, 120, 17]) {
        let (_bits, _zero_rest) = limb.to_bits_le_with_top_bits_zero(bits)?;
    }
    Ok(())
}

fn allocate_borrows(
    cs: ConstraintSystemRef<Fr>,
    lhs: [u128; 3],
    rhs: [u128; 3],
) -> Result<[Boolean<Fr>; 2], SynthesisError> {
    Ok([
        Boolean::new_witness(cs.clone(), || Ok(lhs[0] < rhs[0]))?,
        Boolean::new_witness(cs, || {
            let (subtrahend, overflow) = rhs[1].overflowing_add(u128::from(lhs[0] < rhs[0]));
            Ok(overflow || lhs[1] < subtrahend)
        })?,
    ])
}

/// noir-bignum v0.10.0 constrained_ops.nr::compute_sub_result for U256:
/// result = lhs - rhs + underflow * [0, 0, 65536], with Boolean carry/borrow
/// flags and [120,120,17]-bit outputs. The intermediate 128-bit checks are
/// implied by these tighter output checks. No extra <2^256 restriction exists.
fn sub_result(
    lhs: &[FpVar<Fr>; 3],
    rhs: &[FpVar<Fr>; 3],
    carry: [Boolean<Fr>; 2],
    borrow: [Boolean<Fr>; 2],
    underflow: Boolean<Fr>,
) -> Result<[FpVar<Fr>; 3], SynthesisError> {
    let radix = Fr::from(RADIX);
    let carry = carry.map(FpVar::from);
    let borrow = borrow.map(FpVar::from);
    let result = [
        &lhs[0] - &rhs[0] + &borrow[0] * radix - &carry[0] * radix,
        &lhs[1] - &rhs[1] + &borrow[1] * radix - &carry[1] * radix - &borrow[0] + &carry[0],
        &lhs[2] - &rhs[2] + FpVar::from(underflow) * Fr::from(65536u64) - &borrow[1] + &carry[1],
    ];
    range_amount(&result)?;
    Ok(result)
}

fn enforce_amounts(
    cs: ConstraintSystemRef<Fr>,
    note: &[FpVar<Fr>; 3],
    mint: &[FpVar<Fr>; 3],
    note_assignment: [u128; 3],
    mint_assignment: [u128; 3],
) -> Result<[FpVar<Fr>; 3], SynthesisError> {
    range_amount(note)?;
    range_amount(mint)?;
    // Limb sums cannot wrap Fr under these bounds, exactly as in Noir.
    (&mint[0] + &mint[1] + &mint[2]).enforce_not_equal(&FpVar::zero())?;

    // Integer ordering, independent of modular subtraction: note - mint must
    // admit a nonnegative, canonical borrow-chain result.
    let _ordered_difference = sub_result(
        note,
        mint,
        [Boolean::FALSE; 2],
        allocate_borrows(cs.clone(), note_assignment, mint_assignment)?,
        Boolean::FALSE,
    )?;

    // Preserve the *frozen relation*, not a strengthened integer-subtraction
    // replacement. Upstream constrained_ops.nr::sub documents conditional
    // soundness: its underflow witness is NOT constrained to the comparison.
    // Honest compute_sub_flags assigns false when note >= mint, but a forged
    // true flag can add 2^256 if the result still fits the 257-bit limb range.
    // Carry/borrow flags likewise remain separate Boolean witnesses.
    let carry = [
        Boolean::new_witness(cs.clone(), || Ok(false))?,
        Boolean::new_witness(cs.clone(), || Ok(false))?,
    ];
    let borrow = allocate_borrows(cs.clone(), note_assignment, mint_assignment)?;
    let underflow = Boolean::new_witness(cs, || {
        Ok(note_assignment
            .iter()
            .rev()
            .cmp(mint_assignment.iter().rev())
            .is_lt())
    })?;
    sub_result(note, mint, carry, borrow, underflow)
}

fn hash_multi(tag: Fr, values: &[FpVar<Fr>]) -> Result<FpVar<Fr>, SynthesisError> {
    let mut state = poseidon2_hash2([
        FpVar::Constant(tag),
        FpVar::Constant(Fr::from(values.len() as u64)),
    ])?;
    for value in values {
        state = poseidon2_hash2([state, value.clone()])?;
    }
    Ok(state)
}

impl ConstraintSynthesizer<Fr> for EmitCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        // Allocate all eight public fields first, in exact Noir ABI order.
        let chain = FpVar::new_input(cs.clone(), || Ok(Fr::from(self.public.chain_id)))?;
        let root = FpVar::new_input(cs.clone(), || Ok(self.public.root))?;
        let nullifier = FpVar::new_input(cs.clone(), || Ok(self.public.nullifier))?;
        let owner = FpVar::new_input(cs.clone(), || Ok(self.public.note_owner))?;
        let mint = [
            FpVar::new_input(cs.clone(), || Ok(Fr::from(self.public.mint_units[0])))?,
            FpVar::new_input(cs.clone(), || Ok(Fr::from(self.public.mint_units[1])))?,
            FpVar::new_input(cs.clone(), || Ok(Fr::from(self.public.mint_units[2])))?,
        ];
        let change = FpVar::new_input(cs.clone(), || Ok(self.public.change_commitment))?;
        let (_chain_bits, _zero_rest) = chain.to_bits_le_with_top_bits_zero(64)?;
        let (_owner_bits, _zero_rest) = owner.to_bits_le_with_top_bits_zero(160)?;

        let note = [
            FpVar::new_witness(cs.clone(), || Ok(Fr::from(self.witness.note_amount[0])))?,
            FpVar::new_witness(cs.clone(), || Ok(Fr::from(self.witness.note_amount[1])))?,
            FpVar::new_witness(cs.clone(), || Ok(Fr::from(self.witness.note_amount[2])))?,
        ];
        let key = FpVar::new_witness(cs.clone(), || Ok(self.witness.note_spend_key))?;
        key.enforce_not_equal(&FpVar::zero())?;
        nullifier.enforce_not_equal(&FpVar::zero())?;
        let remaining = enforce_amounts(
            cs.clone(),
            &note,
            &mint,
            self.witness.note_amount,
            self.public.mint_units,
        )?;

        // These tags are witness-independent native constants; all hashes
        // involving witness/public data use the constrained iterated hash-2.
        let serial_tag = hash::tag_note_sn().map_err(|_| SynthesisError::Unsatisfiable)?;
        let commitment_tag = hash::tag_commitment().map_err(|_| SynthesisError::Unsatisfiable)?;
        let nullifier_tag = hash::tag_nullifier().map_err(|_| SynthesisError::Unsatisfiable)?;
        let change_tag = hash::tag_change_key().map_err(|_| SynthesisError::Unsatisfiable)?;
        let serial = hash_multi(serial_tag, &[owner.clone(), key.clone()])?;
        let commitment = hash_multi(
            commitment_tag,
            &[
                chain.clone(),
                serial.clone(),
                note[0].clone(),
                note[1].clone(),
                note[2].clone(),
            ],
        )?;
        serial.enforce_not_equal(&FpVar::zero())?;
        commitment.enforce_not_equal(&FpVar::zero())?;
        let computed_nullifier = hash_multi(nullifier_tag, &[commitment.clone(), key.clone()])?;
        computed_nullifier.enforce_equal(&nullifier)?;

        let index = FpVar::new_witness(cs.clone(), || Ok(Fr::from(self.witness.leaf_index)))?;
        let (index_bits, _) = index.to_bits_le_with_top_bits_zero(32)?;
        let domain = FpVar::Constant(hash::emit_domain());
        let mut node = commitment;
        for (bit, sibling) in index_bits.iter().zip(self.witness.auth_path) {
            let sibling = FpVar::new_witness(cs.clone(), || Ok(sibling))?;
            // A set leaf-index bit puts the current node on the right.
            let left = bit.select(&sibling, &node)?;
            let right = bit.select(&node, &sibling)?;
            node = poseidon2_hash3([domain.clone(), left, right])?;
        }
        node.enforce_equal(&root)?;

        let creates_change = !(&remaining[0] + &remaining[1] + &remaining[2]).is_zero()?;
        (!change.is_zero()?).enforce_equal(&creates_change)?;
        // Always build this branch; only its assertions are conditionally
        // enforced. Full/partial mint witnesses therefore have identical shape.
        let next_key = hash_multi(change_tag, &[key, computed_nullifier])?;
        let next_serial = hash_multi(serial_tag, &[owner, next_key.clone()])?;
        let next_commitment = hash_multi(
            commitment_tag,
            &[
                chain,
                next_serial.clone(),
                remaining[0].clone(),
                remaining[1].clone(),
                remaining[2].clone(),
            ],
        )?;
        let next_nullifier =
            hash_multi(nullifier_tag, &[next_commitment.clone(), next_key.clone()])?;
        for value in [&next_key, &next_serial, &next_commitment, &next_nullifier] {
            value.conditional_enforce_not_equal(&FpVar::zero(), &creates_change)?;
        }
        next_commitment.conditional_enforce_equal(&change, &creates_change)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_ff::{AdditiveGroup, Field};
    use ark_relations::gr1cs::ConstraintSystem;

    fn arithmetic(note: [u128; 3], mint: [u128; 3]) -> (bool, [Fr; 3]) {
        let cs = ConstraintSystem::<Fr>::new_ref();
        let note_vars = note.map(|v| FpVar::new_witness(cs.clone(), || Ok(Fr::from(v))).unwrap());
        let mint_vars = mint.map(|v| FpVar::new_input(cs.clone(), || Ok(Fr::from(v))).unwrap());
        let remaining = enforce_amounts(cs.clone(), &note_vars, &mint_vars, note, mint).unwrap();
        (
            cs.is_satisfied().unwrap(),
            remaining.map(|v| v.value().unwrap()),
        )
    }

    #[test]
    fn integer_equality_partial_and_cross_limb_borrow() {
        for value in [
            [1, 0, 0],
            [0, 1, 0],
            [0, 0, 1],
            [RADIX - 1, RADIX - 1, 131071],
        ] {
            let (valid, remaining) = arithmetic(value, value);
            assert!(valid);
            assert_eq!(remaining, [Fr::ZERO; 3]);
        }
        let (valid, remaining) = arithmetic([5, 7, 2], [9, 8, 1]);
        assert!(valid);
        assert_eq!(remaining, [RADIX - 4, RADIX - 2, 0].map(Fr::from));
        let (valid, remaining) = arithmetic([0, 0, 65536], [1, 0, 0]);
        assert!(valid);
        assert_eq!(remaining, [RADIX - 1, RADIX - 1, 65535].map(Fr::from));
        assert!(arithmetic([5, 7, 131071], [9, 8, 1]).0);
    }

    #[test]
    fn rejects_zero_overspend_and_out_of_range_limbs() {
        for (note, mint) in [
            ([1, 0, 0], [0, 0, 0]),
            ([0, 0, 0], [1, 0, 0]),
            ([1, 0, 0], [2, 0, 0]),
            ([RADIX - 1, 0, 0], [0, 1, 0]),
            ([0, 0, 1], [0, 0, 2]),
            ([RADIX, 1, 0], [1, 0, 0]),
            ([1, RADIX, 0], [1, 0, 0]),
            ([1, 0, 131072], [1, 0, 0]),
            ([1, 0, 1], [RADIX, 0, 0]),
            ([1, 0, 1], [0, RADIX, 0]),
            ([1, 0, 131072], [1, 0, 131072]),
        ] {
            assert!(!arithmetic(note, mint).0, "note={note:?}, mint={mint:?}");
        }
    }

    #[test]
    fn frozen_subtraction_retains_unbound_underflow_flag() {
        let cs = ConstraintSystem::<Fr>::new_ref();
        let lhs =
            [5u128, 0, 0].map(|v| FpVar::new_witness(cs.clone(), || Ok(Fr::from(v))).unwrap());
        let rhs =
            [1u128, 0, 0].map(|v| FpVar::new_witness(cs.clone(), || Ok(Fr::from(v))).unwrap());
        // This intentionally records the frozen library's allowed relation,
        // not the honest assignment generated by compute_sub_flags.
        let underflow = Boolean::new_witness(cs.clone(), || Ok(true)).unwrap();
        let remaining = sub_result(
            &lhs,
            &rhs,
            [Boolean::FALSE; 2],
            [Boolean::FALSE; 2],
            underflow,
        )
        .unwrap();
        assert_eq!(
            remaining.map(|v| v.value().unwrap()),
            [4u128, 0, 65536].map(Fr::from)
        );
        assert!(cs.is_satisfied().unwrap());
    }

    fn fixture() -> EmitCircuit {
        match crate::noir::fixture("emit_mint", 0).unwrap() {
            crate::noir::Fixture::Emit(c) => c,
            _ => panic!("expected Emit fixture"),
        }
    }

    fn satisfied(circuit: EmitCircuit) -> bool {
        let cs = ConstraintSystem::<Fr>::new_ref();
        match circuit.generate_constraints(cs.clone()) {
            Ok(()) => cs.is_satisfied().unwrap(),
            Err(SynthesisError::Unsatisfiable) => false,
            Err(error) => panic!("unexpected synthesis failure: {error:?}"),
        }
    }

    #[test]
    fn emit_full_partial_and_frozen_257_bit_boundaries() {
        use outbe_protocol::protocol::zk::Circuit;

        for (note, mint) in [
            ([1, 0, 0], [1, 0, 0]),
            ([0, 0, 65536], [0, 0, 65536]),
            (
                [RADIX - 1, RADIX - 1, 131071],
                [RADIX - 1, RADIX - 1, 131071],
            ),
            ([0, 0, 65536], [1, 0, 0]),
            ([5, 7, 131071], [9, 8, 1]),
        ] {
            let c = crate::noir::emit_fixture_with_amounts(note, mint, 1).unwrap();
            let expected = emit_mint::EmitMint::public_inputs(&c.public);
            assert_eq!(c.public.change_commitment == Fr::ZERO, note == mint);
            let cs = ConstraintSystem::<Fr>::new_ref();
            c.clone().generate_constraints(cs.clone()).unwrap();
            assert!(cs.is_satisfied().unwrap());
            assert_eq!(
                &cs.borrow().unwrap().instance_assignment().unwrap()[1..],
                &expected
            );
            if note == mint {
                let mut bad = c;
                bad.public.change_commitment = Fr::ONE;
                assert!(!satisfied(bad));
            }
        }
        let invalid =
            crate::noir::emit_fixture_with_amounts([0, 0, 131072], [0, 0, 131072], 1).unwrap();
        assert!(!satisfied(invalid));
    }

    #[test]
    fn emit_fixture_and_statement_mutations() {
        let c = fixture();
        assert!(satisfied(c.clone()));
        let mut bad = c.clone();
        bad.public.change_commitment = Fr::ZERO;
        assert!(!satisfied(bad));
        let mut bad = c.clone();
        bad.public.nullifier += Fr::ONE;
        assert!(!satisfied(bad));
        let mut bad = c.clone();
        bad.witness.note_spend_key = Fr::ZERO;
        assert!(!satisfied(bad));
        let mut bad = c.clone();
        bad.witness.auth_path[31] += Fr::ONE;
        assert!(!satisfied(bad));
        let mut bad = c.clone();
        bad.witness.leaf_index ^= 1;
        assert!(!satisfied(bad));
        let mut bad = c.clone();
        bad.public.note_owner = Fr::from(2u64).pow([160]);
        assert!(!satisfied(bad));
        let mut bad = c;
        bad.public.mint_units = [0; 3];
        assert!(!satisfied(bad));
    }
}
