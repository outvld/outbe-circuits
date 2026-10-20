// SPDX-License-Identifier: Apache-2.0
package main

import (
	"bytes"
	"context"
	"encoding/binary"
	"encoding/hex"
	"encoding/json"
	"io"
	"math/big"
	"os"
	"path/filepath"
	"reflect"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend/groth16"
	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
)

// This small real Groth16 circuit exercises artifact ownership and request
// lifecycle only. TestExactBB5ProofConstraints retains the full inner verifier
// coverage; no production circuit or compile options are changed here.
type lifecycleCircuit Circuit

var lifecycleGate struct {
	sync.Mutex
	started chan struct{}
	release chan struct{}
}

var registerLifecycleHint sync.Once

func lifecycleHint(_ *big.Int, inputs, outputs []*big.Int) error {
	lifecycleGate.Lock()
	started, release := lifecycleGate.started, lifecycleGate.release
	lifecycleGate.Unlock()
	if started != nil {
		close(started)
		<-release
	}
	outputs[0].Set(inputs[0])
	return nil
}

func (c *lifecycleCircuit) Define(api frontend.API) error {
	for i := range c.Public {
		api.AssertIsEqual(api.Mul(c.Public[i], c.Public[i]), c.Proof.Pairing[i])
	}
	hinted, err := api.Compiler().NewHint(lifecycleHint, 1, c.Public[0])
	if err != nil {
		return err
	}
	api.AssertIsEqual(hinted[0], c.Public[0])
	return nil
}

func writeProvenance(t *testing.T, dir string, p provenance) {
	t.Helper()
	data, err := json.Marshal(p)
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(dir, "setup.json"), data, 0600); err != nil {
		t.Fatal(err)
	}
}

func lifecycleSetup(t *testing.T) (string, request, provenance) {
	t.Helper()
	registerLifecycleHint.Do(func() { solver.RegisterHint(lifecycleHint) })
	// Match the production witness layout while intentionally omitting the
	// expensive inner-verifier constraints from this lifecycle-only fixture.
	cs, err := frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &lifecycleCircuit{}, frontend.IgnoreUnconstrainedInputs())
	if err != nil {
		t.Fatal(err)
	}
	pk, vk, err := groth16.Setup(cs)
	if err != nil {
		t.Fatal(err)
	}
	dir := t.TempDir()
	p := provenance{
		Circuit: "demo_tribute", InnerVKHash: innerVKHash, SolidityHash: referenceHash,
		InnerProofSystem: "ultrahonk-keccak", NoirVersion: "1.0.0-beta.22", BarretenbergVersion: "5.0.0-nightly.20260522",
		SourceHash: sourceHash(), GnarkRevision: gnarkRevision, Curve: "bn254",
		Kind: "single-party-experimental-not-production", Counts: shape(cs), Hashes: make(map[string]string),
	}
	for _, artifact := range []struct {
		name  string
		value io.WriterTo
	}{{"circuit.r1cs", cs}, {"proving.key", pk}, {"verifying.key", vk}} {
		digest, err := save(filepath.Join(dir, artifact.name), artifact.value)
		if err != nil {
			t.Fatal(err)
		}
		p.Hashes[artifact.name] = digest
	}
	writeProvenance(t, dir, p)
	inner := make([]byte, proofBytes)
	req := request{Circuit: "demo_tribute", InnerVKHash: innerVKHash}
	for i := range 4 {
		word := make([]byte, 32)
		big.NewInt(int64(i + 1)).FillBytes(word)
		req.PublicInputs = append(req.PublicInputs, "0x"+hex.EncodeToString(word))
		big.NewInt(int64((i + 1) * (i + 1))).FillBytes(inner[i*32 : (i+1)*32])
	}
	req.ProofHex = "0x" + hex.EncodeToString(inner)
	return dir, req, p
}

func assertInitialization(t *testing.T, init initialization, prover bool) {
	t.Helper()
	id, err := hex.DecodeString(init.ID)
	if err != nil || len(id) != 16 || init.ID != strings.ToLower(init.ID) {
		t.Fatalf("invalid initialization ID %q", init.ID)
	}
	started, err := time.Parse(time.RFC3339Nano, init.StartedAt)
	if err != nil {
		t.Fatal(err)
	}
	finished, err := time.Parse(time.RFC3339Nano, init.FinishedAt)
	if err != nil || finished.Before(started) {
		t.Fatalf("invalid initialization interval: %v", err)
	}
	if init.ProverLoaded != prover || init.WallMS < 0 || init.ProcessLifetimePeakRSSBytes == 0 {
		t.Fatalf("invalid initialization snapshot: %+v", init)
	}
	phases := []float64{init.Timings.LoadSetupProvenance, init.Timings.LoadVerifyingKey}
	if prover {
		if init.Timings.LoadProvingArtifacts == nil {
			t.Fatal("missing proving initialization phase")
		}
		phases = append(phases, *init.Timings.LoadProvingArtifacts)
	} else if init.Timings.LoadProvingArtifacts != nil {
		t.Fatal("verify-only session loaded proving artifacts")
	}
	var sum float64
	for _, elapsed := range phases {
		if elapsed < 0 || elapsed > init.WallMS {
			t.Fatalf("initialization phase outside wall interval: %v", elapsed)
		}
		sum += elapsed
	}
	if sum > init.WallMS {
		t.Fatalf("initialization phases exceed wall interval: %v > %v", sum, init.WallMS)
	}
}

