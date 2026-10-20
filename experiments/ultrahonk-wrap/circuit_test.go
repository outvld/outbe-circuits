// SPDX-License-Identifier: Apache-2.0
package main

import (
	"bytes"
	"context"
	"encoding/hex"
	"encoding/json"
	"math/big"
	"os"
	"path/filepath"
	"testing"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend/groth16"
	"github.com/consensys/gnark/frontend"
)

func fixture(t *testing.T) request {
	t.Helper()
	path := os.Getenv("ULTRAHONK_WRAP_FIXTURE")
	if path == "" {
		path = "testdata/demo-input.json"
	}
	b, err := os.ReadFile(path)
	if err != nil {
		t.Fatalf("genuine pinned BB5 Demo fixture is required: %v", err)
	}
	var req request
	if err := json.Unmarshal(b, &req); err != nil {
		t.Fatal(err)
	}
	return req
}

func assignment(t *testing.T, req request) *Circuit {
	t.Helper()
	c, _, _, err := publicAssignment(req)
	if err != nil {
		t.Fatal(err)
	}
	proof, err := hexData(req.ProofHex, proofBytes)
	if err != nil {
		t.Fatal(err)
	}
	if err := assignProof(c, proof); err != nil {
		t.Fatal(err)
	}
	return c
}

func increment(x V) V {
	return new(big.Int).Mod(new(big.Int).Add(x.(*big.Int), big.NewInt(1)), ecc.BN254.ScalarField())
}

func TestExactBB5ProofConstraints(t *testing.T) {
	req := fixture(t)
	// Reuse a CLI-generated R1CS when available, instead of recompiling it.
	cs := groth16.NewCS(ecc.BN254)
	if keys := os.Getenv("ULTRAHONK_WRAP_KEYS"); keys != "" {
		p, err := loadSetup(keys)
		if err != nil {
			t.Fatal(err)
		}
		if err := load(filepath.Join(keys, "circuit.r1cs"), p.Hashes["circuit.r1cs"], cs); err != nil {
			t.Fatal(err)
		}
	} else {
		var err error
		cs, err = compile()
		if err != nil {
			t.Fatal(err)
		}
	}
	if got := shape(cs).Public; got != 4 {
		t.Fatalf("outer public arity: got %d", got)
	}
	check := func(t *testing.T, c *Circuit, valid bool) {
		t.Helper()
		w, err := frontend.NewWitness(c, ecc.BN254.ScalarField())
		if err != nil {
			t.Fatal(err)
		}
		_, err = cs.Solve(w)
		if (err == nil) != valid {
			t.Fatalf("constraint satisfaction: valid=%v error=%v", valid, err)
		}
	}
	t.Run("genuine-ZK-keccak-proof", func(t *testing.T) { check(t, assignment(t, req), true) })
	mutations := []struct {
		name   string
		mutate func(*Circuit)
	}{
		{"owner", func(c *Circuit) { c.Public[0] = increment(c.Public[0]) }},
		{"nft", func(c *Circuit) { c.Public[1] = increment(c.Public[1]) }},
		{"binding", func(c *Circuit) { c.Public[2] = increment(c.Public[2]) }},
		{"root", func(c *Circuit) { c.Public[3] = increment(c.Public[3]) }},
		{"sumcheck-round", func(c *Circuit) { c.Proof.Univariates[0][0] = increment(c.Proof.Univariates[0][0]) }},
		{"sumcheck-relation-evaluation", func(c *Circuit) { c.Proof.Evaluations[41] = increment(c.Proof.Evaluations[41]) }},
		{"libra-evaluation", func(c *Circuit) { c.Proof.LibraEvals[2] = increment(c.Proof.LibraEvals[2]) }},
		{"libra-sum", func(c *Circuit) { c.Proof.LibraSum = increment(c.Proof.LibraSum) }},
		{"gemini-evaluation", func(c *Circuit) { c.Proof.Gemini[logN-1] = increment(c.Proof.Gemini[logN-1]) }},
		{"transcript-commitment", func(c *Circuit) { c.Proof.Mask = constantPoint("1", "2") }},
		{"kzg-pairing-quotient", func(c *Circuit) { c.Proof.Quotient = constantPoint("1", "2") }},
		{"recursive-accumulator", func(c *Circuit) { c.Proof.Pairing[0] = increment(c.Proof.Pairing[0]) }},
		{"accumulator-limb-range", func(c *Circuit) { c.Proof.Pairing[1] = new(big.Int).Lsh(big.NewInt(1), 120) }},
	}
	for _, test := range mutations {
		t.Run(test.name, func(t *testing.T) { c := assignment(t, req); test.mutate(c); check(t, c, false) })
	}
}

