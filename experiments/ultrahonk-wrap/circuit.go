// SPDX-License-Identifier: Apache-2.0
// Copyright 2022 Aztec. Modified: direct gnark constraint implementation.
// UltraKeccakZK verifier port of Aztec's generated Solidity verifier.
// Reference SHA-256: c80d9f2673861398d778493895b34c0a868b9649fc99bb8aed3abb968c66d8be.
// BB 5.0.0-nightly.20260522, Noir 1.0.0-beta.22, Demo Tribute only.
package main

import (
	"fmt"
	"math/big"

	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/algebra/emulated/sw_emulated"
	"github.com/consensys/gnark/std/hash/sha3"
	"github.com/consensys/gnark/std/math/emulated"
	"github.com/consensys/gnark/std/math/uints"
)

const (
	logN             = 16
	proofBytes       = 8768
	innerVKHash      = "6b5d5609be0894c5f8379803ab6421bf66b0f1c25c5667644772f2d2bf7225bc"
	referenceHash    = "c80d9f2673861398d778493895b34c0a868b9649fc99bb8aed3abb968c66d8be"
	vkTranscriptHash = "0x1eb9dad25f55ccf93008a99074a7c04747b940cb0fd3553a9cb5c317f3921e20"
)

type V = frontend.Variable
type Point = sw_emulated.AffinePoint[emulated.BN254Fp]

type HonkProof struct {
	Pairing                                   [8]V
	Mask, W1, W2, W3                          Point
	ReadCounts, ReadTags, W4, Inverses, ZPerm Point
	Libra                                     [3]Point
	LibraSum                                  V
	Univariates                               [logN][9]V
	Evaluations                               [42]V
	LibraEvaluation                           V
	Folds                                     [logN - 1]Point
	Gemini                                    [logN]V
	LibraEvals                                [4]V
	Shplonk, Quotient                         Point
}

// The four original Fr words are the ONLY public inputs of the outer statement.
// In particular, neither the VK nor a native verification result is a witness.
type Circuit struct {
	Public [4]V `gnark:",public"`
	Proof  HonkProof
}

type verifier struct {
	frontend.API
	fp    *emulated.Field[emulated.BN254Fp]
	fr    *emulated.Field[emulated.BN254Fr]
	curve *sw_emulated.Curve[emulated.BN254Fp, emulated.BN254Fr]
	bytes *uints.Bytes
}

type transcript struct {
	eta, beta, gamma, delta V
	alpha                   [28]V
	gate, u                 [logN]V
	libra, rho, r, nu, z    V
}

func integer(s string) *big.Int {
	x, ok := new(big.Int).SetString(s, 0)
	if !ok {
		panic("invalid source constant: " + s)
	}
	return x
}

func (c *Circuit) Define(api frontend.API) error {
	if api.Compiler().Field().Cmp(new(emulated.BN254Fr).Modulus()) != 0 {
		return fmt.Errorf("outer circuit must use BN254 Fr")
	}
	fp, err := emulated.NewField[emulated.BN254Fp](api)
	if err != nil {
		return err
	}
	fr, err := emulated.NewField[emulated.BN254Fr](api)
	if err != nil {
		return err
	}
	curve, err := sw_emulated.New[emulated.BN254Fp, emulated.BN254Fr](api, sw_emulated.GetBN254Params())
	if err != nil {
		return err
	}
	b, err := uints.NewBytes(api)
	if err != nil {
		return err
	}
	v := &verifier{api, fp, fr, curve, b}
	// Constrain every supplied coordinate, including commitments with zero scalars.
	for _, p := range c.Proof.points() {
		fp.AssertIsInRange(&p.X)
		fp.AssertIsInRange(&p.Y)
		curve.AssertIsOnCurve(p) // includes the canonical EVM infinity (0,0).
	}
	for i, limb := range c.Proof.Pairing {
		width := 136
		if i%2 == 1 {
			width = 120
		}
		api.ToBinary(limb, width)
	}
	t := v.makeTranscript(&c.Proof, c.Public)
	v.sumcheck(&c.Proof, &t)
	return v.shplemini(&c.Proof, &t)
}

func (p *HonkProof) points() []*Point {
	out := []*Point{&p.Mask, &p.W1, &p.W2, &p.W3, &p.ReadCounts, &p.ReadTags, &p.W4, &p.Inverses, &p.ZPerm, &p.Libra[0], &p.Libra[1], &p.Libra[2], &p.Shplonk, &p.Quotient}
	for i := range p.Folds {
		out = append(out, &p.Folds[i])
	}
	return out
}

// Native Fr -> 32-byte big-endian canonical encoding. ToBinary uses the
// compiler's modulus check; a witness cannot choose x+Fr as its transcript word.
func (v *verifier) word(x V) []uints.U8 {
	bits := v.ToBinary(x)
	bits = append(bits, 0, 0)
	out := make([]uints.U8, 32)
	for i := range out {
		out[31-i] = v.bytes.FromBits(bits[8*i : 8*i+8]...)
	}
	return out
}

func (v *verifier) pointWords(p *Point) []uints.U8 {
	out := make([]uints.U8, 0, 64)
	for _, coord := range []*emulated.Element[emulated.BN254Fp]{&p.X, &p.Y} {
		bits := v.fp.ToBitsCanonical(coord)
		bits = append(bits, 0, 0)
		for i := 31; i >= 0; i-- {
			out = append(out, v.bytes.FromBits(bits[8*i:8*i+8]...))
		}
	}
	return out
}

