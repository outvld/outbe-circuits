import { acvm, abi } from "@noir-lang/noir_js";
import { ungzip } from "pako";
import { BarretenbergWasmMain } from "../node_modules/@aztec/bb.js/dest/browser/barretenberg_wasm/barretenberg_wasm_main/index.js";
import { AsyncApi } from "../node_modules/@aztec/bb.js/dest/browser/cbind/generated/async.js";
import { BBApiException } from "../node_modules/@aztec/bb.js/dest/browser/bbapi_exception.js";

const VERSIONS = Object.freeze({ noir: "1.0.0-beta.22", bb: "5.0.0-nightly.20260522" });
const SETTINGS = Object.freeze({
  ipaAccumulation: false, oracleHashType: "keccak", disableZk: false,
  optimizedSolidityVerifier: false,
});
const SRS_POINTS = 1048577;
const NOTES = [
  "Timings use the Worker performance clock, in nanoseconds rounded from milliseconds.",
  "witness includes ACVM program deserialization and solving, not predecoded pure solving.",
  "public_inputs_check includes ACVM public-witness extraction and comparison with the Rust fixture.",
  "witness_encode includes ACVM witness-stack compression followed by gzip decompression to BB's MessagePack input.",
  "prove includes BB circuit/witness decoding and proving using the stored VK; there is no stored proving key.",
  "proof_encode/proof_decode roundtrip only proof fields; public fields are separate and not counted twice.",
  "verify includes BB decoding the stored VK; no VK is recomputed by this module.",
  "Memory measurements are four WASM linear-memory capacities, not JS buffers or process RSS.",
];
class VerifierInputError extends Error {}
const elapsed = start => Math.round((performance.now() - start) * 1e6);
function timed(rows, stage, fn) {
  const start = performance.now();
  const result = fn();
  rows.push({ stage, elapsed_ns: elapsed(start) });
  return result;
}
async function timedAsync(rows, stage, fn) {
  const start = performance.now();
  const result = await fn();
  rows.push({ stage, elapsed_ns: elapsed(start) });
  return result;
}
function requireValue(condition, message) {
  if (!condition) throw new Error(message);
}
function field(value) {
  requireValue(typeof value === "string" && /^0x[0-9a-f]{1,64}$/i.test(value), "Invalid field encoding");
  return value.slice(2).toLowerCase().padStart(64, "0");
}
function fieldBytes(value) {
  const hex = field(value);
  return Uint8Array.from({ length: 32 }, (_, i) => parseInt(hex.slice(i * 2, i * 2 + 2), 16));
}
function bytesHex(value) {
  requireValue(value instanceof Uint8Array && value.length === 32, "BB field is not 32 bytes");
  return Array.from(value, byte => byte.toString(16).padStart(2, "0")).join("");
}
function encodeFields(fields) {
  const bytes = new Uint8Array(fields.length * 32);
  fields.forEach((value, i) => {
    requireValue(value instanceof Uint8Array && value.length === 32, "Invalid proof field length");
    bytes.set(value, i * 32);
  });
  return bytes;
}
function decodeFields(bytes) {
  requireValue(bytes.length % 32 === 0, "Proof bytes are not field aligned");
  return Array.from({ length: bytes.length / 32 }, (_, i) => bytes.slice(i * 32, (i + 1) * 32));
}
function comparePublic(actual, expected, label) {
  requireValue(actual.length === expected.length, `${label}: public input arity mismatch`);
  actual.forEach((value, i) => requireValue(value === field(expected[i]), `${label}: public input ${i} differs from Rust fixture`));
}
function changeField(value) {
  return `0x${(BigInt(value) === 0n ? "1" : "0").padStart(64, "0")}`;
}
async function noOracle(name) {
  throw new Error(`Unexpected foreign call in recovered ownership circuit: ${name}`);
}
function assertNotTrap(error) {
  for (let cause = error; cause; cause = cause.cause) {
    if (cause instanceof WebAssembly.RuntimeError || /\b(panic|unreachable|out of bounds|aborted|proc_exit)\b/i.test(String(cause.message || cause))) {
      throw error;
    }
  }
}