func TestInputEncodings(t *testing.T) {
	req := fixture(t)
	proof, err := hexData(req.ProofHex, proofBytes)
	if err != nil {
		t.Fatal(err)
	}
	if err := assignProof(new(Circuit), proof[:len(proof)-1]); err == nil {
		t.Fatal("accepted short proof")
	}
	if err := assignProof(new(Circuit), append(append([]byte{}, proof...), 0)); err == nil {
		t.Fatal("accepted trailing proof bytes")
	}
	bad := append([]byte{}, proof...)
	// First pairing limb is limited to136bits, even though native Fr is254bits.
	bad[0] = 1
	if err := assignProof(new(Circuit), bad); err == nil {
		t.Fatal("accepted oversized pairing limb")
	}
	bad = append([]byte{}, proof...)
	// The first commitment coordinate must be canonical Fp, not merely reduced.
	copy(bad[8*32:8*32+32], bytes.Repeat([]byte{0xff}, 32))
	if err := assignProof(new(Circuit), bad); err == nil {
		t.Fatal("accepted noncanonical point coordinate")
	}
	for _, n := range []int{0, 3, 5} {
		modified := req
		modified.PublicInputs = make([]string, n)
		if _, _, _, err := publicAssignment(modified); err == nil {
			t.Fatalf("accepted %d public inputs", n)
		}
	}
	modified := req
	modified.PublicInputs = append([]string{}, req.PublicInputs...)
	b := make([]byte, 32)
	ecc.BN254.ScalarField().FillBytes(b)
	modified.PublicInputs[0] = "0x" + hex.EncodeToString(b)
	if _, _, _, err := publicAssignment(modified); err == nil {
		t.Fatal("accepted noncanonical public Fr")
	}
	modified = req
	modified.InnerVKHash = referenceHash
	if _, _, _, err := publicAssignment(modified); err == nil {
		t.Fatal("accepted wrong inner VK pin")
	}
}

func TestGroth16ProofBinding(t *testing.T) {
	keys, resultPath := os.Getenv("ULTRAHONK_WRAP_KEYS"), os.Getenv("ULTRAHONK_WRAP_RESULT")
	if keys == "" || resultPath == "" {
		t.Skip("set ULTRAHONK_WRAP_KEYS and ULTRAHONK_WRAP_RESULT to exercise actual outer proof verification")
	}
	b, err := os.ReadFile(resultPath)
	if err != nil {
		t.Fatal(err)
	}
	var req request
	if err := json.Unmarshal(b, &req); err != nil {
		t.Fatal(err)
	}
	s, err := loadSession(keys, false)
	if err != nil {
		t.Fatal(err)
	}
	if _, err := s.execute(context.Background(), req, true); err != nil {
		t.Fatalf("genuine outer proof: %v", err)
	}
	for i := range 4 {
		modified := req
		modified.PublicInputs = append([]string{}, req.PublicInputs...)
		raw, err := hexData(modified.PublicInputs[i], 32)
		if err != nil {
			t.Fatal(err)
		}
		x := new(big.Int).SetBytes(raw)
		x.Add(x, big.NewInt(1)).Mod(x, ecc.BN254.ScalarField()).FillBytes(raw)
		modified.PublicInputs[i] = "0x" + hex.EncodeToString(raw)
		if _, err := s.execute(context.Background(), modified, true); err == nil {
			t.Fatalf("accepted changed public field %d", i)
		}
	}
	modified := req
	raw, err := hexData(req.Groth16ProofHex, -1)
	if err != nil {
		t.Fatal(err)
	}
	raw[0] ^= 1
	modified.Groth16ProofHex = "0x" + hex.EncodeToString(raw)
	if _, err := s.execute(context.Background(), modified, true); err == nil {
		t.Fatal("accepted altered Groth16 proof")
	}
	modified.Groth16ProofHex = req.Groth16ProofHex + "00"
	if _, err := s.execute(context.Background(), modified, true); err == nil {
		t.Fatal("accepted trailing Groth16 proof byte")
	}
}
