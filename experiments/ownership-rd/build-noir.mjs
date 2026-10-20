import { readFile, writeFile, mkdir, open } from "node:fs/promises";
import { createHash } from "node:crypto";
import { dirname, resolve, join, relative, sep } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { homedir } from "node:os";
import { build } from "esbuild";

const HERE = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(HERE, "../..");
const PREPARED = resolve(ROOT, process.argv[2] || "target/groth16-tribute");
const DIST = resolve(ROOT, process.argv[3] || "target/ownership-rd-web");
const DEMO_PREPARED = resolve(ROOT, process.argv[4] || "target/ownership-rd-demo");
const REVISION = "a9fe83ed95bc9bf6335ffac4ffe0ee983cd6f821";
const VERSIONS = { noir: "1.0.0-beta.22", bb: "5.0.0-nightly.20260522" };
const SRS_POINTS = 1048577;
// Exact maximum-prefix pin and G2 from outbe-zk-backend/src/barretenberg/srs.rs.
const G1_SHA256 = "0f238856e55722f15a4d64ef0de12b4260e218245590bd3a6c900aee188de8e5";
const G2 = Buffer.from([
  1,24,196,213,184,55,188,194,188,137,181,179,152,181,151,78,159,89,68,7,59,
  50,7,139,126,35,31,236,147,136,131,176,38,14,1,178,81,246,241,199,231,255,
  78,88,7,145,222,232,234,81,216,122,53,142,3,139,78,254,48,250,192,147,131,
  193,34,254,189,163,192,192,99,42,86,71,91,66,20,229,97,94,17,230,221,63,
  150,230,206,162,133,74,135,212,218,204,94,85,4,252,99,105,247,17,15,227,
  210,81,86,193,187,154,114,133,156,242,160,70,65,249,155,164,238,65,60,128,
  218,106,95,228,
]);
const hash = bytes => createHash("sha256").update(bytes).digest("hex");
const jsonBytes = value => Buffer.from(JSON.stringify(value, null, 2));
async function json(path) { return JSON.parse(await readFile(path, "utf8")); }
function requireValue(condition, message) { if (!condition) throw new Error(message); }
async function put(path, bytes) {
  const destination = join(DIST, path);
  await mkdir(dirname(destination), { recursive: true });
  await writeFile(destination, bytes);
  return { path, bytes: bytes.length, sha256: hash(bytes) };
}
async function recover(preparation, path) {
  const bytes = await readFile(join(PREPARED, path));
  requireValue(bytes.length > 0, `Prepared artifact is empty: ${path}`);
  requireValue(preparation.sha256?.[path] === hash(bytes), `Prepared artifact SHA-256 mismatch: ${path}`);
  return bytes;
}
async function recoverDemo(descriptor, name) {
  requireValue(typeof descriptor?.path === "string" && descriptor.path.startsWith("assets/"), `Missing Demo ${name}; run ownership-rd prepare-demo --scheme poseidon2 --out ${relative(ROOT, DEMO_PREPARED)}`);
  const path = descriptor.path.slice("assets/".length);
  requireValue(path.split(/[\\/]/).every(part => part && part !== "." && part !== "..") && !path.includes(":"), `Unsafe Demo artifact path: ${path}`);
  const bytes = await readFile(join(DEMO_PREPARED, path));
  requireValue(bytes.length > 0 && descriptor.bytes === bytes.length && descriptor.sha256 === hash(bytes), `Demo artifact length/SHA-256 mismatch: ${path}`);
  return bytes;
}
async function bbCode(multithreaded) {
  const name = multithreaded ? "barretenberg-threads.js" : "barretenberg.js";
  const path = join(HERE, "node_modules/@aztec/bb.js/dest/browser/barretenberg_wasm/fetch_code/browser", name);
  const uri = (await import(pathToFileURL(path).href)).default;
  requireValue(typeof uri === "string" && uri.startsWith("data:application/gzip;base64,"), `Unexpected pinned BB WASM packaging: ${path}`);
  const bytes = Buffer.from(uri.slice(uri.indexOf(",") + 1), "base64");
  requireValue(bytes[0] === 0x1f && bytes[1] === 0x8b && bytes[2] === 8, "Invalid BB gzip module");
  return bytes;
}