// The digest is constrained bit-for-bit by LegacyKeccak256. Recomposition in
// native Fr implements exactly uint256(digest) % Fr, NOT a truncated digest.
func (v *verifier) hash(parts ...[]uints.U8) V {
	h, err := sha3.NewLegacyKeccak256(v.API)
	if err != nil {
		panic(err)
	}
	for _, part := range parts {
		h.Write(part)
	}
	digest := h.Sum()
	bits := make([]V, 0, 256)
	for i := len(digest) - 1; i >= 0; i-- {
		bits = append(bits, v.bytes.ToBits(digest[i])...)
	}
	return v.FromBinary(bits...)
}

func (v *verifier) split(x V) (V, V) {
	bits := v.ToBinary(x)
	return v.FromBinary(bits[:127]...), v.FromBinary(bits[127:]...)
}

func (v *verifier) makeTranscript(p *HonkProof, pubs [4]V) transcript {
	var t transcript
	parts := [][]uints.U8{v.word(integer(vkTranscriptHash))}
	for _, x := range pubs {
		parts = append(parts, v.word(x))
	}
	for _, x := range p.Pairing {
		parts = append(parts, v.word(x))
	}
	for _, x := range []*Point{&p.Mask, &p.W1, &p.W2, &p.W3} {
		parts = append(parts, v.pointWords(x))
	}
	prev := v.hash(parts...)
	t.eta, _ = v.split(prev)
	prev = v.hash(v.word(prev), v.pointWords(&p.ReadCounts), v.pointWords(&p.ReadTags), v.pointWords(&p.W4))
	t.beta, t.gamma = v.split(prev)
	prev = v.hash(v.word(prev), v.pointWords(&p.Inverses), v.pointWords(&p.ZPerm))
	t.alpha[0], _ = v.split(prev)
	for i := 1; i < len(t.alpha); i++ {
		t.alpha[i] = v.Mul(t.alpha[i-1], t.alpha[0])
	}
	prev = v.hash(v.word(prev))
	t.gate[0], _ = v.split(prev)
	for i := 1; i < logN; i++ {
		t.gate[i] = v.Mul(t.gate[i-1], t.gate[i-1])
	}
	prev = v.hash(v.word(prev), v.pointWords(&p.Libra[0]), v.word(p.LibraSum))
	t.libra, _ = v.split(prev)
	for i := range logN {
		parts = [][]uints.U8{v.word(prev)}
		for _, x := range p.Univariates[i] {
			parts = append(parts, v.word(x))
		}
		prev = v.hash(parts...)
		t.u[i], _ = v.split(prev)
	}
	parts = [][]uints.U8{v.word(prev)}
	for _, x := range p.Evaluations {
		parts = append(parts, v.word(x))
	}
	parts = append(parts, v.word(p.LibraEvaluation), v.pointWords(&p.Libra[1]), v.pointWords(&p.Libra[2]))
	prev = v.hash(parts...)
	t.rho, _ = v.split(prev)
	parts = [][]uints.U8{v.word(prev)}
	for i := range p.Folds {
		parts = append(parts, v.pointWords(&p.Folds[i]))
	}
	prev = v.hash(parts...)
	t.r, _ = v.split(prev)
	parts = [][]uints.U8{v.word(prev)}
	for _, x := range p.Gemini {
		parts = append(parts, v.word(x))
	}
	for _, x := range p.LibraEvals {
		parts = append(parts, v.word(x))
	}
	prev = v.hash(parts...)
	t.nu, _ = v.split(prev)
	prev = v.hash(v.word(prev), v.pointWords(&p.Shplonk))
	t.z, _ = v.split(prev)
	// Public-input delta includes 4 original public fields AND 8 pairing limbs.
	num, den := V(1), V(1)
	na := v.Add(t.gamma, v.Mul(t.beta, (1<<28)+5))
	da := v.Sub(t.gamma, v.Mul(t.beta, 6))
	all := append(append([]V{}, pubs[:]...), p.Pairing[:]...)
	for _, x := range all {
		num = v.Mul(num, v.Add(na, x))
		den = v.Mul(den, v.Add(da, x))
		na = v.Add(na, t.beta)
		da = v.Sub(da, t.beta)
	}
	t.delta = v.Mul(num, v.Inverse(den))
	return t
}

func (v *verifier) sumcheck(p *HonkProof, t *transcript) {
	target, pow := v.Mul(t.libra, p.LibraSum), V(1)
	for i := range logN {
		v.AssertIsEqual(v.Add(p.Univariates[i][0], p.Univariates[i][1]), target)
		// Exact barycentric formula: every denominator must be nonzero,
		// matching the generated verifier even at challenges in {0,...,8}.
		target = V(0)
		numerator := V(1)
		for j := range 9 {
			den := int64(1)
			for k := range 9 {
				if j != k {
					den *= int64(j - k)
				}
			}
			difference := v.Sub(t.u[i], j)
			numerator = v.Mul(numerator, difference)
			target = v.Add(target, v.Mul(p.Univariates[i][j], v.Inverse(v.Mul(den, difference))))
		}
		target = v.Mul(target, numerator)
		pow = v.Mul(pow, v.Add(1, v.Mul(t.u[i], v.Sub(t.gate[i], 1))))
	}
	var evals [41]V
	copy(evals[:], p.Evaluations[1:])
	relation := v.relations(evals, t, pow)
	disabled := V(1)
	for i := 2; i < logN; i++ {
		disabled = v.Mul(disabled, v.Sub(1, t.u[i]))
	}
	v.AssertIsEqual(v.Add(v.Mul(relation, v.Sub(1, disabled)), v.Mul(p.LibraEvaluation, t.libra)), target)
}
