// SPDX-License-Identifier: Apache-2.0
// Isolated experimental direct UltraHonk -> BN254 Groth16 wrapper.
//
//	go run . setup --out ../../target/ultrahonk-wrap-keys
//	go run . prove --keys ../../target/ultrahonk-wrap-keys < request.json
//	go run . verify --keys ../../target/ultrahonk-wrap-keys < result.json
//
// Request JSON: {"circuit":"demo_tribute","proof_hex":"0x...",
// "public_inputs":["0x<32bytes>","0x<32bytes>","0x<32bytes>","0x<32bytes>"],
// "inner_vk_sha256":"6b5d5609be0894c5f8379803ab6421bf66b0f1c25c5667644772f2d2bf7225bc"}.
// proof_hex is the EXACT 8768-byte keccak, ZK, non-IPA, non-optimized-EVM proof
// returned by the pinned BB5 low-level API; public inputs are not prepended.
// Output includes a real gnark compressed Groth16 proof, verified flag, original
// four public words, stage timings, constraint counts and setup provenance.
// Setup is single-party, OS-random, unaudited, and NOT production trustworthy.
// The prover's OS randomness and toxic waste are never logged or serialized.
package main

import (
	"bufio"
	"bytes"
	"context"
	"crypto/rand"
	"crypto/sha256"
	"embed"
	"encoding/binary"
	"encoding/hex"
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"io"
	"math/big"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"time"

	"github.com/consensys/gnark-crypto/ecc"
	cryptobn "github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark/backend/groth16"
	groth16bn "github.com/consensys/gnark/backend/groth16/bn254"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
	"github.com/consensys/gnark/logger"
	"github.com/consensys/gnark/std/math/emulated"
)

//go:embed circuit.go relations.go shplemini.go vk.go
var circuitSources embed.FS

const gnarkRevision = "f9bda493064413cbc6a187f8c9617046915bb2e4"

type request struct {
	Circuit         string   `json:"circuit"`
	ProofHex        string   `json:"proof_hex"`
	PublicInputs    []string `json:"public_inputs"`
	InnerVKHash     string   `json:"inner_vk_sha256"`
	Groth16ProofHex string   `json:"groth16_proof_hex"`
}

type counts struct {
	Constraints          int `json:"constraints"`
	Internal             int `json:"internal_variables"`
	Secret               int `json:"secret_variables"`
	Public               int `json:"public_inputs"`
	CommitmentExtensions int `json:"groth16_commitment_extensions"`
}

type provenance struct {
	Circuit                   string             `json:"circuit"`
	InnerVKHash               string             `json:"inner_vk_sha256"`
	InnerProofSystem          string             `json:"inner_proof_system"`
	NoirVersion               string             `json:"noir_version"`
	BarretenbergVersion       string             `json:"barretenberg_version"`
	DisableZK                 bool               `json:"disable_zk"`
	IPAAccumulation           bool               `json:"ipa_accumulation"`
	OptimizedSolidityVerifier bool               `json:"optimizedSolidityVerifier"`
	SolidityHash              string             `json:"reference_solidity_sha256"`
	SourceHash                string             `json:"constraint_sources_sha256"`
	GnarkRevision             string             `json:"gnark_revision"`
	Curve                     string             `json:"outer_curve"`
	Kind                      string             `json:"setup_kind"`
	Randomness                string             `json:"randomness"`
	CreatedUTC                string             `json:"created_utc"`
	GoVersion                 string             `json:"go_version"`
	Counts                    counts             `json:"structural_counts"`
	Hashes                    map[string]string  `json:"artifact_sha256"`
	Timings                   map[string]float64 `json:"setup_timings_ms"`
	ProcessPeakRSSBytes       uint64             `json:"process_peak_rss_bytes,omitempty"`
}

