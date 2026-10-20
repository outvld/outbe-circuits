# outbe-circuits

Rust workspace for the **Outbe zero-knowledge protocol**: concrete BN254/Grumpkin consensus primitives, a Noir + barretenberg proving backend, and a frozen, versioned canonical circuit registry.

## Crates

| Crate | What it is |
|-------|------------|
| [`outbe-protocol`](crates/outbe-protocol) | Concrete BN254 / Grumpkin / Poseidon2 / Schnorr consensus primitives. Module-level formulas live in `primitive::hash`; entity, signer, circuit and backend interfaces remain extensible. Hashing routes through [`outbe-poseidon`](https://github.com/outbe/outbe-poseidon). |
| [`outbe-protocol-derive`](crates/outbe-protocol-derive) | `#[derive(Entity)]` — maps a typed struct's `#[outbe(...)]`-annotated fields to the canonical entity-hash preimage. |
| [`outbe-zk-backend`](crates/outbe-zk-backend) | Noir proving backend: an ACVM witness solver plus a barretenberg (UltraHonkKeccak, FFI) prover/verifier. Generic over any circuit implementing the `outbe-protocol` zk seams. |
| [`outbe-zk-canonical`](crates/outbe-zk-canonical) | Concrete circuit/witness types, append-only L1 releases, and VK-pinned L2 chain-version selections. Builds from committed artifacts — ships to crates.io, no Noir toolchain required. |
| `xtask` | Circuit tooling: `cargo xtask test-circuits`, `cargo xtask freeze-circuits`, and the read-only `cargo xtask freeze-circuits --check`. |

## Build

Plain cargo works for everything and needs **no** Noir toolchain — `outbe-zk-canonical` builds from its committed frozen artifacts:

```bash
cargo build --workspace
cargo test  --workspace           # fast suites: outbe-protocol + outbe-zk-canonical
cargo test  -p outbe-zk-backend   # barretenberg proving round-trips (builds the C++ FFI; downloads SRS)
```

The first build compiles the bundled barretenberg C++ FFI (several minutes — not hung). `[profile.dev]` is `opt-level = 3` deliberately; proving is unusably slow otherwise.

## Circuits

The catalog in `crates/outbe-zk-canonical/circuits/manifest.toml` contains four active L1 circuits and two L2 chain registrations. Paths below are relative to that crate:

| Directory | Nargo name | Type | Role |
|-----------|-----------|------|------|
| `noir/outbe-circuit-core/` | `outbe_circuit_core` | lib | Shared `ownership`, `inclusion`, and `hash` modules. Depends on `noir-lang/schnorr` v0.2.0; Poseidon2 via the stdlib permutation. |
| `noir/outbe-paynote-lib/` | `paynote_lib` | lib | Shared Paynote hashing and Merkle formulas used by spend and merge circuits. |
| `noir/ownership_proof/` | `ownership_proof` | bin | Single-NFT ownership proof. |
| `l2/57005/tribute/` | `demo_tribute` | bin | Demo Tribute ownership + depth-32 Merkle inclusion. |
| `noir/emit_mint/` | `emit_mint` | bin | Contains all Emit-specific formulas plus mint, nullifier, membership, and optional change constraints. |
| `noir/paynote/` | `paynote` | bin | Private ERC20 payment note with 256-bit amounts, bearer spend authority by key knowledge, asset-bound commitment, nullifier, membership, and optional change. |
| `noir/paynote_merge/` | `paynote_merge` | bin | Merge existing Paynotes into a single note. |
| `l2/9900501/tribute/` | `niflheim_tribute` | bin | Self-contained Niflheim Tribute ownership and depth-32 Merkle inclusion proof. |

L1 uses conventional source and artifact paths: `noir/<module>/` and `resources/circuits/<module>/<version>/`. L2 chain versions select arrays of package paths with pinned `vk_hash` values; each package stores `abi.json`, `bytecode.b64`, and `circuit.vk` directly under `l2/<chain_id>/<package>/`. `outbe_zk_canonical::l2_circuits(chain_id, version)` returns the exact selection's package/key descriptors without per-chain Rust modules. Demo uses mutable chain version `1.0.0`; Niflheim uses stable `1.0.0`.

### Toolchain

The Noir toolchain is needed only to evolve circuits or check their reproducibility, pinned via `mise`:

```bash
mise install nargo bb    # nargo 1.0.0-beta.22 + bb 5.0.0-nightly.20260522
```

`cargo xtask` is aliased in `.cargo/config.toml`.

### Circuit-change workflow

```bash
cargo xtask freeze-circuits          # mint L1 releases / refresh permitted L2 artifacts
```

For L1, changed ACIR or ABI mints a release and appends an active `[[circuit.versions]]` record; the previous release becomes deprecated and keeps its VK. ABI changes require `--abi-change` (minor) or `--semantic` (major + new domain decision).

For L2, `[[l2_chain.versions]]` selects a package array and declares `stable`. Stable versions pin **verification identity only**: comments, refactoring, and ABI renames may be refreshed if the VK is unchanged. A different stable key requires a new package and an explicitly declared chain version. Mutable versions can refresh their artifacts and pins in place. Freezing never automatically bumps chain versions. New packages may omit their pin until the first freeze, but an existing frozen stable package cannot remove its pin to bypass protection. Commit source, artifacts, and manifest together.

To check without minting or changing committed files:

```bash
cargo xtask freeze-circuits --check
# or: mise run freeze-circuits:check
```

The check requires the exact `nargo` and `bb` versions pinned in `mise.toml`.
It compiles catalog packages in a per-process scratch tree under `target/`.
L1 checks compare decoded bytecode, structural ABI, and a freshly derived VK;
L2 checks require committed and derived VKs to match the manifest pins. L2 ABI
or source spelling changes alone do not fail verification. A mismatch exits
nonzero; scratch files are cleaned up
on success or failure. The manifest, frozen artifacts, and tracked Noir compiler
outputs remain unchanged. CI uses this mode. `--check` cannot be combined with
`--abi-change` or `--semantic`.

## Isolated Groth16 Tribute experiment

`experiments/groth16-tribute` compares BN254 Groth16 with the pinned Noir/bb
backend for Demo Tribute, the current Emit mint circuit, and historical
flat-ownership capacities 1, 2, 4, 8, 16, 32, and 64. It does not restore
production aggregation APIs, change the canonical registry, or replace the
production backend. Its Groth16
keys are single-party **test setup only**, never production ceremony material.

```bash
cargo test --release -p groth16-tribute
cargo build --release -p groth16-tribute
./target/release/groth16-tribute prepare --out target/groth16-tribute
./target/release/groth16-tribute check --out target/groth16-tribute
./target/release/groth16-tribute run --out target/groth16-tribute \
  --samples 3 --results experiments/groth16-tribute/results/measurements
```

Preparation needs the pinned `nargo`/`bb` tools and Git revision
`a9fe83ed95bc9bf6335ffac4ffe0ee983cd6f821`. It recovers historical sources and
artifacts under `target/`, reproduces ACIR/ABI/VKs (VK identity for current
Demo), and creates one Groth16 keypair per shape in sequential child processes.
Emit uses the active frozen L1 release and records its version in preparation
metadata; it is not recovered from the historical flat-circuit revision.
`check` runs differential statement rejection checks and real proof round-trips.
These external-tool exercises are not automatic workspace tests.

Emit measurements use partial mints with three-limb amounts, borrowing across
both radix-`2^120` boundaries, and a real index-5 note inclusion path. Checks
also cover full mints, change sentinels, amount/address ranges and malformed
paths. The port preserves the pinned `noir-bignum v0.10.0` `[120,120,17]`-bit
limb bounds and subtraction-flag relation rather than silently strengthening
the frozen statement. Use `--case emit_mint` for an Emit-only measurement.

Stage a local SRS at `$OUTBE_BB_SRS_PATH`, or `~/.bb-crs/bn254_g1.dat`.
The experiment disables backend network downloads and checksums the existing
1,048,577-point prefix pin before initializing bb. It uses this maximum prefix,
not a circuit-minimal SRS. Large keys and recovered artifacts must stay under
`target/`.

`run` produces plain `report.md`, exact-nanosecond `timings.csv`, measured
`memory.csv`, `observations.json`, and provenance-rich `metadata.json`.
Each backend/case/sample gets a fresh process, fixture, blinded proof,
encoding round-trip and verification;
backend order alternates. Filesystem caches are unmanaged and verification is
after proving. No warmup, averages or best-of selection are used. Setup and
compilation are separate from core timings. `--threads N` sets the requested
Rayon/native thread counts; `--case demo_tribute --samples 1` selects a smoke run.
Nonempty preparation directories and existing result directories are refused;
use the first unused `-2`, `-3`, … suffix for reruns to preserve evidence.

On macOS, each setup/sample worker runs under `/usr/bin/time -l`, with its
resource output stored separately from native prover logs. The report shows
whole-process peak RSS in MiB; `memory.csv` preserves exact bytes. This includes
native allocations, keys, SRS and witness/proving data—not just the Rust heap
or serialized artifact sizes. It is not a per-stage memory profile. Sample 0
covers Groth16 setup and key writes; compiler and CLI VK-generation memory is
not measured. Total process wall time is measured independently with `Instant`,
including wrapper/process startup, log setup and shutdown. Named witness/prove/
verify stage timers are unchanged. Older preparation directories without
memory observations must be regenerated in a fresh output directory.

The completed nine-circuit comparison with process memory (54 verified proof
samples plus nine setup observations) is retained in
[`results/measurements-3/report.md`](experiments/groth16-tribute/results/measurements-3/report.md),
alongside exact `timings.csv`, `memory.csv`, and metadata. The prior
[`nine-circuit comparison`](experiments/groth16-tribute/results/measurements-2/report.md)
and original
[`eight-circuit comparison`](experiments/groth16-tribute/results/measurements/report.md)
remain unchanged.

## Ownership R&D and mobile WASM demo

`experiments/ownership-rd` runs three experiments over the same public test
fixtures: fresh-process versus resident-key Groth16, a new Grumpkin
Schnorr/Poseidon2 authorization scheme, and genuine one-ownership-step Nova
IVC with Spartan IPA compression. It reuses the canonical experimental
ownership/Poseidon2 gadgets instead of copying them. The seven capacities
1, 2, 4, 8, 16, 32, and 64 are available in the demo.

The browser additionally offers **Noir / UltraHonk (barretenberg)** for the
canonical signature. Noir is the circuit language/toolchain; ACVM solves its
witness and bb creates the proof. Nova is an IVC/folding proof system, not a
language. The experimental Poseidon2 signature remains a separate Rust circuit
and is explicitly disabled when Noir is selected.

Select **Demo Tribute (full_proof)** for the complete ownership + depth-32
Merkle-inclusion statement, rather than flat aggregation with one slot.
Its four public fields are `derived_owner`, `nft_hash`, `binding_hash`, and
`merkle_root`, in that order. Direct Groth16 supports both the canonical
signature and the same experimental Poseidon2 signature used by flat aggregation.
Both constrain the complete Merkle path; the tree uses Poseidon2 in either case.
Noir/bb and its backend wrapper remain canonical-only with their original VK.
Nova is disabled here because the ownership-only Nova step does not prove
Merkle inclusion. Switching back restores the flat-aggregation controls.

```bash
cargo test --release -p ownership-rd --lib
cargo build --release -p ownership-rd
./target/release/ownership-rd prepare --out target/ownership-rd
./target/release/ownership-rd prepare-demo --scheme poseidon2 --out target/ownership-rd-demo
./target/release/ownership-rd run --out target/ownership-rd \
  --capacities 1,8,64 --samples 3 \
  --results experiments/ownership-rd/results/measurements
cargo build --release -p groth16-tribute
./target/release/groth16-tribute prepare --out target/groth16-tribute
bash experiments/ownership-rd/build-web.sh target/ownership-rd target/ownership-rd-web target/groth16-tribute target/ownership-rd-demo
python3 -m http.server 8080 --bind 0.0.0.0 --directory target/ownership-rd-web
```

`prepare` makes fourteen Groth16 key pairs and two reusable Nova parameter sets.
The default native matrix is two engines × two signatures × three capacities
× cold/warm × three samples: **72 independent verified proofs**. Use
`--engine groth16|nova`, `--scheme canonical|poseidon2`, `--mode cold|warm`,
`--capacities 1`, or `--samples 1` for a targeted run. Native process RSS
accounting currently requires macOS. Nonempty preparation and existing result
directories are refused; choose a fresh suffix rather than replacing evidence.
`prepare-demo` creates only the requested Demo Tribute key pair(s), without
regenerating the flat/Nova matrix. It is experimental trusted setup, not the
browser's preparation button. Its output is the required fourth web-build
argument (default `target/ownership-rd-demo`); the retained Poseidon2 keys here
are in `target/ownership-rd-demo-1`.

The completed [native R&D comparison](experiments/ownership-rd/results/measurements/report.md)
contains all 72 verified runs, with raw
[`timings.csv`](experiments/ownership-rd/results/measurements/timings.csv),
[`observations.json`](experiments/ownership-rd/results/measurements/observations.json)
and provenance metadata. The retained prepared assets in this working tree are
under `target/ownership-rd-3`; the ready-to-serve bundle is
`target/ownership-rd-web`.

The [browser validation record](experiments/ownership-rd/results/browser-validation.json)
and [actual browser export](experiments/ownership-rd/results/browser-smoke.json)
cover both engines/signatures, cold/warm worker lifetime, cancellation/recovery,
asset-integrity errors, and narrow layouts. They contain 18 successful proofs
from desktop Chromium, **not physical-phone benchmark results**. Every
downloadable Groth16 key shape was additionally exercised natively.

The [Noir browser export](experiments/ownership-rd/results/noir-browser.json)
([CSV](experiments/ownership-rd/results/noir-browser.csv)) retains 15 verified
Noir proofs across all seven capacities and two existing-engine regression
proofs. The [Noir validation record](experiments/ownership-rd/results/noir-browser-validation.json)
also records wrong-public/signature/owner/proof rejection checks and
cancellation/recovery. These are desktop browser observations, not phone measurements.

The [full-proof and wrapping report](experiments/ownership-rd/results/full-proof-report.html)
retains every individual measurement: 12 local Demo Tribute proofs, six genuine
UltraHonk-to-Groth16 pipelines, and three flat-engine regression proofs.
Its [JSON](experiments/ownership-rd/results/full-proof-browser.json),
[CSV](experiments/ownership-rd/results/full-proof-browser.csv), and
[validation/provenance](experiments/ownership-rd/results/full-proof-validation.json)
include client memory capacities, cold native backend RSS, setup costs,
public-input/proof rejection and cancellation/recovery. These too are desktop
measurements, not physical-phone results.

The web build installs exact npm pins from `package-lock.json`: Noir JS/ACVM/ABI
`1.0.0-beta.22` and bb.js `5.0.0-nightly.20260522`. Its third argument is the
already-verified `groth16-tribute prepare` directory. It checks the recovered
artifact hashes and local SRS prefix, stages all browser modules/SRS locally,
and never freezes or edits canonical circuits. In this working tree the input
directories are `target/ownership-rd-3`, `target/groth16-tribute-3`, and
`target/ownership-rd-demo-1` (fourth argument).
`OUTBE_BB_SRS_PATH` selects the local G1 file; otherwise `~/.bb-crs/bn254_g1.dat`
is used. No runtime SRS download fallback is used. Proving is local unless
the explicit backend-wrapping option below is enabled.

Noir uses the stored VK for both proving and verification, with blinded
UltraHonk/Keccak settings. There is no separately stored proving key in this
API. ABI encoding, ACVM execution (including its internal program decode),
witness serialization, proof generation, and verification are timed separately.
The SRS is the same pinned maximum 1,048,577-point prefix as the native
comparison, not a circuit-minimal load. The browser reports individual
Rust-fixture, Noir ACVM, Noir ABI and BB linear-memory capacities plus their
sum; it does not confuse that sum with RSS or include JS buffers in it.

For native measurements, cold means one checked key load in each fresh process.
Warm means one checked
load followed by independent fresh proofs in the same process, with no discarded
warmup. Warm-series peak RSS is **not** a per-proof peak. Groth16 separates
witness synthesis from matrix-only proving. Nova's `folding` stage includes
step synthesis and recursive proving; `compress` measures real Spartan IPA
compression. Its final proof is **not Groth16 or EVM calldata**. Native results
retain all individual observations, stages, hashes, sizes and process memory;
there are no statistical aggregates or estimated subtractions.

Open `http://localhost:8080` on the computer. On a phone in the same trusted
Wi-Fi network, use `http://<computer-LAN-IP>:8080` (on macOS, obtain the address
with `ipconfig getifaddr en0`). Allow the server through the local firewall if
needed. The static page runs all proving locally in a module Web Worker; it
does not call a remote prover or upload witnesses. Start with one slot.
All three engines, their supported signature combinations, and cold/warm modes
are selectable; JSON/CSV exports
preserve every run and browser environment.

The browser defaults to **Warm** with an explicit **Подготовить WASM** button.
That step initializes the Worker, fetches and hashes artifacts, and decodes
PK/VK, Nova parameters, or Noir/SRS. It does not generate a proof or perform a
trusted setup. **Создать и проверить** then reuses that decoded session across
successive clicks/series; `pk_decode` is not charged again. Preparation timings
are shown separately, immediately after preparation.

Changing circuit, engine, signature or slot count invalidates preparation.
Changing sample count or the optional wrapping flag does not. **Освободить память**
terminates the retained Worker; cancellation, errors and page navigation also
invalidate it. Browser reclamation of physical memory is asynchronous.
**Cold benchmark** remains available and intentionally includes a fresh
preparation for each proof; its separate preparation button is disabled.
These controls change client WASM lifetime only. The native backend initializes
once at server startup and is not reset by client Cold/Warm modes.

Current browser JSON exports use `format: 3`: client records have
`kind: "preparation" | "proof"`, and warm proof records reference
`preparation_id`. Client key-load timings appear only in that preparation record.
`backend_initializations` stores each server startup once by its `id`, with
startup timings and setup provenance; wrapper runs/requests reference
`initialization_id` rather than repeating startup costs. CSV has separately scoped
`backend_initialization` rows. Historical format-1 and format-2 records remain
unchanged.
The [preparation/Poseidon2 browser record](experiments/ownership-rd/results/preparation-poseidon-browser.json)
and [CSV](experiments/ownership-rd/results/preparation-poseidon-browser.csv)
retain real proofs and preparation/error/cancellation observations;
[validation](experiments/ownership-rd/results/preparation-poseidon-validation.json)
records reuse, invalidation and recovery checks. These are desktop browser
checks with a mobile viewport, not physical-phone benchmarks.

The browser build uses serial arkworks; Nova's Rayon operations use its
single-thread global fallback. Native parallel features remain enabled.
This avoids ark-ec's explicit native thread-pool creation, which traps in
ordinary WASM. No SharedArrayBuffer, atomics, or cross-origin-isolation headers
are required. Artifact SHA-256 uses the same Rust implementation as native;
proof entropy still uses browser `crypto.getRandomValues`. This permits LAN
HTTP for these **public fixtures only**. HTTP cannot authenticate the code or
manifest: use HTTPS on an untrusted network, or if a browser blocks its APIs.
Never enter production secrets; the app has no private-key input.

Browser memory readings are WASM linear-memory capacity before/after operations,
**not RSS, live Rust heap, total browser RAM, or physical peak memory**. Mobile
tabs can terminate below the WASM address-space limit; large capacities are not
guaranteed to fit. Cancellation terminates the Worker and preserves completed
observations. A desktop browser with a mobile viewport is a UI smoke check,
not a measurement of phone performance.

### UltraHonk → backend Groth16

`experiments/ultrahonk-wrap` is an isolated Go/gnark port of the **exact**
BB `5.0.0-nightly.20260522` UltraKeccakZK verifier for the current Demo Tribute
VK. It constrains Keccak Fiat–Shamir, all 29 relations, 16 sumcheck rounds,
Libra/Gemini/Shplonk and the BN254 KZG pairing inside BN254 Groth16.
There is no native-verifier Boolean substituted for these constraints.
The fixed inner VK and the original four public fields define the outer
statement; the UltraHonk proof is the backend's private witness.

From the repository root, after building the WASM bundle:

```bash
mise install go@1.27.2
mise exec go@1.27.2 -- go -C experiments/ultrahonk-wrap build -o ../../target/ultrahonk-wrap .
GOMEMLIMIT=16GiB GOMAXPROCS=12 ./target/ultrahonk-wrap setup --out target/ultrahonk-wrap-keys
GOMEMLIMIT=16GiB GOMAXPROCS=12 ./target/ultrahonk-wrap serve \
  --keys target/ultrahonk-wrap-keys --web target/ownership-rd-web --listen 0.0.0.0:8082
```

Open port **8082**, choose **Demo Tribute (full_proof)** and **Noir / UltraHonk**,
then explicitly enable backend wrapping. Default mode remains entirely local.
The client generates and verifies UltraHonk in its Worker, uploads only the
proof and four public inputs, then the backend creates and verifies Groth16.
A separate verification call checks the serialized outer proof with the resident
VK; this is **server-side verification in the same process, not a WASM verifier
or a separate cold process**.
Private ownership witnesses and signing keys never enter the request.
The plain Python static server still supports local modes but has no wrapper API.
The retained setup in this working tree is `target/ultrahonk-wrap-1`; pass that
directory to `serve` to reuse it without another setup. The service binds IPv4
explicitly and refuses an occupied wildcard IPv4 port.

At startup the service checks setup provenance and SHA-256, decodes R1CS/PK/VK
with checked, buffered readers, and only then announces readiness and serves HTTP.
It keeps these artifacts in memory until shutdown; there is no artifact-file access,
key decoding, setup, or subprocess launch on the proof/verification path.
Invalid startup artifacts prevent the server from becoming available.

`GET /api/wrapper` and successful responses include an immutable `initialization`
snapshot: instance ID, timestamps, phase timings (including
`load_proving_artifacts`), startup wall time and the process-lifetime RSS peak
observed at startup completion. The browser displays it immediately when wrapping
is enabled, even before a client proof. Request `timings_ms` excludes all startup
stages; `request_wall_ms` measures only that request. Result
`process_lifetime_peak_rss_bytes` is a high-water mark for the entire resident
process, **not a per-proof peak or an incremental memory cost**. Darwin/Linux
units are normalized to bytes; browser WASM capacities remain separate.

Only one native wrap/verify runs at a time; concurrent requests receive 409.
Canceling the browser request stops waiting and releases its local Worker, but
the pinned gnark prover cannot interrupt an already-started native computation.
The server remains busy until that computation finishes, discards the disconnected
result and retains its initialized keys. No automatic retry is performed.
Restart the server to measure a new initialization or release its retained memory.

The [resident browser export](experiments/ownership-rd/results/resident-backend-browser.json)
and [CSV](experiments/ownership-rd/results/resident-backend-browser.csv) contain
one startup and three real client/wrapper proofs. Additional
[native HTTP observations](experiments/ownership-rd/results/resident-backend-native.json)
prove reuse with the configured startup-key path temporarily unavailable
(only a removable symlink was hidden; original keys were untouched).
[Validation](experiments/ownership-rd/results/resident-backend-validation.json)
records tampering rejection, cancellation/admission recovery and startup metric
deduplication. Earlier cold-backend measurements are preserved as historical
baselines, not current request-latency claims.

The CLI also accepts the same request JSON on stdin with `prove --keys DIR`
and the resulting outer proof with `verify --keys DIR`. A standalone invocation
initializes once for that invocation and reports initialization separately;
verify-only loads only the VK. Repeated HTTP requests instead reuse the server
session and its stable initialization ID.
The outer encoding is gnark BN254 `WriteTo`, including its commitment extensions
and commitment proof of knowledge; it is **not** assumed to be a 128-byte
arkworks proof or EVM calldata. Setup uses OS randomness, is single-party and
unaudited, and refuses to overwrite an existing directory. Source/artifact
hashes and protocol pins are retained in `setup.json` and every result.
Only this pinned Demo Tribute VK is supported; other circuits or BB formats
require a different wrapper/setup, not an untrusted request-supplied VK.

The self-contained genuine BB proof fixture in
`experiments/ultrahonk-wrap/testdata/demo-input.json` was generated and verified
by the browser. Run the constraint/parser/service checks with
`mise exec go@1.27.2 -- go -C experiments/ultrahonk-wrap test ./...`.
To additionally exercise a generated outer proof, set
`ULTRAHONK_WRAP_KEYS` and `ULTRAHONK_WRAP_RESULT` to absolute paths before that
command. The optional outer-proof test explicitly skips without those files.
The current fixture has the ordinary all-zero recursive accumulator; it does
not establish test coverage of valid non-default recursive accumulators or
constitute a cryptographic audit.

### R&D security boundary

The experimental signature retains the owner formula and uses the challenge
`h0 = H3(domain, pk.x, pk.y)`, `h1 = H3(h0, R.x, R.y)`,
`e = H3(h1, signing_payload, 6)` on Grumpkin, with canonical nonzero `s` and `e`.
It is a **new, unaudited protocol**, incompatible with canonical signatures.
Its nonce relies on the supplied cryptographic RNG and does not inherit the
canonical signer's deterministic auxiliary-entropy hedge. Fixture keys and
signing seeds are publicly known; setup and proof blinding use OS/browser
entropy. Groth16 keys are single-party research setup, never ceremony outputs.

The Nova step constrains the actual ownership equations via an arkworks-to-Nova
R1CS bridge. Its state commits the ordered owner/NFT list, count and binding.
There is no spent-nullifier tracking, ledger state transition, or production
rollup admission policy. Nova parameters require trusted provenance.
Dependency-policy findings from `cargo deny` remain visible rather than being
silenced: the Nova/arkworks/Noir dependency versions need reconciliation before
production adoption.

## License

MIT for the main workspace. See [`LICENSE`](LICENSE).
The Aztec-derived UltraHonk verifier port in `experiments/ultrahonk-wrap`
retains its Apache-2.0 headers and [license](experiments/ultrahonk-wrap/LICENSE).