func assertRequestMetrics(t *testing.T, res result, init initialization, prove bool) {
	t.Helper()
	if !res.Verified || !reflect.DeepEqual(res.Initialization, init) {
		t.Fatal("request changed initialization or failed to verify")
	}
	if res.RequestWallMS < 0 || res.ProcessLifetimePeakRSSBytes < init.ProcessLifetimePeakRSSBytes {
		t.Fatalf("invalid request metrics: %+v", res)
	}
	expected := []string{"groth16_verify"}
	if prove {
		expected = append(expected, "parse_inner_proof", "witness_encoding", "constraint_solve_and_groth16_prove")
	}
	if len(res.Timings) != len(expected) {
		t.Fatalf("request contains initialization or missing stages: %v", res.Timings)
	}
	for _, name := range expected {
		if elapsed, ok := res.Timings[name]; !ok || elapsed < 0 || elapsed > res.RequestWallMS {
			t.Fatalf("invalid request stage %s: %v", name, res.Timings)
		}
	}
	data, err := json.Marshal(res)
	if err != nil {
		t.Fatal(err)
	}
	var fields map[string]json.RawMessage
	if err := json.Unmarshal(data, &fields); err != nil {
		t.Fatal(err)
	}
	for _, name := range []string{"process_wall_ms", "process_peak_rss_bytes"} {
		if _, ok := fields[name]; ok {
			t.Fatalf("obsolete request metric %s", name)
		}
	}
}

func TestSessionRetainsCheckedArtifacts(t *testing.T) {
	keys, req, _ := lifecycleSetup(t)
	s, err := loadSession(keys, true)
	if err != nil {
		t.Fatal(err)
	}
	assertInitialization(t, s.initialization, true)
	for _, name := range []string{"circuit.r1cs", "proving.key"} {
		if err := os.Remove(filepath.Join(keys, name)); err != nil {
			t.Fatal(err)
		}
	}
	verifier, err := loadSession(keys, false)
	if err != nil {
		t.Fatalf("verify-only loader required proving artifacts: %v", err)
	}
	assertInitialization(t, verifier.initialization, false)
	if verifier.initialization.ID == s.initialization.ID {
		t.Fatal("separate initializations reused an instance ID")
	}
	if err := os.RemoveAll(keys); err != nil {
		t.Fatal(err)
	}
	var previous string
	for range 2 {
		res, err := s.execute(context.Background(), req, false)
		if err != nil {
			t.Fatalf("retained proving artifacts: %v", err)
		}
		assertRequestMetrics(t, res, s.initialization, true)
		if res.Groth16ProofHex == previous {
			t.Fatal("proving did not produce a fresh randomized proof")
		}
		previous = res.Groth16ProofHex
		verifyReq := req
		verifyReq.Groth16ProofHex = res.Groth16ProofHex
		verified, err := verifier.execute(context.Background(), verifyReq, true)
		if err != nil {
			t.Fatalf("retained verifying key: %v", err)
		}
		assertRequestMetrics(t, verified, verifier.initialization, false)
		for i := range 4 {
			modified := verifyReq
			modified.PublicInputs = append([]string{}, verifyReq.PublicInputs...)
			modified.PublicInputs[i] = "0x" + strings.Repeat("0", 64)
			if _, err := verifier.execute(context.Background(), modified, true); err == nil {
				t.Fatalf("accepted changed public field %d", i)
			}
		}
		verifyReq.InnerVKHash = referenceHash
		if _, err := verifier.execute(context.Background(), verifyReq, true); err == nil {
			t.Fatal("accepted wrong inner VK pin")
		}
	}
	if _, err := verifier.execute(context.Background(), req, false); err == nil {
		t.Fatal("verify-only session allowed proving")
	}
}

