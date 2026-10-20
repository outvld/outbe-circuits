//! BN254 Poseidon2 hashes, matching Noir's rate-3 sponge framing.

use ark_bn254::Fr;
use ark_r1cs_std::fields::{fp::FpVar, FieldVar};
use ark_relations::gr1cs::SynthesisError;

use crate::poseidon2_constants::BN254_CONFIG;

fn fifth_power(x: &FpVar<Fr>) -> Result<FpVar<Fr>, SynthesisError> {
    let square = x.square()?;
    Ok(square.square()? * x)
}

// Same external matrix as barretenberg and outbe-poseidon, expressed as linear
// combinations so mixing introduces no multiplication constraints.
fn external_layer(state: &mut [FpVar<Fr>; 4]) {
    let t0 = &state[0] + &state[1];
    let t1 = &state[2] + &state[3];
    let t2 = &state[1] + &state[1] + &t1;
    let t3 = &state[3] + &state[3] + &t0;
    let t4 = &t1 + &t1;
    let t4 = &t4 + &t4 + &t3;
    let t5 = &t0 + &t0;
    let t5 = &t5 + &t5 + &t2;
    *state = [&t3 + &t5, t5, &t2 + &t4, t4];
}

fn permutation(state: &mut [FpVar<Fr>; 4]) -> Result<(), SynthesisError> {
    let config = &BN254_CONFIG;
    external_layer(state);
    let half_full = config.rounds_f as usize / 2;
    let partial_end = half_full + config.rounds_p as usize;
    for round in 0..(config.rounds_f + config.rounds_p) as usize {
        if (half_full..partial_end).contains(&round) {
            state[0] += config.round_constant[round][0];
            state[0] = fifth_power(&state[0])?;
            let sum = &state[0] + &state[1] + &state[2] + &state[3];
            for (value, diagonal) in state.iter_mut().zip(config.internal_matrix_diagonal) {
                *value = &*value * diagonal + &sum;
            }
        } else {
            for (value, constant) in state.iter_mut().zip(config.round_constant[round]) {
                *value += constant;
                *value = fifth_power(value)?;
            }
            external_layer(state);
        }
    }
    Ok(())
}

/// Absorb two fields into the rate-3 state, with capacity `2 << 64`.
/// Unlike hash-3, the partial block needs only the final squeeze permutation.
pub fn poseidon2_hash2(inputs: [FpVar<Fr>; 2]) -> Result<FpVar<Fr>, SynthesisError> {
    let [a, b] = inputs;
    let mut state = [a, b, FpVar::zero(), FpVar::Constant(Fr::from(2u128 << 64))];
    permutation(&mut state)?;
    Ok(state[0].clone())
}

/// Absorb exactly three fields, then perform the separate final squeeze
/// permutation. The capacity starts at `3 << 64`, not zero.
pub fn poseidon2_hash3(inputs: [FpVar<Fr>; 3]) -> Result<FpVar<Fr>, SynthesisError> {
    let [a, b, c] = inputs;
    let mut state = [a, b, c, FpVar::Constant(Fr::from(3u128 << 64))];
    permutation(&mut state)?;
    permutation(&mut state)?;
    Ok(state[0].clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_ff::{AdditiveGroup, Field, MontFp, UniformRand};
    use ark_r1cs_std::{alloc::AllocVar, eq::EqGadget, GR1CSVar};
    use ark_relations::gr1cs::ConstraintSystem;
    use ark_std::rand::{rngs::StdRng, SeedableRng};
    use outbe_poseidon::{Poseidon2, PoseidonHasher};

    fn assert_hash(inputs: [Fr; 3], expected: Fr) {
        let cs = ConstraintSystem::<Fr>::new_ref();
        let allocated = inputs.map(|value| FpVar::new_witness(cs.clone(), || Ok(value)).unwrap());
        let actual = poseidon2_hash3(allocated).unwrap();
        let expected_var = FpVar::new_input(cs.clone(), || Ok(expected)).unwrap();
        actual.enforce_equal(&expected_var).unwrap();
        assert_eq!(actual.value().unwrap(), expected);
        assert!(cs.is_satisfied().unwrap());
    }

    #[test]
    fn known_answer() {
        assert_hash(
            [Fr::from(1u64), Fr::from(2u64), Fr::from(3u64)],
            MontFp!("0x16f5da1a6b40e7d71bcdf29687e7908cdf74da44c09058fe36a0a99e269c6972"),
        );
    }

    #[test]
    fn native_parity() {
        let mut native = Poseidon2::<Fr>::new();
        let mut check = |inputs| assert_hash(inputs, native.hash(&inputs).unwrap());
        check([Fr::ZERO; 3]);
        check([-Fr::ONE; 3]);
        check([-Fr::ONE, -Fr::from(2u64), Fr::ZERO]);
        let mut rng = StdRng::seed_from_u64(0x706f736569646f6e);
        for _ in 0..8 {
            check(core::array::from_fn(|_| Fr::rand(&mut rng)));
        }
    }

    #[test]
    fn hash2_native_allocated_parity() {
        let mut native = Poseidon2::<Fr>::new();
        let mut rng = StdRng::seed_from_u64(0x706f736569646f32);
        let mut inputs = vec![
            [Fr::ZERO; 2],
            [-Fr::ONE; 2],
            [-Fr::ONE, Fr::ZERO],
            [Fr::from(1u64), Fr::from(2u64)],
        ];
        inputs.extend((0..8).map(|_| core::array::from_fn(|_| Fr::rand(&mut rng))));
        for input in inputs {
            let cs = ConstraintSystem::<Fr>::new_ref();
            let allocated = input.map(|v| FpVar::new_witness(cs.clone(), || Ok(v)).unwrap());
            let output = poseidon2_hash2(allocated).unwrap();
            let expected = native.hash(&input).unwrap();
            let public = FpVar::new_input(cs.clone(), || Ok(expected)).unwrap();
            output.enforce_equal(&public).unwrap();
            assert_eq!(output.value().unwrap(), expected);
            assert!(cs.is_satisfied().unwrap());
        }
    }

    #[test]
    fn hash2_known_answer_and_wrong_output() {
        let cs = ConstraintSystem::<Fr>::new_ref();
        let input = [Fr::from(1u64), Fr::from(2u64)]
            .map(|v| FpVar::new_witness(cs.clone(), || Ok(v)).unwrap());
        let output = poseidon2_hash2(input).unwrap();
        output
            .enforce_equal(&FpVar::Constant(MontFp!(
                "0x038682aa1cb5ae4e0a3f13da432a95c77c5c111f6f030faf9cad641ce1ed7383"
            )))
            .unwrap();
        assert!(cs.is_satisfied().unwrap());
        output.enforce_equal(&FpVar::zero()).unwrap();
        assert!(!cs.is_satisfied().unwrap());
    }

    #[test]
    fn wrong_output_is_unsatisfied() {
        let cs = ConstraintSystem::<Fr>::new_ref();
        let allocated = [Fr::from(1u64), Fr::from(2u64), Fr::from(3u64)]
            .map(|value| FpVar::new_witness(cs.clone(), || Ok(value)).unwrap());
        let output = poseidon2_hash3(allocated).unwrap();
        output.enforce_equal(&FpVar::Constant(Fr::ZERO)).unwrap();
        assert!(!cs.is_satisfied().unwrap());
    }
}
