// SPDX-License-Identifier: Apache-2.0
// Copyright 2022 Aztec. Modified: direct gnark constraint implementation.
// Gemini, Shplonk, Libra and KZG equations from the pinned Aztec verifier.
package main

import (
	"math/big"

	"github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
	"github.com/consensys/gnark/std/math/emulated"
)

func (v *verifier) scalar(x V) *emulated.Element[emulated.BN254Fr] {
	return v.fr.FromBits(v.ToBinary(x)...)
}

func (v *verifier) shplemini(p *HonkProof, t *transcript) error {
	a, m, s := v.Add, v.Mul, v.Sub
	var powers [logN]V
	powers[0] = t.r
	for i := 1; i < logN; i++ {
		powers[i] = m(powers[i-1], powers[i-1])
	}
	pos, neg := v.Inverse(s(t.z, t.r)), v.Inverse(a(t.z, t.r))
	unshifted := v.Neg(a(pos, m(t.nu, neg)))
	shifted := v.Neg(m(v.Inverse(t.r), s(pos, m(t.nu, neg))))
	var scalars [58]V // 37 unshifted + logN + 3 Libra + 2
	commitments := make([]*Point, 0, len(scalars))
	commitments = append(commitments, &p.Shplonk, &p.Mask)
	for _, point := range fixedVK() {
		q := point
		commitments = append(commitments, &q)
	}
	commitments = append(commitments, &p.W1, &p.W2, &p.W3, &p.W4, &p.ZPerm, &p.Inverses, &p.ReadCounts, &p.ReadTags)
	scalars[0] = 1
	batch, batchedEval := V(1), V(0)
	for i := 1; i <= 37; i++ {
		scalars[i] = m(unshifted, batch)
		batchedEval = a(batchedEval, m(p.Evaluations[i-1], batch))
		batch = m(batch, t.rho)
	}
	for i := range 5 {
		scalars[30+i] = a(scalars[30+i], m(shifted, batch))
		batchedEval = a(batchedEval, m(p.Evaluations[37+i], batch))
		batch = m(batch, t.rho)
	}
	var foldPos [logN]V
	for i := logN - 1; i >= 0; i-- {
		r, u := powers[i], t.u[i]
		batchedEval = m(s(m(r, batchedEval, 2), m(p.Gemini[i], s(m(r, s(1, u)), u))), v.Inverse(a(m(r, s(1, u)), u)))
		foldPos[i] = batchedEval
	}
	constant := a(m(foldPos[0], pos), m(p.Gemini[0], t.nu, neg))
	batch = m(t.nu, t.nu)
	boundary := 38
	for i := range logN - 1 {
		pos, neg = v.Inverse(s(t.z, powers[i+1])), v.Inverse(a(t.z, powers[i+1]))
		factorPos, factorNeg := m(batch, pos), m(batch, t.nu, neg)
		scalars[boundary+i] = v.Neg(a(factorPos, factorNeg))
		constant = a(constant, m(factorNeg, p.Gemini[i+1]), m(factorPos, foldPos[i+1]))
		batch = m(batch, t.nu, t.nu)
		commitments = append(commitments, &p.Folds[i])
	}
	boundary += logN - 1
	generator := integer("0x07b0c561a6148404f086204a9f36ffb0617942546750f230c893619174a57a76")
	den := v.Inverse(s(t.z, t.r))
	denShifted := v.Inverse(s(t.z, m(generator, t.r)))
	var libraScalars [4]V
	for i := range 4 {
		d := den
		if i == 1 {
			d = denShifted
		}
		factor := m(d, batch)
		libraScalars[i] = v.Neg(factor)
		batch = m(batch, t.nu)
		constant = a(constant, m(factor, p.LibraEvals[i]))
	}
	scalars[boundary] = libraScalars[0]
	scalars[boundary+1] = a(libraScalars[1], libraScalars[2])
	scalars[boundary+2] = libraScalars[3]
	for i := range p.Libra {
		commitments = append(commitments, &p.Libra[i])
	}
	boundary += 3
	base := constantPoint("1", "2")
	commitments = append(commitments, &base, &p.Quotient)
	scalars[boundary] = constant
	scalars[boundary+1] = t.z
	v.libraConsistency(p, t)
	zero := constantPoint("0", "0")
	lhs := &zero
	for i, point := range commitments {
		term := v.curve.ScalarMul(point, v.scalar(scalars[i]))
		lhs = v.curve.AddUnified(lhs, term)
	}
	rhs := v.curve.Neg(&p.Quotient)
	// Preserve the generated verifier's complete recursive-accumulator handling.
	// All eight limbs are authenticated by both the transcript and permutation.
	others := [2]Point{}
	allZero := V(1)
	for _, limb := range p.Pairing {
		allZero = v.And(allZero, v.IsZero(limb))
	}
	for i := range 2 {
		coords := [2]*emulated.Element[emulated.BN254Fp]{}
		for j := range 2 {
			idx := 4*i + 2*j
			bits := append(v.ToBinary(p.Pairing[idx], 136), v.ToBinary(p.Pairing[idx+1], 120)...)
			coords[j] = v.fp.FromBits(bits...)
			v.fp.AssertIsInRange(coords[j])
		}
		others[i] = Point{X: *coords[0], Y: *coords[1]}
		v.curve.AssertIsOnCurve(&others[i])
		infinity := v.And(v.fp.IsZero(coords[0]), v.fp.IsZero(coords[1]))
		v.AssertIsEqual(m(s(1, allZero), infinity), 0)
	}
	separator := v.hash(v.pointWords(&others[0]), v.pointWords(&others[1]), v.pointWords(lhs), v.pointWords(rhs))
	sep := v.scalar(separator)
	aggregatedLHS := v.curve.AddUnified(v.curve.ScalarMul(lhs, sep), &others[0])
	aggregatedRHS := v.curve.AddUnified(v.curve.ScalarMul(rhs, sep), &others[1])
	lhs = v.curve.Select(allZero, lhs, aggregatedLHS)
	rhs = v.curve.Select(allZero, rhs, aggregatedRHS)
	pairing, err := sw_bn254.NewPairing(v.API)
	if err != nil {
		return err
	}
	// Fixed G2 generator and exact BB ceremony [tau]2, not a new KZG SRS.
	g2 := fixedG2(
		"0x1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed",
		"0x198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2",
		"0x12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa",
		"0x090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b")
	tau := fixedG2(
		"0x0118c4d5b837bcc2bc89b5b398b5974e9f5944073b32078b7e231fec938883b0",
		"0x260e01b251f6f1c7e7ff4e580791dee8ea51d87a358e038b4efe30fac09383c1",
		"0x22febda3c0c0632a56475b4214e5615e11e6dd3f96e6cea2854a87d4dacc5e55",
		"0x04fc6369f7110fe3d25156c1bb9a72859cf2a04641f99ba4ee413c80da6a5fe4")
	pairing.AssertIsOnG1(lhs)
	pairing.AssertIsOnG1(rhs)
	return pairing.PairingCheck([]*sw_bn254.G1Affine{lhs, rhs}, []*sw_bn254.G2Affine{&g2, &tau})
}