func TestArtifactLoadChecksIntegrityAndDecoderBoundary(t *testing.T) {
	keys, _, p := lifecycleSetup(t)
	for _, artifact := range []struct {
		name      string
		newObject func() io.ReaderFrom
	}{
		{"verifying.key", func() io.ReaderFrom { return groth16.NewVerifyingKey(ecc.BN254) }},
		{"proving.key", func() io.ReaderFrom { return groth16.NewProvingKey(ecc.BN254) }},
		{"circuit.r1cs", func() io.ReaderFrom { return groth16.NewCS(ecc.BN254) }},
	} {
		t.Run(artifact.name, func(t *testing.T) {
			data, err := os.ReadFile(filepath.Join(keys, artifact.name))
			if err != nil {
				t.Fatal(err)
			}
			if err := load(filepath.Join(keys, artifact.name), p.Hashes[artifact.name], artifact.newObject()); err != nil {
				t.Fatalf("valid artifact rejected: %v", err)
			}
			path := filepath.Join(t.TempDir(), artifact.name)
			trailing := append(append([]byte{}, data...), 0x42)
			if err := os.WriteFile(path, trailing, 0600); err != nil {
				t.Fatal(err)
			}
			if err := load(path, p.Hashes[artifact.name], artifact.newObject()); err == nil || !strings.Contains(err.Error(), "hash mismatch") {
				t.Fatalf("accepted artifact digest mismatch: %v", err)
			}
			// The digest covers the trailing byte; only the same buffered reader
			// used by ReadFrom can detect bytes prefetched past the object.
			if err := load(path, hashBytes(trailing), artifact.newObject()); err == nil || !strings.Contains(err.Error(), "trailing setup artifact bytes") {
				t.Fatalf("accepted trailing artifact bytes: %v", err)
			}
			if err := os.WriteFile(path, data[:len(data)/2], 0600); err != nil {
				t.Fatal(err)
			}
			if err := load(path, hashBytes(data[:len(data)/2]), artifact.newObject()); err == nil {
				t.Fatal("accepted truncated artifact")
			}
		})
	}
	data, err := os.ReadFile(filepath.Join(keys, "verifying.key"))
	if err != nil {
		t.Fatal(err)
	}
	copy(data[:32], bytes.Repeat([]byte{0xff}, 32))
	path := filepath.Join(t.TempDir(), "invalid.key")
	if err := os.WriteFile(path, data, 0600); err != nil {
		t.Fatal(err)
	}
	if err := load(path, hashBytes(data), groth16.NewVerifyingKey(ecc.BN254)); err == nil {
		t.Fatal("accepted noncanonical curve point with a matching digest")
	}
	for _, digest := range []string{"", strings.Repeat("z", 64)} {
		if err := load(path, digest, groth16.NewVerifyingKey(ecc.BN254)); err == nil {
			t.Fatal("accepted missing or malformed artifact digest")
		}
	}
}

func TestSessionRejectsChangedSetup(t *testing.T) {
	for _, test := range []string{"missing-key", "corrupt-key", "wrong-source", "wrong-counts", "trailing-provenance"} {
		t.Run(test, func(t *testing.T) {
			keys, _, p := lifecycleSetup(t)
			switch test {
			case "missing-key":
				if err := os.Remove(filepath.Join(keys, "proving.key")); err != nil {
					t.Fatal(err)
				}
			case "corrupt-key":
				if err := os.WriteFile(filepath.Join(keys, "verifying.key"), []byte("not a key"), 0600); err != nil {
					t.Fatal(err)
				}
			case "wrong-source":
				p.SourceHash = strings.Repeat("0", 64)
				writeProvenance(t, keys, p)
			case "wrong-counts":
				p.Counts.Constraints++
				writeProvenance(t, keys, p)
			case "trailing-provenance":
				data, err := os.ReadFile(filepath.Join(keys, "setup.json"))
				if err != nil {
					t.Fatal(err)
				}
				if err := os.WriteFile(filepath.Join(keys, "setup.json"), append(data, []byte(" {}")...), 0600); err != nil {
					t.Fatal(err)
				}
			}
			if s, err := newWrapperService(keys); err == nil || s != nil {
				t.Fatalf("invalid setup created a resident service: %v", err)
			}
		})
	}
}

func TestRetainedVerifierRejectsMalformedProof(t *testing.T) {
	keys, req, _ := lifecycleSetup(t)
	s, err := loadSession(keys, true)
	if err != nil {
		t.Fatal(err)
	}
	res, err := s.execute(context.Background(), req, false)
	if err != nil {
		t.Fatal(err)
	}
	for _, kind := range []string{"altered", "trailing", "commitment-count"} {
		t.Run(kind, func(t *testing.T) {
			modified := req
			encoded, err := hexData(res.Groth16ProofHex, -1)
			if err != nil {
				t.Fatal(err)
			}
			switch kind {
			case "altered":
				encoded[0] ^= 1
			case "trailing":
				encoded = append(encoded, 0)
			case "commitment-count":
				binary.BigEndian.PutUint32(encoded[128:132], ^uint32(0))
			}
			modified.Groth16ProofHex = "0x" + hex.EncodeToString(encoded)
			if _, err := s.execute(context.Background(), modified, true); err == nil {
				t.Fatalf("accepted %s Groth16 proof", kind)
			}
		})
	}
}