type result struct {
	Circuit                     string             `json:"circuit"`
	Pipeline                    string             `json:"pipeline"`
	Verified                    bool               `json:"verified"`
	PublicInputs                []string           `json:"public_inputs"`
	InnerVKHash                 string             `json:"inner_vk_sha256"`
	InnerProofHash              string             `json:"inner_proof_sha256,omitempty"`
	StatementHash               string             `json:"statement_sha256"`
	Groth16ProofHex             string             `json:"groth16_proof_hex"`
	Groth16ProofHash            string             `json:"groth16_proof_sha256"`
	Groth16ProofBytes           int                `json:"groth16_proof_bytes"`
	Groth16Encoding             string             `json:"groth16_encoding"`
	Groth16Commitments          int                `json:"groth16_commitments"`
	Timings                     map[string]float64 `json:"timings_ms"`
	Counts                      counts             `json:"structural_counts"`
	Setup                       provenance         `json:"setup_provenance"`
	Initialization              initialization     `json:"initialization"`
	ProcessLifetimePeakRSSBytes uint64             `json:"process_lifetime_peak_rss_bytes"`
	RequestWallMS               float64            `json:"request_wall_ms"`
}

type initializationTimings struct {
	LoadSetupProvenance  float64  `json:"load_setup_provenance"`
	LoadVerifyingKey     float64  `json:"load_verifying_key"`
	LoadProvingArtifacts *float64 `json:"load_proving_artifacts,omitempty"`
}

type initialization struct {
	ID                          string                `json:"id"`
	StartedAt                   string                `json:"started_at"`
	FinishedAt                  string                `json:"finished_at"`
	ProverLoaded                bool                  `json:"prover_loaded"`
	Timings                     initializationTimings `json:"timings_ms"`
	WallMS                      float64               `json:"wall_ms"`
	ProcessLifetimePeakRSSBytes uint64                `json:"process_lifetime_peak_rss_bytes"`
}

// A session owns checked artifacts for its entire lifetime. Requests never
// consult the setup directory again; only witnesses and proofs are per-request.
type session struct {
	setup          provenance
	initialization initialization
	cs             constraint.ConstraintSystem
	pk             groth16.ProvingKey
	vk             groth16.VerifyingKey
}

func ms(start time.Time) float64 { return float64(time.Since(start).Nanoseconds()) / 1e6 }
func hashBytes(b []byte) string  { sum := sha256.Sum256(b); return hex.EncodeToString(sum[:]) }

func sourceHash() string {
	h := sha256.New()
	for _, path := range []string{"circuit.go", "relations.go", "shplemini.go", "vk.go"} {
		b, err := circuitSources.ReadFile(path)
		if err != nil {
			panic(err)
		}
		h.Write([]byte(path + "\x00"))
		h.Write(b)
	}
	return hex.EncodeToString(h.Sum(nil))
}

func shape(cs constraint.ConstraintSystem) counts {
	return counts{cs.GetNbConstraints(), cs.GetNbInternalVariables(), cs.GetNbSecretVariables(), cs.GetNbPublicVariables() - 1, len(cs.GetCommitments().(constraint.Groth16Commitments))}
}

// gnark's NbPublicWitness includes commitment-derived verifier slots. They are
// recomputed from the proof, NOT extra caller-supplied public statement fields.
func publicArity(vk groth16.VerifyingKey) int {
	key := vk.(*groth16bn.VerifyingKey)
	return len(key.G1.K) - len(key.PublicAndCommitmentCommitted) - 1
}

func compile() (constraint.ConstraintSystem, error) {
	return frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &Circuit{})
}

func save(path string, object io.WriterTo) (string, error) {
	f, err := os.OpenFile(path, os.O_CREATE|os.O_EXCL|os.O_WRONLY, 0600)
	if err != nil {
		return "", err
	}
	h := sha256.New()
	_, err = object.WriteTo(io.MultiWriter(f, h))
	closeErr := f.Close()
	if err != nil {
		return "", err
	}
	if closeErr != nil {
		return "", closeErr
	}
	return hex.EncodeToString(h.Sum(nil)), nil
}