func fixedG2(x0, x1, y0, y1 string) sw_bn254.G2Affine {
	var p bn254.G2Affine
	p.X.A0.SetBigInt(integer(x0))
	p.X.A1.SetBigInt(integer(x1))
	p.Y.A0.SetBigInt(integer(y0))
	p.Y.A1.SetBigInt(integer(y1))
	// Coordinates and line precomputations are compile-time constants, never witness input.
	return sw_bn254.NewG2AffineFixed(p)
}

func (v *verifier) libraConsistency(p *HonkProof, t *transcript) {
	a, m, s := v.Add, v.Mul, v.Sub
	r256 := t.r
	for range 8 {
		r256 = m(r256, r256)
	}
	vanishing := s(r256, 1)
	v.AssertIsDifferent(vanishing, 0)
	var challengePoly [256]V
	for i := range challengePoly {
		challengePoly[i] = 0
	}
	challengePoly[0] = 1
	for round := range logN {
		idx := 1 + 9*round
		challengePoly[idx] = 1
		for j := 1; j < 9; j++ {
			challengePoly[idx+j] = m(challengePoly[idx+j-1], t.u[round])
		}
	}
	inverseGenerator := integer("0x204bd3277422fad364751ad938e2b5e6a54cf8c68712848a692c553d0329f5d6")
	rootPower := integer("1")
	challengeEval, first, last := V(0), V(0), V(0)
	modulus := new(emulated.BN254Fr).Modulus()
	for i := range 256 {
		den := v.Inverse(s(m(new(big.Int).Set(rootPower), t.r), 1))
		challengeEval = a(challengeEval, m(challengePoly[i], den))
		if i == 0 {
			first = den
		}
		if i == 255 {
			last = den
		}
		rootPower.Mul(rootPower, inverseGenerator).Mod(rootPower, modulus)
	}
	numerator := m(vanishing, v.Inverse(256))
	challengeEval, first, last = m(challengeEval, numerator), m(first, numerator), m(last, numerator)
	diff := a(m(first, p.LibraEvals[2]), m(s(t.r, inverseGenerator), s(s(p.LibraEvals[1], p.LibraEvals[2]), m(p.LibraEvals[0], challengeEval))), m(last, s(p.LibraEvals[2], p.LibraEvaluation)))
	v.AssertIsEqual(s(diff, m(vanishing, p.LibraEvals[3])), 0)
}
