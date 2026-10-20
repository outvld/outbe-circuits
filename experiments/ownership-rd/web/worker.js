// All cryptography is delegated to the real wasm-bindgen session classes.
// Explicit warm preparation retains this decoded session across proving series.
const ROOT = new URL("./", import.meta.url);
let wasm;
let session;
let config;
let busy = false;

function elapsed(start) { return Math.round((performance.now() - start) * 1e6); }
function status(id, phase, detail = {}) { self.postMessage({ type: "status", id, phase, ...detail }); }
function memoryComponents() {
  const components = config?.engine === "noir_bb" && session
    ? session.memory_components()
    : { rust_fixture_wasm_bytes: wasm.memory_bytes() };
  if (Object.values(components).some(value => !Number.isSafeInteger(value) || value <= 0)) {
    throw new Error("Некорректная ёмкость линейной памяти WASM.");
  }
  return components;
}
function memoryBytes() { return Object.values(memoryComponents()).reduce((sum, value) => sum + value, 0); }
function parseReport(text) { return JSON.parse(text); }
function validateConfig(value) {
  if (!value || !["flat_aggregation", "demo_tribute"].includes(value.circuit)
      || !["groth16", "nova", "noir_bb"].includes(value.engine) || !["canonical", "poseidon2"].includes(value.scheme)
      || ![1, 2, 4, 8, 16, 32, 64].includes(value.capacity)) throw new Error("Некорректная конфигурация Worker.");
  if (value.engine === "noir_bb" && value.scheme !== "canonical") throw new Error("Для Noir подключён только исходный canonical circuit.");
  if (value.wrap && (value.circuit !== "demo_tribute" || value.engine !== "noir_bb")) throw new Error("Backend wrapping доступен только для Demo Tribute + Noir.");
  if (value.circuit === "demo_tribute" && (value.capacity !== 1 || value.engine === "nova")) {
    throw new Error("Demo Tribute (full_proof): один proof, Groth16 или canonical Noir; Nova недоступна.");
  }
}
function assetURL(asset) {
  if (!asset || typeof asset.path !== "string" || !Number.isSafeInteger(asset.bytes) || asset.bytes <= 0
      || typeof asset.sha256 !== "string" || !/^[a-f0-9]{64}$/i.test(asset.sha256)) throw new Error("Некорректная запись ключа в manifest.");
  const url = new URL(asset.path, ROOT);
  if (url.origin !== ROOT.origin || !url.href.startsWith(ROOT.href) || url.search || url.hash) {
    throw new Error("Ключи должны находиться внутри статического каталога приложения на том же origin.");
  }
  return url;
}
async function fetchAsset(id, name, asset, observations) {
  const url = assetURL(asset);
  status(id, "asset_fetch", { asset: name, bytes: asset.bytes });
  let start = performance.now();
  const response = await fetch(url, { credentials: "same-origin", redirect: "error" });
  if (!response.ok) throw new Error(`${asset.path}: HTTP ${response.status}`);
  // arrayBuffer avoids keeping a second array of stream chunks for huge PKs.
  const buffer = await response.arrayBuffer();
  observations.push({ source: "javascript", stage: `asset_fetch:${name}`, elapsed_ns: elapsed(start), bytes: buffer.byteLength });
  if (buffer.byteLength !== asset.bytes) throw new Error(`${asset.path}: длина ${buffer.byteLength}, ожидалось ${asset.bytes} байт.`);
  status(id, "asset_sha256", { asset: name });
  start = performance.now();
  const hex = wasm.asset_sha256(new Uint8Array(buffer));
  observations.push({ source: "javascript", stage: `asset_sha256:${name}`, elapsed_ns: elapsed(start) });
  if (hex !== asset.sha256.toLowerCase()) throw new Error(`${asset.path}: SHA-256 не совпадает с manifest. Скачивание или комплект файлов повреждены.`);
  return new Uint8Array(buffer);
}
async function initialize(message) {
  if (session) throw new Error("Worker уже содержит сессию; создайте новый Worker для cold-запуска.");
  if (!self.crypto?.getRandomValues) throw new Error("Браузер не предоставляет криптографическую случайность getRandomValues. Используйте актуальный браузер и HTTPS.");
  validateConfig(message.config);
  config = { ...message.config };
  const names = config.engine === "groth16" ? ["pk", "vk"] : config.engine === "nova" ? ["params"]
    : ["program", "vk", "metadata", "acvm_wasm", "abi_wasm", "bb_wasm", "g1", "g2"];
  for (const name of names) assetURL(message.assets?.[name]);
  const observations = [];
  status(message.id, "module_import");
  let start = performance.now();
  wasm = await import("./pkg/ownership_rd.js");
  observations.push({ source: "javascript", stage: "module_import", elapsed_ns: elapsed(start) });
  status(message.id, "wasm_init");
  start = performance.now();
  await wasm.default();
  observations.push({ source: "javascript", stage: "wasm_download_init", elapsed_ns: elapsed(start) });
  const memoryAfterInit = memoryBytes();
  const memoryComponentsAfterInit = memoryComponents();
  let buildInfo;
  const buildText = wasm.build_info();
  try { buildInfo = JSON.parse(buildText); } catch { buildInfo = buildText; }
  const buffers = {};
  for (const name of names) buffers[name] = await fetchAsset(message.id, name, message.assets[name], observations);
  status(message.id, "session_load");
  start = performance.now();
  if (config.engine === "noir_bb") {
    const moduleStart = performance.now();
    const { NoirSession } = await import("./vendor/noir.js");
    observations.push({ source: "javascript", stage: "noir_module_import", elapsed_ns: elapsed(moduleStart) });
    start = performance.now();
    session = await NoirSession.create({ circuit: config.circuit, capacity: config.capacity, wasm, buffers });
  } else {
    session = config.engine === "groth16"
      ? new wasm.GrothSession(config.circuit, config.scheme, config.capacity, buffers.pk, buffers.vk)
      : new wasm.FoldingSession(config.scheme, buffers.params);
  }
  observations.push({ source: "javascript", stage: "session_load_wall", elapsed_ns: elapsed(start) });
  // This enclosing interval is not additive with the engine's inner load stages.
  const loadReport = config.engine === "noir_bb" ? session.load_report() : parseReport(session.load_report());
  if (config.engine === "noir_bb") buildInfo = { fixture_module: buildInfo, noir_backend: loadReport };
  const memoryAfterLoad = memoryBytes();
  self.postMessage({ type: "ready", id: message.id, load_report: loadReport, observations,
    memory_after_init_bytes: memoryAfterInit, memory_after_load_bytes: memoryAfterLoad,
    memory_components_after_init: memoryComponentsAfterInit, memory_components_after_load: memoryComponents(),
    build_info: buildInfo, execution: { threads: 1, worker: "module", shared_array_buffer_required: false } });
}
async function run(message) {
  if (!session) throw new Error("Сначала загрузите WASM-сессию.");
  if (!Number.isSafeInteger(message.sample) || message.sample < 1) throw new Error("Номер запуска должен быть положительным целым числом.");
  status(message.id, "proving", { sample: message.sample });
  const before = memoryBytes();
  const componentsBefore = memoryComponents();
  const start = performance.now();
  if (message.wrap && (config.circuit !== "demo_tribute" || config.engine !== "noir_bb")) throw new Error("Wrapping требует Demo Tribute + Noir.");
  const packaged = message.wrap ? await session.package_proof(message.sample) : null;
  const output = packaged ? packaged.report : config.engine === "groth16" ? session.run(message.sample)
    : config.engine === "nova" ? session.run(config.capacity, message.sample)
    : await session.run(message.sample);
  const runWall = elapsed(start);
  const after = memoryBytes();
  const result = config.engine === "noir_bb" ? output : parseReport(output);
  if (result.verified !== true || result.engine !== config.engine || result.scheme !== config.scheme
      || result.circuit !== config.circuit || result.capacity !== config.capacity || result.sample !== message.sample || !Array.isArray(result.timings)) {
    throw new Error("WASM не вернул успешное независимое доказательство с ожидаемыми параметрами.");
  }
  for (const row of result.timings) {
    if (typeof row.stage !== "string" || !Number.isSafeInteger(row.elapsed_ns) || row.elapsed_ns < 0) {
      throw new Error("WASM вернул некорректное время этапа (elapsed_ns).");
    }
  }
  let wrapperExpected;
  if (packaged) {
    const decode = hex => {
      if (!/^0x(?:[0-9a-f]{2})+$/i.test(hex)) throw new Error("Некорректный hex в экспортированном proof.");
      return Uint8Array.from(hex.slice(2).match(/../g), byte => parseInt(byte, 16));
    };
    const request = packaged.request;
    if (request.circuit !== "demo_tribute" || request.public_inputs?.length !== 4 || request.public_inputs.some(word => !/^0x[0-9a-f]{64}$/i.test(word))) throw new Error("Некорректный statement экспортированного proof.");
    wrapperExpected = {
      statement_sha256: wasm.asset_sha256(decode("0x" + request.public_inputs.map(word => word.slice(2)).join(""))),
      inner_proof_sha256: wasm.asset_sha256(decode(request.proof_hex)),
    };
  }
  self.postMessage({ type: "result", id: message.id, result,
    ...(packaged ? { wrapper_request: packaged.request, wrapper_expected: wrapperExpected } : {}),
    memory_before_bytes: before, memory_after_bytes: after,
    memory_components_before: componentsBefore, memory_components_after: memoryComponents(),
    observations: [{ source: "javascript", stage: "run_wall", elapsed_ns: runWall }] });
}
self.onmessage = async event => {
  const message = event.data;
  if (!message || !Number.isSafeInteger(message.id)) return;
  if (busy) { self.postMessage({ type: "error", id: message.id, message: "Worker занят." }); return; }
  busy = true;
  try {
    if (message.type === "init") await initialize(message);
    else if (message.type === "run") await run(message);
    else if (message.type === "check" && config?.engine === "noir_bb" && session) {
      self.postMessage({ type: "checked", id: message.id, result: await session.check(message.sample ?? 1) });
    }
    else if (message.type === "export_proof" && config?.engine === "noir_bb" && session) {
      self.postMessage({ type: "proof", id: message.id, result: await session.package_proof(message.sample ?? 1) });
    }
    else throw new Error("Неизвестная команда Worker.");
  } catch (error) {
    const panic = wasm?.last_panic?.();
    const messageText = String(error?.message || error);
    self.postMessage({ type: "error", id: message.id, message: panic ? `${messageText}: ${panic}` : messageText, stack: error?.stack || null });
  } finally { busy = false; }
};