func setup(out string) error {
	// Never overwrite a prior experimental ceremony or mix keys from two setups.
	if err := os.Mkdir(out, 0700); err != nil {
		return fmt.Errorf("new setup directory: %w", err)
	}
	start := time.Now()
	cs, err := compile()
	if err != nil {
		return err
	}
	compileMS := ms(start)
	if shape(cs).Public != 4 {
		return errors.New("unexpected outer public arity")
	}
	start = time.Now()
	pk, vk, err := groth16.Setup(cs)
	if err != nil {
		return err
	}
	setupMS := ms(start)
	if publicArity(vk) != 4 {
		return errors.New("unexpected verifying key public arity")
	}
	p := provenance{
		Circuit: "demo_tribute", InnerVKHash: innerVKHash, SolidityHash: referenceHash,
		InnerProofSystem: "ultrahonk-keccak", NoirVersion: "1.0.0-beta.22", BarretenbergVersion: "5.0.0-nightly.20260522",
		SourceHash: sourceHash(), GnarkRevision: gnarkRevision, Curve: "bn254",
		Kind: "single-party-experimental-not-production", Randomness: "gnark/gnark-crypto crypto/rand (OS CSPRNG); toxic waste not retained",
		CreatedUTC: time.Now().UTC().Format(time.RFC3339Nano), GoVersion: runtime.Version(),
		Counts: shape(cs), Hashes: make(map[string]string), Timings: map[string]float64{"constraint_compile": compileMS, "groth16_setup": setupMS},
	}
	for _, artifact := range []struct {
		name  string
		value io.WriterTo
	}{{"circuit.r1cs", cs}, {"proving.key", pk}, {"verifying.key", vk}} {
		hash, err := save(filepath.Join(out, artifact.name), artifact.value)
		if err != nil {
			return err
		}
		p.Hashes[artifact.name] = hash
	}
	p.ProcessPeakRSSBytes = peakRSSBytes()
	data, err := json.MarshalIndent(p, "", "  ")
	if err != nil {
		return err
	}
	if err := os.WriteFile(filepath.Join(out, "setup.json"), append(data, '\n'), 0600); err != nil {
		return err
	}
	return json.NewEncoder(os.Stdout).Encode(p)
}

func load(path, expected string, object io.ReaderFrom) error {
	digest, err := hex.DecodeString(expected)
	if err != nil || len(digest) != sha256.Size {
		return errors.New("missing or invalid setup artifact digest")
	}
	f, err := os.Open(path)
	if err != nil {
		return err
	}
	defer f.Close()
	h := sha256.New()
	if _, err := io.Copy(h, f); err != nil {
		return err
	}
	if hex.EncodeToString(h.Sum(nil)) != expected {
		return fmt.Errorf("setup artifact hash mismatch: %s", path)
	}
	if _, err := f.Seek(0, io.SeekStart); err != nil {
		return err
	}
	reader := bufio.NewReaderSize(f, 1<<20)
	if _, err := object.ReadFrom(reader); err != nil {
		return err
	}
	if _, err := reader.ReadByte(); err != io.EOF {
		return errors.New("trailing setup artifact bytes")
	}
	return nil
}

func loadSetup(dir string) (provenance, error) {
	var p provenance
	f, err := os.Open(filepath.Join(dir, "setup.json"))
	if err != nil {
		return p, err
	}
	defer f.Close()
	data, err := io.ReadAll(io.LimitReader(f, 65537))
	if err != nil {
		return p, err
	}
	if len(data) > 65536 {
		return p, errors.New("setup provenance exceeds limit")
	}
	if err := json.Unmarshal(data, &p); err != nil {
		return p, err
	}
	if p.Circuit != "demo_tribute" || p.InnerVKHash != innerVKHash || p.SolidityHash != referenceHash || p.SourceHash != sourceHash() || p.GnarkRevision != gnarkRevision || p.Curve != "bn254" || p.Counts.Public != 4 || p.Kind != "single-party-experimental-not-production" {
		return p, errors.New("setup provenance is not for this exact wrapper build")
	}
	if p.InnerProofSystem != "ultrahonk-keccak" || p.NoirVersion != "1.0.0-beta.22" || p.BarretenbergVersion != "5.0.0-nightly.20260522" || p.DisableZK || p.IPAAccumulation || p.OptimizedSolidityVerifier {
		return p, errors.New("setup inner proof-system configuration mismatch")
	}
	return p, nil
}