requireValue(DIST.startsWith(resolve(ROOT, "target") + sep), "Noir browser assets must remain under target/");
const preparation = await json(join(PREPARED, "preparation.json"));
requireValue(preparation.historical_revision === REVISION, "Wrong historical preparation revision");
requireValue(preparation.versions?.bb === VERSIONS.bb && preparation.versions?.nargo?.includes(`nargo version = ${VERSIONS.noir}`), "Preparation tool versions do not match browser pins");
for (const [name, version] of [["@noir-lang/noir_js", VERSIONS.noir], ["@noir-lang/acvm_js", VERSIONS.noir], ["@noir-lang/noirc_abi", VERSIONS.noir], ["@aztec/bb.js", VERSIONS.bb]]) {
  requireValue((await json(join(HERE, "node_modules", name, "package.json"))).version === version, `Installed package version mismatch: ${name}`);
}
const manifest = await json(join(DIST, "assets/manifest.json"));
requireValue(manifest.format === 1 && manifest.groth16 && manifest.nova, "Missing prepared R&D web manifest");
let demoPreparationBytes;
try { demoPreparationBytes = await readFile(join(DEMO_PREPARED, "manifest.json")); }
catch (error) { throw new Error(`Missing Demo preparation at ${DEMO_PREPARED}; run ownership-rd prepare-demo --scheme poseidon2 --out ${relative(ROOT, DEMO_PREPARED)}`, { cause: error }); }
const demoPreparation = JSON.parse(demoPreparationBytes);
requireValue(demoPreparation.format === 1 && demoPreparation.circuit === "demo_tribute"
  && demoPreparation.capacity === 1 && demoPreparation.merkle_depth === 32
  && demoPreparation.public_input_count === 4, "Invalid Demo preparation shape");
const demoReport = demoPreparation.preparation?.find(entry => entry.scheme === "poseidon2")?.report;
requireValue(demoReport?.circuit === "demo_tribute" && demoReport.scheme === "poseidon2"
  && demoReport.capacity === 1 && demoReport.single_party_test_setup === true
  && demoReport.sizes?.instance_variables === 5, "Missing/invalid Poseidon2 Demo preparation report");
const demoPk = await recoverDemo(demoPreparation.groth16?.poseidon2?.pk, "Poseidon2 PK");
const demoVk = await recoverDemo(demoPreparation.groth16?.poseidon2?.vk, "Poseidon2 VK");
requireValue(demoReport.sizes.pk_bytes === demoPk.length && demoReport.sizes.vk_bytes === demoVk.length, "Demo key sizes differ from preparation report");
const srsPath = process.env.OUTBE_BB_SRS_PATH || join(homedir(), ".bb-crs/bn254_g1.dat");
const g1 = Buffer.alloc(SRS_POINTS * 64);
const file = await open(srsPath, "r");
try {
  let offset = 0;
  while (offset < g1.length) {
    const { bytesRead } = await file.read(g1, offset, g1.length - offset, offset);
    requireValue(bytesRead > 0, `Local SRS too short: ${srsPath}`);
    offset += bytesRead;
  }
} finally { await file.close(); }
requireValue(hash(g1) === G1_SHA256, `Pinned SRS checksum mismatch: ${srsPath}`);
requireValue(G2.length === 128, "Invalid fixed G2 encoding");