export class NoirSession {
  static async create({ circuit, capacity, wasm, buffers }) {
    requireValue(["flat_aggregation", "demo_tribute"].includes(circuit), "Unsupported Noir circuit");
    requireValue(circuit === "demo_tribute" ? capacity === 1 : [1, 2, 4, 8, 16, 32, 64].includes(capacity), "Unsupported Noir capacity");
    const session = new NoirSession();
    session.capacity = capacity;
    session.circuitKind = circuit;
    session.publicInputCount = circuit === "demo_tribute" ? 4 : 2 * capacity + 1;
    session.wasm = wasm;
    const timings = [];
    const { program, metadata } = timed(timings, "artifact_decode", () => {
      const decoder = new TextDecoder("utf-8", { fatal: true });
      return {
        program: JSON.parse(decoder.decode(buffers.program)),
        metadata: JSON.parse(decoder.decode(buffers.metadata)),
      };
    });
    requireValue(program.noir_version === VERSIONS.noir && metadata.versions?.noir === VERSIONS.noir && metadata.versions?.bb === VERSIONS.bb, "Noir/BB artifact version mismatch");
    const expectedName = circuit === "demo_tribute" ? "demo_tribute" : `flat_aggregation_n${capacity}`;
    requireValue(metadata.statement === circuit && metadata.capacity === capacity && metadata.circuit === expectedName
      && metadata.public_input_count === session.publicInputCount, "Noir artifact statement/arity mismatch");
    if (circuit === "demo_tribute") requireValue(metadata.merkle_depth === 32, "Demo Tribute requires depth 32");
    requireValue(metadata.srs_points === SRS_POINTS && buffers.g1.length === SRS_POINTS * 64 && buffers.g2.length === 128, "Invalid local SRS dimensions");
    requireValue(buffers.vk.length > 0, "Stored verification key is empty");
    session.program = program;
    session.metadata = metadata;
    session.vk = buffers.vk;
    session.compressedAcir = timed(timings, "bytecode_base64_decode", () => Uint8Array.from(atob(program.bytecode), character => character.charCodeAt(0)));
    const acirBytes = timed(timings, "bytecode_gzip_decode", () => ungzip(session.compressedAcir));
    session.acvmMemory = (await timedAsync(timings, "acvm_wasm_init", () => acvm.default({ module_or_path: buffers.acvm_wasm }))).memory;
    session.abiMemory = (await timedAsync(timings, "abi_wasm_init", () => abi.default({ module_or_path: buffers.abi_wasm }))).memory;
    const bbBytes = timed(timings, "bb_wasm_gzip_decode", () => ungzip(buffers.bb_wasm));
    const bbModule = await timedAsync(timings, "bb_wasm_compile", () => WebAssembly.compile(bbBytes));
    session.bbWasm = new BarretenbergWasmMain();
    await timedAsync(timings, "bb_wasm_init", () => session.bbWasm.init(bbModule, 1));
    session.api = new AsyncApi({
      call: bytes => session.bbWasm.cbindCall("bbapi", bytes),
      destroy: () => session.bbWasm.destroy(),
    });
    session.circuit = { name: metadata.circuit, bytecode: acirBytes, verificationKey: session.vk };
    session.stats = await timedAsync(timings, "circuit_stats", () => session.api.circuitStats({
      circuit: session.circuit, includeGatesPerOpcode: false, settings: SETTINGS,
    }));
    requireValue(session.stats.numGates === metadata.gates && session.stats.numGatesDyadic === metadata.dyadic_size, "Recovered circuit statistics differ from metadata");
    requireValue(session.stats.numGatesDyadic + 1 <= SRS_POINTS, "Local SRS is too small for circuit");
    await timedAsync(timings, "srs_init", () => session.api.srsInitSrs({
      pointsBuf: buffers.g1, numPoints: SRS_POINTS, g2Point: buffers.g2,
    }));
    session.load = {
      engine: "noir_bb", circuit, scheme: "canonical", capacity, timings,
      versions: VERSIONS, settings: SETTINGS, provenance: metadata,
      sizes: { pk_bytes: null, vk_bytes: session.vk.length, program_bytes: buffers.program.length,
        g1_bytes: buffers.g1.length, g2_bytes: buffers.g2.length,
        gates: session.stats.numGates, dyadic_size: session.stats.numGatesDyadic },
      notes: NOTES,
    };
    return session;
  }

  load_report() { return structuredClone(this.load); }