func loadSession(dir string, prover bool) (*session, error) {
	started := time.Now()
	var id [16]byte
	if _, err := rand.Read(id[:]); err != nil {
		return nil, fmt.Errorf("initialization ID: %w", err)
	}
	s := &session{initialization: initialization{
		ID: hex.EncodeToString(id[:]), StartedAt: started.UTC().Format(time.RFC3339Nano),
		ProverLoaded: prover,
	}}
	start := time.Now()
	p, err := loadSetup(dir)
	if err != nil {
		return nil, err
	}
	s.setup = p
	s.initialization.Timings.LoadSetupProvenance = ms(start)
	start = time.Now()
	s.vk = groth16.NewVerifyingKey(ecc.BN254)
	if err := load(filepath.Join(dir, "verifying.key"), p.Hashes["verifying.key"], s.vk); err != nil {
		return nil, err
	}
	if publicArity(s.vk) != 4 {
		return nil, errors.New("wrong outer key arity")
	}
	if len(s.vk.(*groth16bn.VerifyingKey).PublicAndCommitmentCommitted) != p.Counts.CommitmentExtensions {
		return nil, errors.New("verifying-key commitment count mismatch")
	}
	s.initialization.Timings.LoadVerifyingKey = ms(start)
	if prover {
		start = time.Now()
		s.cs = groth16.NewCS(ecc.BN254)
		if err := load(filepath.Join(dir, "circuit.r1cs"), p.Hashes["circuit.r1cs"], s.cs); err != nil {
			return nil, err
		}
		if shape(s.cs) != p.Counts {
			return nil, errors.New("constraint-system counts mismatch")
		}
		s.pk = groth16.NewProvingKey(ecc.BN254)
		if err := load(filepath.Join(dir, "proving.key"), p.Hashes["proving.key"], s.pk); err != nil {
			return nil, err
		}
		elapsed := ms(start)
		s.initialization.Timings.LoadProvingArtifacts = &elapsed
	}
	s.initialization.ProcessLifetimePeakRSSBytes = peakRSSBytes()
	finished := time.Now()
	s.initialization.FinishedAt = finished.UTC().Format(time.RFC3339Nano)
	s.initialization.WallMS = float64(finished.Sub(started).Nanoseconds()) / 1e6
	return s, nil
}

func hexData(s string, size int) ([]byte, error) {
	if !strings.HasPrefix(s, "0x") {
		return nil, errors.New("hex requires 0x prefix")
	}
	if size >= 0 && len(s) != 2+2*size {
		return nil, fmt.Errorf("expected %d bytes", size)
	}
	b, err := hex.DecodeString(s[2:])
	if err != nil {
		return nil, err
	}
	return b, nil
}

func publicAssignment(req request) (*Circuit, []string, []byte, error) {
	if req.Circuit != "demo_tribute" || req.InnerVKHash != innerVKHash {
		return nil, nil, nil, errors.New("circuit or inner VK pin mismatch")
	}
	if len(req.PublicInputs) != 4 {
		return nil, nil, nil, errors.New("exactly four public Fr words required")
	}
	assignment := new(Circuit)
	canonical := make([]string, 4)
	statement := make([]byte, 0, 128)
	for i, s := range req.PublicInputs {
		b, err := hexData(s, 32)
		if err != nil {
			return nil, nil, nil, fmt.Errorf("public input %d: %w", i, err)
		}
		x := new(big.Int).SetBytes(b)
		if x.Cmp(ecc.BN254.ScalarField()) >= 0 {
			return nil, nil, nil, fmt.Errorf("public input %d is noncanonical Fr", i)
		}
		assignment.Public[i] = x
		canonical[i] = "0x" + hex.EncodeToString(b)
		statement = append(statement, b...)
	}
	return assignment, canonical, statement, nil
}