manifest.noir_runtime = {
  acvm_wasm: await put("assets/noir/runtime/acvm_js_bg.wasm", await readFile(join(HERE, "node_modules/@noir-lang/acvm_js/web/acvm_js_bg.wasm"))),
  abi_wasm: await put("assets/noir/runtime/noirc_abi_wasm_bg.wasm", await readFile(join(HERE, "node_modules/@noir-lang/noirc_abi/web/noirc_abi_wasm_bg.wasm"))),
  bb_wasm: await put("assets/noir/runtime/barretenberg.wasm.gz", await bbCode(false)),
  bb_threads_wasm: await put("assets/noir/runtime/barretenberg-threads.wasm.gz", await bbCode(true)),
  g1: await put("assets/noir/srs/bn254_g1.dat", g1),
  g2: await put("assets/noir/srs/bn254_g2.dat", G2),
};
manifest.noir = { canonical: {} };
const circuitCases = [
  ...[1, 2, 4, 8, 16, 32, 64].map(capacity => ({ statement: "flat_aggregation", circuit: `flat_aggregation_n${capacity}`, capacity })),
  { statement: "demo_tribute", circuit: "demo_tribute", capacity: 1 },
];
for (const { statement, circuit, capacity } of circuitCases) {
  const abiBytes = await recover(preparation, `artifacts/${circuit}/abi.json`);
  const bytecode = await recover(preparation, `artifacts/${circuit}/bytecode.b64`);
  const vk = await recover(preparation, `artifacts/${circuit}/circuit.vk`);
  const abi = JSON.parse(abiBytes.toString("utf8"));
  const demo = statement === "demo_tribute";
  if (demo) {
    requireValue(abi.parameters.map(p => p.name).join(",") === "pk,signature,nonce,merkle_path_siblings,merkle_path_indices,derived_owner,nft_hash,binding_hash,merkle_root", "Demo Tribute ABI differs from full-proof layout");
    requireValue(abi.parameters[3].type.length === 32 && abi.parameters[4].type.length === 32, "Demo Tribute Merkle depth differs");
  } else {
    requireValue(abi.parameters.map(p => p.name).join(",") === "pk,signature,nonce,public_inputs", `${circuit}: unexpected ABI order`);
    requireValue(abi.parameters.slice(0, 3).every(p => p.type.kind === "array" && p.type.length === capacity)
      && abi.parameters[3].type.length === 2 * capacity + 1, `${circuit}: unexpected ABI capacity`);
  }
  const stats = preparation.stats[circuit];
  requireValue(Number.isInteger(stats?.gates) && Number.isInteger(stats?.dyadic_size) && stats.dyadic_size + 1 <= SRS_POINTS, `${circuit}: invalid preparation stats/SRS requirement`);
  const prefix = `assets/noir/${circuit}`;
  const program = { noir_version: VERSIONS.noir, abi, bytecode: bytecode.toString("utf8").trim(), debug_symbols: "", file_map: {} };
  const metadata = {
    circuit, statement, capacity, public_input_count: demo ? 4 : 2 * capacity + 1,
    historical_revision: demo ? null : REVISION,
    source_kind: demo ? "current_l2_demo_snapshot" : "historical_flat_ownership",
    merkle_depth: demo ? 32 : null,
    versions: VERSIONS,
    srs_points: SRS_POINTS, srs_policy: "same maximum pinned prefix as native comparison; no runtime SRS download fallback",
    g1_sha256: G1_SHA256, gates: stats.gates, dyadic_size: stats.dyadic_size,
    source_artifact_sha256: { abi: hash(abiBytes), bytecode: hash(bytecode), vk: hash(vk) },
    preparation_sha256: hash(await readFile(join(PREPARED, "preparation.json"))),
    package_lock_sha256: hash(await readFile(join(HERE, "package-lock.json"))),
  };
  const noirAssets = {
    program: await put(`${prefix}/program.json`, jsonBytes(program)),
    vk: await put(`${prefix}/circuit.vk`, vk),
    metadata: await put(`${prefix}/metadata.json`, jsonBytes(metadata)),
  };
  if (demo) {
    const pk = await recover(preparation, "keys/demo_tribute/pk.bin");
    const grothVk = await recover(preparation, "keys/demo_tribute/vk.bin");
    manifest.demo_tribute = {
      label: "Demo Tribute (full_proof)", capacity: 1,
      merkle_depth: 32, public_input_count: 4,
      groth16: {
        canonical: {
          pk: await put("assets/demo_tribute/groth16/canonical/pk.bin", pk),
          vk: await put("assets/demo_tribute/groth16/canonical/vk.bin", grothVk),
        },
        poseidon2: {
          pk: await put("assets/demo_tribute/groth16/poseidon2/pk.bin", demoPk),
          vk: await put("assets/demo_tribute/groth16/poseidon2/vk.bin", demoVk),
        },
      },
      noir_bb: noirAssets,
      groth16_key_source: {
        canonical: { preparation_sha256: metadata.preparation_sha256, pk_sha256: hash(pk), vk_sha256: hash(grothVk), single_party_test_setup: true },
        poseidon2: {
          preparation_sha256: hash(demoPreparationBytes), prepared_directory: relative(ROOT, DEMO_PREPARED),
          pk_sha256: hash(demoPk), vk_sha256: hash(demoVk), single_party_test_setup: true,
          setup_randomness: demoPreparation.setup_randomness, source_sha256: demoPreparation.source_sha256,
          warning: "Unaudited experimental Poseidon2 signature and single-party setup; R&D only",
        },
      },
    };
  } else {
    manifest.noir.canonical[String(capacity)] = noirAssets;
  }
}
await mkdir(join(DIST, "vendor"), { recursive: true });
const bundle = await build({
  entryPoints: [join(HERE, "web/noir.js")], outfile: join(DIST, "vendor/noir.js"),
  bundle: true, platform: "browser", format: "esm", target: "es2022",
  conditions: ["browser"], mainFields: ["browser", "module", "main"],
  minify: true, legalComments: "linked", metafile: true, logLevel: "info",
});
manifest.noir_build = {
  versions: VERSIONS, package_lock_sha256: hash(await readFile(join(HERE, "package-lock.json"))),
  bundle_sha256: hash(await readFile(join(DIST, "vendor/noir.js"))),
  source_sha256: Object.fromEntries(await Promise.all(["web/noir.js", "web/worker.js", "web/app.js", "web/index.html", "build-noir.mjs", "src/wasm.rs", "src/statement.rs", "src/groth16.rs", "src/folding.rs", "../groth16-tribute/src/circuit.rs"].map(async path => [path, hash(await readFile(join(HERE, path)))]))),
  historical_preparation: relative(ROOT, PREPARED), srs_points: SRS_POINTS,
  settings: { ipaAccumulation: false, oracleHashType: "keccak", disableZk: false, optimizedSolidityVerifier: false },
};
await writeFile(join(DIST, "vendor/noir-metafile.json"), jsonBytes(bundle.metafile));
await writeFile(join(DIST, "assets/manifest.json"), jsonBytes(manifest));
console.log(`Prepared seven flat Noir circuits, Demo Tribute (Groth16 + Noir), and local SRS at ${DIST}; no production artifacts changed.`);