  memory_components() {
    const result = {
      rust_fixture_wasm_bytes: this.wasm.memory_bytes(),
      noir_acvm_wasm_bytes: this.acvmMemory.buffer.byteLength,
      noir_abi_wasm_bytes: this.abiMemory.buffer.byteLength,
      bb_wasm_bytes: this.bbWasm.getMemory().byteLength,
    };
    for (const value of Object.values(result)) requireValue(Number.isSafeInteger(value) && value > 0, "Invalid WASM memory capacity");
    return result;
  }

  fixture(sample) {
    requireValue(Number.isSafeInteger(sample) && sample >= 1 && sample <= 0xffffffff, "Sample must be a positive u32");
    const fixture = JSON.parse(this.wasm.noir_fixture(this.circuitKind, this.capacity, sample));
    requireValue(fixture.circuit === this.circuitKind && fixture.public_inputs.length === this.publicInputCount, "Rust fixture has unexpected statement/public input count");
    const inputPublics = this.circuitKind === "demo_tribute"
      ? [fixture.inputs.derived_owner, fixture.inputs.nft_hash, fixture.inputs.binding_hash, fixture.inputs.merkle_root]
      : fixture.inputs.public_inputs;
    comparePublic(inputPublics.map(field), fixture.public_inputs, "Rust ABI fixture");
    return fixture;
  }

  async proveSample(sample) {
    const timings = [];
    const fixture = timed(timings, "fixture", () => this.fixture(sample));
    const initial = timed(timings, "abi_encode", () => abi.abiEncode(this.program.abi, fixture.inputs));
    const solved = await timedAsync(timings, "witness", () => acvm.executeProgram(this.compressedAcir, initial, noOracle));
    timed(timings, "public_inputs_check", () => {
      requireValue(solved[0]?.index === 0, "ACVM did not return the main witness");
      const publics = acvm.getPublicWitness(this.compressedAcir, solved[0].witness);
      const values = [...publics.entries()].sort(([a], [b]) => a - b).map(([, value]) => field(value));
      comparePublic(values, fixture.public_inputs, "ACVM");
    });
    const witness = timed(timings, "witness_encode", () => ungzip(acvm.compressWitnessStack(solved)));
    const response = await timedAsync(timings, "prove", () => this.api.circuitProve({
      circuit: this.circuit, witness, settings: SETTINGS,
    }));
    timed(timings, "bb_public_inputs_check", () => comparePublic(response.publicInputs.map(bytesHex), fixture.public_inputs, "BB"));
    const encoded = timed(timings, "proof_encode", () => encodeFields(response.proof));
    const proof = timed(timings, "proof_decode", () => decodeFields(encoded));
    const publicInputs = response.publicInputs;
    const verified = await timedAsync(timings, "verify", async () => (await this.verify(proof, publicInputs)).verified);
    requireValue(verified === true, "BB rejected its ownership proof with stored VK");
    this.proofFieldCount = proof.length;
    return {
      proof, publicInputs, encoded,
      report: {
        engine: "noir_bb", circuit: this.circuitKind, scheme: "canonical", capacity: this.capacity, sample, verified,
        statement_digest: fixture.statement_digest, witness_digest: fixture.witness_digest, timings,
        versions: VERSIONS, settings: SETTINGS, provenance: this.metadata,
        sizes: { proof_bytes: encoded.length, public_bytes: publicInputs.length * 32,
          vk_bytes: this.vk.length, pk_bytes: null, witness_msgpack_bytes: witness.length,
          gates: this.stats.numGates, dyadic_size: this.stats.numGatesDyadic },
        notes: NOTES,
      },
    };
  }

  verify(proof, publicInputs) {
    // The pinned BB WASM verifier asserts on malformed arity. Reject malformed
    // transport inputs before FFI; same-arity tampering still reaches verification.
    if (publicInputs.length !== this.publicInputCount
        || publicInputs.some(value => !(value instanceof Uint8Array) || value.length !== 32)) {
      throw new VerifierInputError("Noir verifier public-input arity/field width mismatch");
    }
    if (!proof.length || proof.some(value => !(value instanceof Uint8Array) || value.length !== 32)
        || (this.proofFieldCount !== undefined && proof.length !== this.proofFieldCount)) {
      throw new VerifierInputError("Noir verifier proof length/field width mismatch");
    }
    return this.api.circuitVerify({ verificationKey: this.vk, publicInputs, proof, settings: SETTINGS });
  }

  async run(sample) { return (await this.proveSample(sample)).report; }