// No native UltraHonk verifier runs here: parsing merely preserves canonical
// field/group encodings. Every cryptographic acceptance check is in Circuit.
func assignProof(c *Circuit, proof []byte) error {
	if len(proof) != proofBytes {
		return fmt.Errorf("expected exact BB5 ZK proof of %d bytes, got %d", proofBytes, len(proof))
	}
	position := 0
	word := func(modulus *big.Int) (*big.Int, error) {
		if position+32 > len(proof) {
			return nil, io.ErrUnexpectedEOF
		}
		x := new(big.Int).SetBytes(proof[position : position+32])
		position += 32
		if x.Cmp(modulus) >= 0 {
			return nil, fmt.Errorf("noncanonical proof word %d", position/32-1)
		}
		return x, nil
	}
	fr := func(dst *V) error {
		x, err := word(ecc.BN254.ScalarField())
		if err == nil {
			*dst = x
		}
		return err
	}
	point := func(dst *Point) error {
		x, err := word(new(emulated.BN254Fp).Modulus())
		if err != nil {
			return err
		}
		y, err := word(new(emulated.BN254Fp).Modulus())
		if err != nil {
			return err
		}
		*dst = Point{X: emulated.ValueOf[emulated.BN254Fp](x), Y: emulated.ValueOf[emulated.BN254Fp](y)}
		return nil
	}
	p := &c.Proof
	for i := range p.Pairing {
		width := uint(136)
		if i%2 == 1 {
			width = 120
		}
		x, err := word(new(big.Int).Lsh(big.NewInt(1), width))
		if err != nil {
			return err
		}
		p.Pairing[i] = x
	}
	for _, dst := range []*Point{&p.Mask, &p.W1, &p.W2, &p.W3, &p.ReadCounts, &p.ReadTags, &p.W4, &p.Inverses, &p.ZPerm, &p.Libra[0]} {
		if err := point(dst); err != nil {
			return err
		}
	}
	if err := fr(&p.LibraSum); err != nil {
		return err
	}
	for i := range p.Univariates {
		for j := range p.Univariates[i] {
			if err := fr(&p.Univariates[i][j]); err != nil {
				return err
			}
		}
	}
	for i := range p.Evaluations {
		if err := fr(&p.Evaluations[i]); err != nil {
			return err
		}
	}
	if err := fr(&p.LibraEvaluation); err != nil {
		return err
	}
	if err := point(&p.Libra[1]); err != nil {
		return err
	}
	if err := point(&p.Libra[2]); err != nil {
		return err
	}
	for i := range p.Folds {
		if err := point(&p.Folds[i]); err != nil {
			return err
		}
	}
	for i := range p.Gemini {
		if err := fr(&p.Gemini[i]); err != nil {
			return err
		}
	}
	for i := range p.LibraEvals {
		if err := fr(&p.LibraEvals[i]); err != nil {
			return err
		}
	}
	if err := point(&p.Shplonk); err != nil {
		return err
	}
	if err := point(&p.Quotient); err != nil {
		return err
	}
	if position != len(proof) {
		return errors.New("unconsumed proof bytes")
	}
	return nil
}