  async package_proof(sample) {
    const { encoded, publicInputs, report } = await this.proveSample(sample);
    const request = timed(report.timings, "wrapper_payload_encode", () => ({
      circuit: this.circuitKind,
      proof_hex: `0x${Array.from(encoded, byte => byte.toString(16).padStart(2, "0")).join("")}`,
      public_inputs: publicInputs.map(value => `0x${bytesHex(value)}`),
      inner_vk_sha256: this.metadata.source_artifact_sha256.vk,
    }));
    return { request, report };
  }

  async check(sample = 1) {
    const { proof, publicInputs, report } = await this.proveSample(sample);
    const checks = [{ name: "valid_proof", rejected: false, verified: true }];
    const expectVerifierRejection = async (name, candidateProof, candidatePublics) => {
      try {
        const result = await this.verify(candidateProof, candidatePublics);
        requireValue(result.verified === false, `${name}: verifier accepted invalid statement/proof`);
        checks.push({ name, rejected: true, outcome: "verified_false" });
      } catch (error) {
        assertNotTrap(error);
        if (error instanceof VerifierInputError) {
          checks.push({ name, rejected: true, outcome: "input_boundary_rejection", message: error.message });
          return;
        }
        // Only normal BB ErrorResponse exceptions qualify, not WASM aborts or arbitrary JS errors.
        if (!(error instanceof BBApiException)) throw error;
        checks.push({ name, rejected: true, outcome: "bb_error_response", message: error.message });
      }
    };
    for (let i = 0; i < publicInputs.length; i++) {
      const changed = publicInputs.slice();
      changed[i] = fieldBytes(changeField(`0x${bytesHex(publicInputs[i])}`));
      await expectVerifierRejection(`changed_public_${i}`, proof, changed);
    }
    await expectVerifierRejection("public_arity_short", proof, publicInputs.slice(0, -1));
    await expectVerifierRejection("public_arity_long", proof, [...publicInputs, new Uint8Array(32)]);
    await expectVerifierRejection("truncated_proof", proof.slice(0, -1), publicInputs);
    const damaged = proof.map(value => value.slice());
    requireValue(damaged.length > 0, "Empty BB proof");
    damaged[0][31] ^= 1;
    await expectVerifierRejection("damaged_proof", damaged, publicInputs);
    const expectWitnessRejection = async (name, mutate) => {
      const fixture = this.fixture(sample);
      mutate(fixture.inputs);
      // ABI encoding must succeed: this tests the circuit, not input-schema validation.
      const initial = abi.abiEncode(this.program.abi, fixture.inputs);
      try {
        await acvm.executeProgram(this.compressedAcir, initial, noOracle);
      } catch (error) {
        assertNotTrap(error);
        if (!(error instanceof Error) || !("acirFunctionId" in error || "rawAssertionPayload" in error || "callStack" in error)) throw error;
        checks.push({ name, rejected: true, outcome: "acvm_constraint_failure", message: error.message });
        return;
      }
      throw new Error(`${name}: ACVM accepted invalid ownership witness`);
    };
    if (this.circuitKind === "demo_tribute") {
      await expectWitnessRejection("bad_signature", inputs => { inputs.signature = Array(64).fill(0); });
      await expectWitnessRejection("bad_owner", inputs => { inputs.derived_owner = changeField(inputs.derived_owner); });
      await expectWitnessRejection("bad_merkle_root", inputs => { inputs.merkle_root = changeField(inputs.merkle_root); });
      await expectWitnessRejection("bad_merkle_sibling", inputs => { inputs.merkle_path_siblings[0] = changeField(inputs.merkle_path_siblings[0]); });
      await expectWitnessRejection("bad_merkle_direction", inputs => { inputs.merkle_path_indices[0] = !inputs.merkle_path_indices[0]; });
    } else {
      await expectWitnessRejection("bad_signature", inputs => { inputs.signature[0] = Array(64).fill(0); });
      await expectWitnessRejection("bad_owner", inputs => { inputs.public_inputs[0] = changeField(inputs.public_inputs[0]); });
    }
    // Ensure failed verifier inputs did not poison the reusable instance.
    requireValue((await this.verify(proof, publicInputs)).verified === true, "Valid proof failed after rejection checks");
    checks.push({ name: "valid_after_rejections", rejected: false, verified: true });
    return { engine: "noir_bb", circuit: this.circuitKind, scheme: "canonical", capacity: this.capacity, sample,
      passed: true, checks, statement_digest: report.statement_digest, witness_digest: report.witness_digest };
  }
}