func (s *session) execute(ctx context.Context, req request, verifyOnly bool) (out result, err error) {
	requestStart := time.Now()
	out = result{
		Circuit: "demo_tribute", Pipeline: "direct-ultrakeccakzk-bb5-to-gnark-groth16",
		InnerVKHash: innerVKHash, Timings: make(map[string]float64),
		Setup: s.setup, Counts: s.setup.Counts, Initialization: s.initialization,
	}
	defer func() {
		out.RequestWallMS = ms(requestStart)
		out.ProcessLifetimePeakRSSBytes = peakRSSBytes()
	}()
	if err := ctx.Err(); err != nil {
		return out, err
	}
	if !verifyOnly && (s.cs == nil || s.pk == nil) {
		return out, errors.New("session has no proving artifacts")
	}
	start := time.Now()
	assignment, canonical, statement, err := publicAssignment(req)
	if err != nil {
		return out, err
	}
	out.PublicInputs, out.StatementHash = canonical, hashBytes(statement)
	var encoded []byte
	proof := groth16.NewProof(ecc.BN254)
	if verifyOnly {
		extensions := len(s.vk.(*groth16bn.VerifyingKey).PublicAndCommitmentCommitted)
		encodedBytes := (3+extensions)*cryptobn.SizeOfG1AffineCompressed + cryptobn.SizeOfG2AffineCompressed + 4
		encoded, err = hexData(req.Groth16ProofHex, encodedBytes)
		if err != nil {
			return out, err
		}
		// Bound the length-prefixed commitment vector before the general decoder
		// can allocate memory using a hostile request-controlled length.
		offset := 2*cryptobn.SizeOfG1AffineCompressed + cryptobn.SizeOfG2AffineCompressed
		if int(binary.BigEndian.Uint32(encoded[offset:offset+4])) != extensions {
			return out, errors.New("wrong Groth16 commitment-extension count")
		}
		r := bytes.NewReader(encoded)
		if _, err := proof.ReadFrom(r); err != nil {
			return out, err
		}
		if r.Len() != 0 {
			return out, errors.New("trailing Groth16 proof bytes")
		}
	} else {
		inner, err := hexData(req.ProofHex, proofBytes)
		if err != nil {
			return out, err
		}
		if err := assignProof(assignment, inner); err != nil {
			return out, err
		}
		out.InnerProofHash = hashBytes(inner)
		out.Timings["parse_inner_proof"] = ms(start)
		start = time.Now()
		witness, err := frontend.NewWitness(assignment, ecc.BN254.ScalarField())
		if err != nil {
			return out, err
		}
		out.Timings["witness_encoding"] = ms(start)
		start = time.Now()
		// gnark has no context-aware native prover. Once started it must finish.
		if err := ctx.Err(); err != nil {
			return out, err
		}
		proof, err = groth16.Prove(s.cs, s.pk, witness)
		if err != nil {
			return out, fmt.Errorf("constrained inner verification/proving: %w", err)
		}
		out.Timings["constraint_solve_and_groth16_prove"] = ms(start)
		var buffer bytes.Buffer
		if _, err := proof.WriteTo(&buffer); err != nil {
			return out, err
		}
		encoded = buffer.Bytes()
	}
	start = time.Now()
	pubs, err := frontend.NewWitness(assignment, ecc.BN254.ScalarField(), frontend.PublicOnly())
	if err != nil {
		return out, err
	}
	if verifyOnly {
		if err := ctx.Err(); err != nil {
			return out, err
		}
	}
	if err := groth16.Verify(proof, s.vk, pubs); err != nil {
		return out, fmt.Errorf("outer Groth16 verification: %w", err)
	}
	out.Timings["groth16_verify"] = ms(start)
	out.Verified = true
	out.Groth16ProofHex, out.Groth16ProofHash, out.Groth16ProofBytes = "0x"+hex.EncodeToString(encoded), hashBytes(encoded), len(encoded)
	out.Groth16Encoding = "gnark-bn254-WriteTo-compressed-Ar-Bs-Krs-commitments-commitmentPok"
	out.Groth16Commitments = len(proof.(*groth16bn.Proof).Commitments)
	return out, nil
}

func run() error {
	logger.SetOutput(os.Stderr)
	if len(os.Args) < 2 {
		return errors.New("usage: ultrahonk-wrap setup --out NEW_DIRECTORY | prove --keys DIRECTORY | verify --keys DIRECTORY | serve --keys DIRECTORY --web DIRECTORY --listen ADDRESS")
	}
	command := os.Args[1]
	flags := flag.NewFlagSet(command, flag.ContinueOnError)
	directory := flags.String("keys", "", "trusted local setup directory")
	out := flags.String("out", "", "new single-party experimental setup directory")
	web := flags.String("web", "", "static web directory")
	listen := flags.String("listen", "0.0.0.0:8081", "HTTP listen address")
	if err := flags.Parse(os.Args[2:]); err != nil {
		return err
	}
	if flags.NArg() != 0 {
		return errors.New("unexpected positional arguments")
	}
	if command == "setup" {
		if *out == "" {
			return errors.New("setup requires --out")
		}
		return setup(*out)
	}
	if command == "serve" {
		if *directory == "" || *web == "" {
			return errors.New("serve requires --keys and --web")
		}
		return serve(*directory, *web, *listen)
	}
	if command != "prove" && command != "verify" {
		return errors.New("unknown command")
	}
	if *directory == "" {
		return errors.New("--keys is required")
	}
	s, err := loadSession(*directory, command == "prove")
	if err != nil {
		return err
	}
	requestStart := time.Now()
	var req request
	decoder := json.NewDecoder(io.LimitReader(os.Stdin, 2*proofBytes+65536))
	if err := decoder.Decode(&req); err != nil {
		return err
	}
	if err := decoder.Decode(new(any)); err != io.EOF {
		return errors.New("trailing request data")
	}
	res, err := s.execute(context.Background(), req, command == "verify")
	if err != nil {
		return err
	}
	res.RequestWallMS = ms(requestStart)
	return json.NewEncoder(os.Stdout).Encode(res)
}

func main() {
	if err := run(); err != nil {
		json.NewEncoder(os.Stdout).Encode(map[string]any{"verified": false, "error": err.Error()})
		os.Exit(1)
	}
}
