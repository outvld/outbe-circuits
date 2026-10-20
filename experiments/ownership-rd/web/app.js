const $ = id => document.getElementById(id);
const ROOT = new URL("./", import.meta.url);
const CAPACITIES = [1, 2, 4, 8, 16, 32, 64];
const ui = Object.fromEntries(["circuit", "engine", "engine-note", "scheme", "scheme-note", "capacity", "capacity-note", "samples", "mode", "settings", "prepare", "preparation-state", "release", "start", "cancel", "reload", "asset-summary", "status", "error", "progress", "progress-detail", "results", "environment", "large-warning", "manifest-warning", "json-export", "csv-export"].map(id => [id, $(id)]));
let manifest;
let running = false;
let cancelled = false;
let activeWorker;
let prepared;
let rejectPending;
let sequence = 0;
let flatControls;
const runs = [];
ui.wrap = $("wrap");
const INNER_VK = "6b5d5609be0894c5f8379803ab6421bf66b0f1c25c5667644772f2d2bf7225bc";
const WRAPPER_PIPELINE = "direct-ultrakeccakzk-bb5-to-gnark-groth16";
let backendController;
let wrapperInfo;
const backendInitializations = new Map();

function validateInitialization(value) {
  const phases = value?.timings_ms;
  if (!/^[0-9a-f]{32}$/.test(value?.id || "") || value.prover_loaded !== true
      || !Number.isFinite(Date.parse(value.started_at)) || !Number.isFinite(Date.parse(value.finished_at))
      || Date.parse(value.finished_at) < Date.parse(value.started_at)
      || !Number.isFinite(value.wall_ms) || value.wall_ms < 0
      || !Number.isFinite(value.process_lifetime_peak_rss_bytes) || value.process_lifetime_peak_rss_bytes <= 0
      || !phases || Object.values(phases).some(ms => !Number.isFinite(ms) || ms < 0)
      || ["load_setup_provenance", "load_verifying_key", "load_proving_artifacts"].some(stage => !Number.isFinite(phases[stage]))) {
    throw new Error("Backend: некорректная resident initialization.");
  }
}
function rememberInitialization(info) {
  validateInitialization(info.initialization);
  const initialization = info.initialization;
  const existing = backendInitializations.get(initialization.id);
  if (existing) {
    if (JSON.stringify(existing.initialization) !== JSON.stringify(initialization)
        || JSON.stringify(existing.setup_provenance) !== JSON.stringify(info.setup_provenance)) throw new Error("Backend: immutable initialization изменилась.");
    return;
  }
  const observation = { initialization, setup_provenance: info.setup_provenance, backend: info.backend, circuit: info.circuit, inner_vk_sha256: info.inner_vk_sha256 };
  backendInitializations.set(initialization.id, observation);
  if (!runs.length && backendInitializations.size === 1) ui.results.replaceChildren();
  const node = element("article", undefined, "observation");
  node.append(element("h3", `Native backend · startup ${initialization.id}`));
  node.append(table(Object.entries(initialization.timings_ms).map(([stage, elapsed]) => [stage, ms(elapsed * 1e6)]).concat([["initialization wall (включает этапы)", ms(initialization.wall_ms * 1e6)]]), "Однократная инициализация resident backend"));
  node.append(element("p", `Lifetime peak RSS на момент завершения startup: ${bytes(initialization.process_lifetime_peak_rss_bytes)}. Это неизменный снимок пика за жизнь процесса, не память отдельного proof. Клиентские Cold / Warm не перезапускают сервер.`, "muted"));
  node.append(details("Startup / setup provenance", observation));
  ui.results.append(node);
  enableExports();
}
function assertResidentResponse(value) {
  validateInitialization(value.initialization);
  if (value.initialization.id !== wrapperInfo?.initialization.id) throw new Error("Backend перезапущен: initialization ID изменился. Запрос не принят как наблюдение прежней сессии. Повторно включите backend wrapping и запустите новую серию; автоматического повтора нет.");
  rememberInitialization({ ...wrapperInfo, initialization: value.initialization, setup_provenance: value.setup_provenance });
}

async function backendJSON(path, body) {
  const response = await fetch(new URL(path, location.origin), {
    method: body === undefined ? "GET" : "POST",
    headers: body === undefined ? {} : { "Content-Type": "application/json" },
    body: body === undefined ? undefined : JSON.stringify(body),
    credentials: "same-origin", redirect: "error", cache: "no-store",
    signal: backendController?.signal,
  });
  let value;
  try { value = await response.json(); } catch { throw new Error("Backend API недоступен: статический сервер не поддерживает /api/wrapper. Запустите ultrahonk-wrap serve."); }
  if (response.status === 409) throw new Error("Backend занят: предыдущая native computation ещё выполняется (в том числе после отмены клиентом). Дождитесь её завершения и запустите серию вручную.");
  if (!response.ok) throw new Error(value.error || `Backend HTTP ${response.status}`);
  return value;
}
async function discoverWrapper() {
  const info = await backendJSON("/api/wrapper");
  if (info.available !== true || info.circuit !== "demo_tribute" || info.inner_vk_sha256 !== INNER_VK || info.backend !== "native-go-gnark") throw new Error("Backend не поддерживает ожидаемый Demo Tribute / VK pin.");
  if (info.setup_provenance?.circuit !== "demo_tribute" || info.setup_provenance?.inner_vk_sha256 !== INNER_VK
      || info.process_policy !== "resident artifacts initialized once at server startup; one native request at a time"
      || info.cancellation_policy !== "client cancellation stops waiting; active native computation finishes before the server accepts another request") throw new Error("Backend: ожидается resident API с проверенным setup provenance.");
  rememberInitialization(info);
  return info;
}
function validateWrapper(value, sample, verification = false) {
  assertResidentResponse(value);
  const request = sample.wrapper_request;
  const expected = sample.wrapper_expected;
  if (value.verified !== true || value.circuit !== request.circuit || value.pipeline !== WRAPPER_PIPELINE
      || value.inner_vk_sha256 !== INNER_VK || request.inner_vk_sha256 !== INNER_VK
      || JSON.stringify(value.public_inputs) !== JSON.stringify(request.public_inputs.map(word => word.toLowerCase()))
      || value.statement_sha256 !== expected.statement_sha256
      || (!verification && value.inner_proof_sha256 !== expected.inner_proof_sha256)) throw new Error("Backend: proof / public inputs / VK / statement не совпадают с клиентским экспортом.");
  if (!/^0x(?:[0-9a-f]{2})+$/i.test(value.groth16_proof_hex || "")
      || value.groth16_proof_bytes !== (value.groth16_proof_hex.length - 2) / 2
      || value.groth16_encoding !== "gnark-bn254-WriteTo-compressed-Ar-Bs-Krs-commitments-commitmentPok"
      || !Number.isSafeInteger(value.groth16_commitments) || value.groth16_commitments < 0
      || !Number.isFinite(value.process_lifetime_peak_rss_bytes) || value.process_lifetime_peak_rss_bytes <= 0
      || !Number.isFinite(value.request_wall_ms) || value.request_wall_ms < 0
      || !value.timings_ms || Object.values(value.timings_ms).some(ms => !Number.isFinite(ms) || ms < 0)) throw new Error("Backend: некорректный формат proof или метрик.");
  if (verification && value.groth16_proof_hex !== sample.wrapper.wrap.groth16_proof_hex) throw new Error("Resident verify вернул другое доказательство.");
  if ("load_verifying_key" in value.timings_ms || "load_proving_artifacts" in value.timings_ms || "load_setup_provenance" in value.timings_ms) throw new Error("Backend: startup stages не должны входить в request timings.");
  for (const stage of verification ? ["groth16_verify"] : ["parse_inner_proof", "witness_encoding", "constraint_solve_and_groth16_prove", "groth16_verify"]) {
    if (!Number.isFinite(value.timings_ms[stage])) throw new Error(`Backend: отсутствует этап ${stage}.`);
  }
  if (value.setup_provenance?.circuit !== request.circuit || value.setup_provenance?.inner_vk_sha256 !== INNER_VK) throw new Error("Backend: setup provenance не совпадает со statement.");
}
async function wrapSample(sample) {
  const wrapper = sample.wrapper = { status: "running", backend: wrapperInfo.backend, initialization_id: wrapperInfo.initialization.id, stages: [] };
  try {
    if (sample.result.verified !== true) throw new Error("Клиентское доказательство не проверено.");
    if (sample.wrapper_request?.circuit !== "demo_tribute" || sample.wrapper_request.inner_vk_sha256 !== INNER_VK) throw new Error("Клиентский экспорт не соответствует поддерживаемому backend VK pin.");
    for (const operation of ["wrap", "verify"]) {
      ui.status.textContent = operation === "wrap" ? "Resident backend: in-circuit UltraHonk → Groth16 prove + verify…" : "Resident backend: отдельный verification call с уже загруженным VK…";
      const value = await backendJSON(`/api/${operation}`, operation === "wrap" ? sample.wrapper_request : {
        circuit: sample.wrapper_request.circuit, public_inputs: sample.wrapper_request.public_inputs,
        inner_vk_sha256: sample.wrapper_request.inner_vk_sha256, groth16_proof_hex: wrapper.wrap.groth16_proof_hex,
      });
      validateWrapper(value, sample, operation === "verify");
      const { initialization, setup_provenance, ...requestResult } = value;
      wrapper[operation] = { ...requestResult, initialization_id: initialization.id };
      for (const [stage, elapsed_ms] of Object.entries(value.timings_ms)) wrapper.stages.push({ source: "native_backend", stage: `${operation}:${stage}`, elapsed_ns: Math.round(elapsed_ms * 1e6) });
      wrapper.stages.push({ source: "native_backend", stage: `${operation}:request_wall`, elapsed_ns: Math.round(value.request_wall_ms * 1e6) });
    }
    wrapper.status = "verified";
  } catch (error) {
    wrapper.status = cancelled ? "cancelled" : "error";
    wrapper.error = String(error?.message || error);
    throw error;
  }
}

const environment = {
  captured_at: new Date().toISOString(),
  user_agent: navigator.userAgent,
  platform: navigator.userAgentData?.platform || navigator.platform || null,
  user_agent_data: navigator.userAgentData?.toJSON?.() || null,
  hardware_concurrency_reported: navigator.hardwareConcurrency ?? null,
  device_memory_gib_reported: navigator.deviceMemory ?? null,
  languages: navigator.languages,
  screen: { width: screen.width, height: screen.height, device_pixel_ratio: devicePixelRatio },
  secure_context: window.isSecureContext,
  artifact_hash: "SHA-256 in the same Rust/WASM module; no SubtleCrypto dependency",
  requested_execution_threads: 1,
  thread_policy: "One execution thread per browser Worker. Noir selects the compatible BB WASM artifact from cross-origin isolation / SharedArrayBuffer availability; execution remains single-threaded.",
  memory_policy: "Observed WASM linear-memory capacities only. Noir totals sum Rust fixture, ACVM, ABI and BB module capacities; component snapshots identify the measured modules. Excludes JS buffers, runtime RSS, physical RAM and peak memory.",
  timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
};
ui.environment.textContent = JSON.stringify(environment, null, 2);

function element(tag, text, className) {
  const node = document.createElement(tag);
  if (text !== undefined) node.textContent = text;
  if (className) node.className = className;
  return node;
}
function bytes(value) { return `${value.toLocaleString("ru-RU")} байт (${(value / 1048576).toFixed(2)} MiB)`; }
function ms(ns) { return (ns / 1e6).toLocaleString("ru-RU", { minimumFractionDigits: 3, maximumFractionDigits: 3 }); }
function circuitCaption(circuit) { return circuit === "demo_tribute" ? "Demo Tribute (full_proof)" : "Flat aggregation"; }
function showError(error) {
  ui.error.hidden = false;
  ui.error.textContent = String(error?.message || error);
}
function assertSupport() {
  if (!globalThis.crypto?.getRandomValues) throw new Error("Нужен браузер с криптографической случайностью getRandomValues. Используйте актуальный Safari, Chrome или Firefox; если API заблокирован, откройте HTTPS.");
  if (!window.Worker || !window.WebAssembly) throw new Error("Этот браузер должен поддерживать WebAssembly и модульные Web Worker. Используйте актуальный Safari, Chrome или Firefox.");
}
function validateAsset(asset) {
  if (!asset || typeof asset.path !== "string" || !Number.isSafeInteger(asset.bytes) || asset.bytes <= 0 || !/^[a-f0-9]{64}$/i.test(asset.sha256 || "")) {
    throw new Error("manifest: у каждого артефакта нужны path, положительный целый bytes и SHA-256 (64 hex-символа).");
  }
  const url = new URL(asset.path, ROOT);
  if (url.origin !== ROOT.origin || !url.href.startsWith(ROOT.href) || url.search || url.hash) throw new Error("manifest: артефакты должны находиться в каталоге приложения на том же origin.");
}
function availableCapacities(scheme, engine = ui.engine.value, circuit = ui.circuit.value) {
  if (circuit === "demo_tribute") return (engine === "groth16" && manifest.demo_tribute?.groth16?.[scheme]) || (engine === "noir_bb" && scheme === "canonical" && manifest.demo_tribute?.noir_bb) ? [1] : [];
  const entries = engine === "noir_bb" ? manifest.noir?.[scheme] : manifest.groth16?.[scheme];
  return Object.keys(entries || {}).map(Number).filter(n => Number.isSafeInteger(n) && n > 0 && (engine === "noir_bb" || CAPACITIES.includes(n))).sort((a, b) => a - b);
}
function selectedConfig() {
  return { circuit: ui.circuit.value, engine: ui.engine.value, scheme: ui.scheme.value, capacity: Number(ui.capacity.value), mode: ui.mode.value, samples: Number(ui.samples.value), wrap: ui.wrap.checked };
}
function selectedAssets(config) {
  const noir = config.engine === "noir_bb";
  const demo = config.circuit === "demo_tribute";
  if (!demo && config.circuit !== "flat_aggregation") throw new Error("Неизвестная схема.");
  if (demo && (config.capacity !== 1 || !["groth16", "noir_bb"].includes(config.engine))) throw new Error("Demo Tribute (full_proof): один слот, Groth16 или Noir / UltraHonk. Nova недоступна.");
  if (demo && (manifest.demo_tribute?.capacity !== 1 || manifest.demo_tribute?.merkle_depth !== 32)) throw new Error("manifest: Demo Tribute требует capacity: 1 и merkle_depth: 32.");
  if (noir && config.scheme !== "canonical") throw new Error("Noir / UltraHonk доступен только для Canonical; экспериментальная Poseidon2-подпись не поддерживается.");
  const assets = noir
    ? {
      ...(demo ? manifest.demo_tribute?.noir_bb : manifest.noir?.canonical?.[String(config.capacity)]),
      ...manifest.noir_runtime,
      bb_wasm: manifest.noir_runtime?.[globalThis.crossOriginIsolated && typeof SharedArrayBuffer !== "undefined" ? "bb_threads_wasm" : "bb_wasm"],
    }
    : config.engine === "groth16" ? (demo ? manifest.demo_tribute?.groth16?.[config.scheme] : manifest.groth16?.[config.scheme]?.[String(config.capacity)]) : manifest.nova?.[config.scheme];
  const names = noir ? ["program", "vk", "metadata", "acvm_wasm", "abi_wasm", "bb_wasm", "g1", "g2"] : config.engine === "groth16" ? ["pk", "vk"] : ["params"];
  if (!assets || !availableCapacities(config.scheme, config.engine, config.circuit).includes(config.capacity)) throw new Error("В manifest нет артефактов для выбранной конфигурации.");
  const selected = {};
  for (const name of names) { validateAsset(assets[name]); selected[name] = assets[name]; }
  return selected;
}
function updateOptions() {
  if (!manifest || running) return;
  const demo = ui.circuit.value === "demo_tribute";
  const changedEngine = demo && ui.engine.value === "nova";
  for (const option of ui.engine.options) option.disabled = demo && option.value === "nova";
  if (changedEngine) ui.engine.value = "groth16";
  ui["engine-note"].textContent = demo
    ? `${changedEngine ? "Nova переключена на Groth16. " : ""}Demo Tribute (full_proof) поддерживает Groth16 и Noir / UltraHonk. Nova недоступна: её ownership-шаг не проверяет Merkle inclusion.`
    : "Groth16, Nova и Noir / UltraHonk доступны для Flat aggregation. Nova выполняет ownership-шаг по каждому слоту.";
  const noir = ui.engine.value === "noir_bb";
  ui.wrap.disabled = !(demo && noir);
  if (ui.wrap.disabled) { ui.wrap.checked = false; wrapperInfo = undefined; }
  const canonicalOnly = noir;
  const changedScheme = canonicalOnly && ui.scheme.value !== "canonical";
  for (const option of ui.scheme.options) option.disabled = canonicalOnly && option.value !== "canonical";
  if (canonicalOnly) ui.scheme.value = "canonical";
  ui["scheme-note"].textContent = canonicalOnly
    ? `${changedScheme ? "Подпись переключена на Canonical. " : ""}Noir / UltraHonk и его backend wrapper используют только Canonical; их VK не изменяется.`
    : "Groth16 поддерживает Canonical и экспериментальную Poseidon2-подпись для Flat aggregation и Demo Tribute; Nova — только для Flat aggregation. Merkle inclusion в обоих full-proof вариантах сохраняется.";
  ui.capacity.disabled = demo;
  ui["capacity-note"].textContent = demo
    ? "Demo Tribute: один NFT, 32 уровня Merkle inclusion, четыре public values. Дерево использует Poseidon2 в обоих вариантах; выбор подписи меняет проверку авторизации. Количество слотов фиксировано: 1."
    : "Доступные размеры берутся из manifest выбранного движка. Nova выполняет по одному ownership-шагу на слот; Noir использует отдельную схему для каждого размера.";
  const previous = Number(ui.capacity.value) || 1;
  ui.capacity.replaceChildren();
  const capacities = availableCapacities(ui.scheme.value);
  for (const capacity of capacities) {
    const option = element("option", `${capacity} ${capacity === 1 ? "слот" : capacity < 5 ? "слота" : "слотов"}`);
    option.value = String(capacity);
    ui.capacity.append(option);
  }
  ui.capacity.value = String(capacities.includes(previous) ? previous : capacities[0]);
  updateSummary();
}
function changeCircuit() {
  if (ui.circuit.value === "demo_tribute") {
    flatControls = { engine: ui.engine.value, scheme: ui.scheme.value, capacity: ui.capacity.value };
  } else if (flatControls) {
    ui.engine.value = flatControls.engine;
    ui.scheme.value = flatControls.scheme;
  }
  updateOptions();
  if (ui.circuit.value === "flat_aggregation" && flatControls) {
    if ([...ui.capacity.options].some(option => option.value === flatControls.capacity)) ui.capacity.value = flatControls.capacity;
    updateSummary();
  }
}
function updateSummary() {
  if (!manifest || running) return;
  const config = selectedConfig();
  const warm = config.mode === "warm";
  if (prepared && (!warm || !matchesPreparation(config))) {
    terminateWorker();
    ui.status.textContent = "Параметры изменены: предыдущая WASM-сессия освобождена.";
  }
  ui["large-warning"].hidden = config.capacity < 16;
  try {
    const assets = selectedAssets(config);
    let total = 0;
    const list = element("ul");
    for (const [name, asset] of Object.entries(assets)) {
      total += asset.bytes;
      list.append(element("li", `${name}: ${asset.path} · ${bytes(asset.bytes)} · SHA-256 ${asset.sha256}`));
    }
    if (!Number.isSafeInteger(total)) throw new Error("Суммарная длина артефактов вне поддерживаемого диапазона.");
    const count = Number.isSafeInteger(config.samples) && config.samples > 0 ? config.samples : null;
    const repeats = warm ? (prepared ? 0 : 1) : count;
    const transfer = repeats && Number.isSafeInteger(total * repeats)
      ? ` За серию артефакты будут прочитаны ${repeats} раз: ${bytes(total * repeats)} до учёта HTTP-кэша.` : "";
    ui["asset-summary"].replaceChildren(
      element("strong", `${circuitCaption(config.circuit)} · файлы из manifest до старта: ${bytes(total)} на загрузку сессии.`), list,
      element("p", `${warm ? prepared ? "Warm: сессия уже подготовлена; повторной загрузки ключей при proving не будет." : "Warm: одна загрузка по кнопке «Подготовить WASM», затем повторное использование между сериями." : "Cold benchmark: новый Worker и повторная подготовка для каждого доказательства."}${transfer}`),
      element("small", `${config.engine === "noir_bb" ? "Включены program, VK, metadata, ABI/ACVM/BB WASM и SRS G1/G2; выбран только один вариант BB WASM. Сохранённого proving key нет: BB использует SRS. Дополнительные JS и Rust fixture WASM не входят в эту сумму." : "Дополнительные JS и Rust WASM не входят в эту сумму: их размеры не заданы в manifest."} Сеть/кэш и SHA-256 измеряются отдельно от этапов движка. Перед использованием проверяются точная длина и SHA-256 каждого артефакта.`)
    );
    ui.prepare.disabled = !warm || !!prepared;
    ui.prepare.textContent = prepared ? "WASM подготовлен" : "Подготовить WASM";
    ui.release.disabled = !activeWorker;
    ui.start.disabled = warm && !prepared;
    ui.start.textContent = warm ? "Создать и проверить" : "Cold: подготовить, создать и проверить";
    ui["preparation-state"].textContent = prepared
      ? `Готова сессия ${prepared.id}: ${circuitCaption(config.circuit)} / ${config.engine} / ${config.scheme}. Ключи остаются декодированными до сброса, отмены, ошибки или закрытия страницы.`
      : warm ? "Сначала подготовьте WASM: загрузка модулей и ключей, SHA-256, pk_decode / параметры / SRS." : "Cold benchmark включает подготовку в каждый запуск; для отдельной кнопки выберите Warm.";
  } catch (error) {
    ui["asset-summary"].textContent = error.message;
    ui.start.disabled = true;
    ui.prepare.disabled = true;
    ui.release.disabled = !activeWorker;
  }
}
async function loadManifest() {
  terminateWorker();
  ui.prepare.disabled = ui.release.disabled = true;
  ui.error.hidden = true;
  ui.reload.hidden = true;
  ui.settings.disabled = true;
  ui.start.disabled = true;
  ui.status.textContent = "Загрузка manifest…";
  try {
    assertSupport();
    const response = await fetch(new URL("assets/manifest.json", ROOT), { cache: "no-cache", redirect: "error" });
    if (!response.ok) throw new Error(`assets/manifest.json: HTTP ${response.status}. Разместите подготовленные assets/ рядом с index.html.`);
    const value = await response.json();
    if (value.format !== 1 || !value.groth16 || !value.nova) throw new Error("Неподдерживаемый manifest: ожидается format: 1 и секции groth16 / nova.");
    manifest = value;
    ui["manifest-warning"].textContent = typeof value.warning === "string" ? value.warning : "";
    ui.settings.disabled = false;
    if (!window.isSecureContext) {
      ui["manifest-warning"].textContent += " HTTP: только публичные тестовые fixtures в доверенной локальной сети. Код и manifest не защищены от подмены при передаче; для недоверенной сети используйте HTTPS.";
    }
    updateOptions();
    ui.status.textContent = "Выберите параметры и подготовьте WASM. Proving запускается отдельной кнопкой.";
  } catch (error) {
    showError(error);
    ui.status.textContent = "Подготовка недоступна.";
    ui.reload.hidden = false;
  }
}

const PHASES = {
  module_import: "Загрузка JS-модуля wasm-bindgen…",
  wasm_init: "Загрузка и инициализация Rust fixture WASM…",
  asset_fetch: "Скачивание/чтение артефакта",
  asset_sha256: "Проверка SHA-256 артефакта",
  session_load: "Декодирование артефактов и подготовка сессии…",
  proving: "Rust: fixture → witness / folding → proof → кодирование → проверка. Точный текущий внутренний этап недоступен; отмена остаётся доступной.",
};
function phaseLabel(phase, engine) {
  if (engine === "noir_bb" && phase === "session_load") return "Инициализация Noir ABI/ACVM и BB WASM, SRS и проверка схемы…";
  if (engine === "noir_bb" && phase === "proving") return "Noir / BB: fixture → ABI → ACVM witness → UltraHonk proof → кодирование → проверка. Точный текущий внутренний этап недоступен; отмена остаётся доступной.";
  return PHASES[phase] || phase;
}
function engineSource(engine) { return engine === "noir_bb" ? "noir_bb_wasm_api" : "rust"; }
function engineCaption(engine) { return engine === "noir_bb" ? "Noir / BB · WASM и API" : "Rust"; }
function stageLabel(stage, engine) {
  if (engine !== "noir_bb") return stage;
  return ({
    fixture: "fixture · Rust export + JSON",
    abi_encode: "abi_encode · Noir ABI",
    witness: "witness · ACVM + внутренняя десериализация программы",
    public_inputs_check: "public_inputs_check · проверка совпадения public inputs",
    witness_encode: "witness_encode · сериализация witness",
    prove: "prove · BB UltraHonk",
    proof_encode: "proof_encode · сериализация доказательства",
    proof_decode: "proof_decode · десериализация доказательства",
    verify: "verify · BB, включая декодирование сохранённого VK",
  })[stage] || stage;
}
function memoryCaption(engine) {
  return engine === "noir_bb" ? "Сумма ёмкостей линейной памяти Rust fixture + Noir ACVM + Noir ABI + BB" : "Ёмкость линейной памяти Rust WASM";
}
function renderMemoryComponents(node, snapshots) {
  if (Object.values(snapshots).every(components => Object.keys(components || {}).length <= 1)) return;
  const phases = { after_init: "Init", after_load: "Загрузка", before: "До", after: "После" };
  const modules = { rust_fixture_wasm_bytes: "Rust fixture", noir_acvm_wasm_bytes: "Noir ACVM", noir_abi_wasm_bytes: "Noir ABI", bb_wasm_bytes: "BB" };
  const rows = [];
  for (const [snapshot, components] of Object.entries(snapshots)) {
    for (const [component, value] of Object.entries(components || {})) {
      rows.push([phases[snapshot] || snapshot, modules[component] || component, (value / 1048576).toFixed(3)]);
    }
  }
  if (rows.length) node.append(table(rows, "Компоненты WASM · ёмкость, не RSS; точные байты в экспорте", ["Снимок", "Модуль", "MiB"]));
}
function request(worker, payload, expectedType, prefix) {
  const id = ++sequence;
  return new Promise((resolve, reject) => {
    const finish = (callback, value) => {
      rejectPending = null;
      worker.onmessage = worker.onerror = worker.onmessageerror = null;
      if (activeWorker === worker && prepared) watchIdleWorker(worker);
      callback(value);
    };
    rejectPending = error => finish(reject, error);
    worker.onmessage = event => {
      const message = event.data;
      if (message?.id !== id) return;
      if (message.type === "status") {
        ui.status.textContent = `${prefix} ${phaseLabel(message.phase, ui.engine.value)}${message.asset ? ` (${message.asset}${message.bytes ? `, ${bytes(message.bytes)}` : ""})` : ""}`;
      } else if (message.type === "error") {
        const error = new Error(message.message);
        error.worker_stack = message.stack;
        finish(reject, error);
      } else if (message.type === expectedType) finish(resolve, message);
    };
    worker.onerror = event => {
      event.preventDefault();
      finish(reject, new Error(`Ошибка Worker/WASM: ${event.message || "Worker остановлен браузером"}. Проверьте MIME-типы, доступность pkg/ownership_rd.js и соответствующего .wasm. При большой схеме возможен лимит памяти браузера; уменьшите число слотов.`));
    };
    worker.onmessageerror = () => finish(reject, new Error("Браузер не смог прочитать сообщение Worker."));
    try { worker.postMessage({ ...payload, id }); } catch (error) { finish(reject, error); }
  });
}
function terminateWorker() {
  activeWorker?.terminate();
  activeWorker = undefined;
  prepared = undefined;
}
function matchesPreparation(config) {
  return activeWorker && prepared && ["circuit", "engine", "scheme", "capacity"].every(key => prepared.config[key] === config[key]);
}
function watchIdleWorker(worker) {
  worker.onerror = worker.onmessageerror = event => {
    event.preventDefault?.();
    terminateWorker();
    backendController?.abort();
    showError(new Error("Подготовленный Worker остановлен браузером. Подготовьте WASM заново; возможен лимит памяти."));
    updateSummary();
  };
}
function setBusy(value) {
  running = value;
  ui.settings.disabled = value;
  ui.cancel.disabled = !value;
  if (value) {
    cancelled = false;
    ui.prepare.disabled = ui.release.disabled = ui.start.disabled = true;
    ui.error.hidden = true;
  } else updateSummary();
}
function releasePreparation() {
  if (running) return;
  terminateWorker();
  ui.status.textContent = "Worker остановлен. Для следующего warm-запуска подготовьте WASM заново.";
  updateSummary();
}
async function initializeSession(run, firstSample) {
  const started = performance.now();
  activeWorker = new Worker(new URL("worker.js", import.meta.url), { type: "module", name: `ownership-${run.id}-${firstSample ?? "prepare"}` });
  const ready = await request(activeWorker, { type: "init", config: run.config, assets: run.manifest.assets }, "ready", `${run.id} · подготовка.`);
  const loaded = { first_sample: firstSample, captured_at: new Date().toISOString(), ...ready };
  delete loaded.id;
  delete loaded.type;
  loaded.observations.push({ source: "javascript", stage: "worker_start_to_ready_wall", elapsed_ns: Math.round((performance.now() - started) * 1e6) });
  run.sessions.push(loaded);
  renderSession(run, loaded);
}
function table(rows, caption, headers = ["Этап", "мс"]) {
  const wrap = element("div", undefined, "table-scroll");
  wrap.tabIndex = 0;
  wrap.setAttribute("role", "region");
  wrap.setAttribute("aria-label", caption);
  const node = element("table");
  node.append(element("caption", caption));
  const head = element("thead");
  const tr = element("tr");
  for (const header of headers) { const th = element("th", header); th.scope = "col"; tr.append(th); }
  head.append(tr);
  node.append(head);
  const body = element("tbody");
  for (const row of rows) {
    const line = element("tr");
    for (const cell of row) line.append(element("td", String(cell)));
    body.append(line);
  }
  node.append(body);
  wrap.append(node);
  return wrap;
}
function details(title, value) {
  const node = element("details");
  node.append(element("summary", title), element("pre", JSON.stringify(value, null, 2)));
  return node;
}
function renderSession(run, loaded) {
  const node = element("details", undefined, "observation");
  node.open = run.kind === "preparation";
  node.append(element("summary", `${run.id} · ${circuitCaption(run.config.circuit)} · ${run.kind === "preparation" ? "подготовка WASM (без proving)" : `холодная загрузка перед запуском ${loaded.first_sample}`}`));
  node.append(element("p", `${memoryCaption(run.config.engine)}: после init ${bytes(loaded.memory_after_init_bytes)}; после загрузки ${bytes(loaded.memory_after_load_bytes)}.${run.config.engine === "noir_bb" ? " При init измерен только Rust fixture; после загрузки — все четыре модуля." : ""} Это не пик и не RSS; JS-буферы и память runtime не включены.`, "muted"));
  renderMemoryComponents(node, { after_init: loaded.memory_components_after_init, after_load: loaded.memory_components_after_load });
  if (Array.isArray(loaded.load_report?.timings)) {
    node.append(table(loaded.load_report.timings.map(row => [stageLabel(row.stage, run.config.engine), ms(row.elapsed_ns)]), `Подготовка сессии · ${engineCaption(run.config.engine)}`));
  }
  node.append(table(loaded.observations.map(row => [row.stage, ms(row.elapsed_ns)]), "Загрузка / внешние интервалы · JavaScript"));
  node.append(element("p", "session_load_wall включает внутренние этапы подготовки движка: интервалы вложены, их нельзя складывать.", "muted"));
  node.append(details("Полная загрузка и build_info", loaded));
  ui.results.append(node);
}
function renderSample(run, sample) {
  const result = sample.result;
  const node = element("article", undefined, "observation");
  node.append(element("h3", `${run.id} · ${circuitCaption(result.circuit)} · ${result.engine} / ${result.scheme} / ${result.capacity} · запуск ${result.sample}`));
  node.append(element("p", "Доказательство создано, кодировано/декодировано и успешно проверено на этом устройстве."));
  node.append(table(result.timings.map(row => [stageLabel(row.stage, result.engine), ms(row.elapsed_ns)]), `Отдельные этапы · ${engineCaption(result.engine)}`));
  node.append(element("p", `${memoryCaption(result.engine)}: до ${bytes(sample.memory_before_bytes)}; после ${bytes(sample.memory_after_bytes)}. Не RSS и не измерение пика; JS-буферы и память runtime не включены. Полный вызов движка с JS-границы: ${ms(sample.observations[0].elapsed_ns)} мс (включает внутренние этапы).`, "muted"));
  renderMemoryComponents(node, { before: sample.memory_components_before, after: sample.memory_components_after });
  node.append(details("Размеры и структурные метрики (исходные единицы движка)", result.sizes || {}));
  if (sample.wrapper) {
    node.append(element("p", sample.wrapper.status === "verified" ? "Pipeline verified: клиентский UltraHonk и outer Groth16 успешно проверены; дополнительный verify использовал resident VK." : `Backend pipeline: ${sample.wrapper.status}. ${sample.wrapper.error || ""}`, sample.wrapper.status === "verified" ? "" : "error"));
    node.append(element("p", `Resident initialization: ${sample.wrapper.initialization_id}. Startup измерен отдельно один раз.`, "muted"));
    node.append(table(sample.wrapper.stages.map(row => [row.stage, ms(row.elapsed_ns)]), "Native backend · только request costs (не мобильный WASM)"));
    for (const operation of ["wrap", "verify"]) {
      const value = sample.wrapper[operation];
      if (value && Number.isFinite(value.groth16_proof_bytes) && Number.isFinite(value.process_lifetime_peak_rss_bytes)) node.append(element("p", `${operation}: compressed Groth16 ${bytes(value.groth16_proof_bytes)}; commitments ${value.groth16_commitments}; process lifetime peak RSS ${bytes(value.process_lifetime_peak_rss_bytes)} (пик за всю жизнь сервера, не отдельного запроса и не дельта). Формат gnark с commitment extensions, не предполагаемые 128 байт / EVM proof.`, "muted"));
    }
  }
  const { wrapper_request, wrapper_expected, wrapper, ...clientObservation } = sample;
  node.append(details("Клиентское наблюдение / digest (proof hex только в JSON export)", clientObservation));
  ui.results.append(node);
}
function enableExports() {
  ui["json-export"].disabled = ui["csv-export"].disabled = runs.length === 0 && backendInitializations.size === 0;
}
function createRecord(config, assets, kind) {
  if (!runs.length && !backendInitializations.size) ui.results.replaceChildren();
  const run = {
    id: `${kind === "preparation" ? "prep" : "run"}-${runs.length + 1}`, kind,
    started_at: new Date().toISOString(),
    environment: { ...environment, captured_at: new Date().toISOString(), visibility_at_start: document.visibilityState },
    config,
    manifest: { format: manifest.format, warning: manifest.warning, source_sha256: manifest.source_sha256, versions: manifest.versions, noir_build: manifest.noir_build, assets },
    measurement_policy: {
      independent_proof_per_sample: true, warmups_discarded: false, aggregation: "none",
      cold: "New module Worker per sample; browser HTTP and OS caches are left intact.",
      warm: "Explicit preparation, retained across proving series until configuration change, release, cancellation, error or page close. Preparation costs occur once in the referenced preparation record; no discarded warmup.",
      verification: "After proving and proof encoding roundtrip in the same session, not standalone cold verification.",
      timers: "Engine elapsed_ns is unchanged; JavaScript module/network/hash/wall intervals are separate. session_load_wall and run_wall contain engine stages and must not be added to them. Noir ACVM witness includes internal program deserialization; verify includes stored VK decoding.",
      download: config.engine === "noir_bb" ? "Manifest sizes include program, VK, metadata, ABI/ACVM/selected BB WASM and SRS G1/G2; additional JS and Rust fixture WASM excluded. HTTP cache may avoid transfer." : "Manifest sizes count key files only; extra JS and Rust WASM excluded. HTTP cache may avoid transfer.",
      memory: environment.memory_policy,
      fixtures: "Public deterministic test keys/witnesses only; not production data.",
    },
    backend_policy: kind === "preparation" ? "Local WASM preparation only; no proof created or uploaded." : config.wrap ? "Explicit opt-in: upload only UltraHonk proof and public inputs, never private witness. Resident artifacts initialized once at server startup; wrap and separate verify use retained keys. Client Cold/Warm never resets server. RSS is process-lifetime high-water mark, separate from WASM capacities. Client abort stops waiting, not active native computation; busy returns 409 until completion." : "Local only; no proof upload.",
    sessions: [], samples: [], status: "running",
  };
  runs.push(run);
  enableExports();
  return run;
}
async function prepare() {
  if (running || prepared || !$("run-form").reportValidity()) return;
  let config, assets;
  try {
    assertSupport();
    config = selectedConfig();
    if (config.mode !== "warm") throw new Error("Отдельная подготовка доступна в Warm; Cold benchmark готовит каждый Worker заново.");
    assets = selectedAssets(config);
  } catch (error) { showError(error); return; }
  const record = createRecord(config, assets, "preparation");
  setBusy(true);
  ui.progress.max = 1;
  ui.progress.removeAttribute("value");
  ui["progress-detail"].textContent = "Подготовка не создаёт доказательство. Все её этапы сохраняются отдельно.";
  try {
    await initializeSession(record, null);
    prepared = { id: record.id, config };
    watchIdleWorker(activeWorker);
    record.status = "completed";
    ui.progress.value = 1;
    ui.status.textContent = "WASM подготовлен. Теперь можно создавать доказательства без повторного pk_decode.";
  } catch (error) {
    record.status = cancelled ? "cancelled" : "error";
    record.error = { message: String(error?.message || error), worker_stack: error?.worker_stack || null, at: new Date().toISOString() };
    terminateWorker();
    ui.progress.value = 0;
    ui.status.textContent = cancelled ? "Подготовка отменена; Worker освобождён." : "Подготовка не завершена.";
    if (!cancelled) showError(error);
  } finally {
    record.finished_at = new Date().toISOString();
    rejectPending = null;
    setBusy(false);
    enableExports();
  }
}
async function execute(event) {
  event.preventDefault();
  if (running) return;
  let config, assets;
  try {
    assertSupport();
    config = selectedConfig();
    if (!Number.isSafeInteger(config.samples) || config.samples < 1) throw new Error("Количество запусков должно быть положительным целым числом.");
    assets = selectedAssets(config);
    if (config.mode === "warm" && !matchesPreparation(config)) throw new Error("Сначала нажмите «Подготовить WASM» для выбранных параметров.");
  } catch (error) { showError(error); return; }
  setBusy(true);
  backendController = new AbortController();
  ui.progress.max = config.samples;
  ui.progress.value = 0;
  ui["progress-detail"].textContent = `Завершено 0 из ${config.samples}. Прогрев не отбрасывается.`;
  const run = createRecord(config, assets, "proof");
  if (config.mode === "warm") {
    run.preparation_id = prepared.id;
    ui.results.append(element("p", `${run.id} использует ${prepared.id}: подготовка и pk_decode не повторяются.`, "muted"));
  }
  try {
    if (config.wrap) {
      if (config.circuit !== "demo_tribute" || config.engine !== "noir_bb") throw new Error("Wrapping требует Demo Tribute + Noir.");
      wrapperInfo = await discoverWrapper();
      run.initialization_id = wrapperInfo.initialization.id;
    }
    for (let sample = 1; sample <= config.samples; sample++) {
      if (cancelled) throw new DOMException("Отменено пользователем", "AbortError");
      const prefix = `${circuitCaption(config.circuit)} · запуск ${sample}/${config.samples}.`;
      if (config.mode === "warm" && !matchesPreparation(config)) throw new Error("Подготовленная сессия потеряна; подготовьте WASM заново.");
      if (!activeWorker) await initializeSession(run, sample);
      const response = await request(activeWorker, { type: "run", sample, wrap: config.wrap }, "result", prefix);
      const observation = { captured_at: new Date().toISOString(), visibility_at_completion: document.visibilityState, ...response };
      delete observation.id;
      delete observation.type;
      run.samples.push(observation);
      try {
        if (config.wrap) await wrapSample(observation);
      } finally { renderSample(run, observation); }
      ui.progress.value = sample;
      ui["progress-detail"].textContent = `Завершено ${sample} из ${config.samples}. Все наблюдения сохраняются отдельно.`;
      if (config.mode === "cold") terminateWorker();
    }
    run.status = "completed";
    ui.status.textContent = `${circuitCaption(config.circuit)} · готово: ${run.samples.length} независимых доказательств успешно проверено. JSON и CSV содержат исходные наблюдения.`;
  } catch (error) {
    run.status = cancelled ? "cancelled" : "error";
    run.error = { message: String(error?.message || error), worker_stack: error?.worker_stack || null, at: new Date().toISOString(), failed_sample: run.samples.at(-1)?.wrapper && run.samples.at(-1).wrapper.status !== "verified" ? run.samples.length : run.samples.length + 1 };
    ui.status.textContent = `${circuitCaption(config.circuit)} · ${cancelled ? "отменено: локальный Worker освобождён; активная native computation сервера продолжится, до её завершения backend занят" : "серия остановлена с ошибкой"}. Сохранено клиентских наблюдений: ${run.samples.length}.`;
    if (!cancelled) showError(error);
  } finally {
    if (config.mode === "cold" || run.status !== "completed") terminateWorker();
    backendController = undefined;
    run.finished_at = new Date().toISOString();
    rejectPending = null;
    setBusy(false);
    enableExports();
  }
}
function cancel() {
  if (!running) return;
  cancelled = true;
  backendController?.abort();
  terminateWorker();
  rejectPending?.(new DOMException("Отменено пользователем", "AbortError"));
  ui.cancel.disabled = true;
}
function download(name, mime, content) {
  const url = URL.createObjectURL(new Blob([content], { type: mime }));
  const anchor = element("a");
  anchor.href = url;
  anchor.download = name;
  document.body.append(anchor);
  anchor.click();
  anchor.remove();
  setTimeout(() => URL.revokeObjectURL(url), 10000);
}
function csvText() {
  const headers = ["run_id", "circuit", "engine", "scheme", "capacity", "mode", "sample", "record", "source", "stage", "elapsed_ns", "value", "unit"];
  const rows = [headers];
  for (const { initialization, ...metadata } of backendInitializations.values()) {
    const startup = (stage, elapsed, value = "", unit = "") => rows.push([initialization.id, metadata.circuit, metadata.backend, "", "", "", "", "backend_initialization", "native_backend_startup", stage, elapsed, value, unit]);
    for (const [stage, elapsed] of Object.entries(initialization.timings_ms)) startup(stage, Math.round(elapsed * 1e6), "", "ns");
    startup("initialization_wall", Math.round(initialization.wall_ms * 1e6), "", "ns");
    startup("process_lifetime_peak_rss_bytes_at_initialization", "", initialization.process_lifetime_peak_rss_bytes, "process_lifetime_high_water_RSS_bytes");
    startup("metadata", "", JSON.stringify({ ...metadata, id: initialization.id, started_at: initialization.started_at, finished_at: initialization.finished_at, prover_loaded: initialization.prover_loaded }), "json");
  }
  for (const run of runs) {
    const base = [run.id, run.config.circuit, run.config.engine, run.config.scheme, run.config.capacity, run.config.mode];
    const metric = (sample, record, source, name, value, unit = "") => rows.push([...base, sample, record, source, name, "", typeof value === "object" ? JSON.stringify(value) : value, unit]);
    const timing = (sample, record, source, row) => rows.push([...base, sample, record, source, row.stage, row.elapsed_ns, "", "ns"]);
    const source = engineSource(run.config.engine);
    const memoryUnit = run.config.engine === "noir_bb" ? "sum_linear_memory_bytes_not_RSS" : "linear_memory_bytes_not_RSS";
    const components = (sample, record, snapshot, values) => {
      for (const [component, value] of Object.entries(values || {})) metric(sample, record, "wasm_linear_memory", `memory_${snapshot}.${component}`, value, "linear_memory_bytes_not_RSS");
    };
    metric("", "metadata", "browser", "environment", run.environment, "json");
    metric("", "metadata", "manifest", "provenance", run.manifest, "json");
    metric("", "metadata", "browser", "measurement_policy", run.measurement_policy, "json");
    metric("", "metadata", "browser", "run_status", run.status);
    metric("", "metadata", "browser", "record_kind", run.kind);
    if (run.preparation_id) metric("", "metadata", "browser", "preparation_id", run.preparation_id);
    if (run.initialization_id) metric("", "metadata", "native_backend", "initialization_id", run.initialization_id);
    metric("", "metadata", "browser", "started_at", run.started_at, "ISO8601");
    if (run.finished_at) metric("", "metadata", "browser", "finished_at", run.finished_at, "ISO8601");
    if (run.error) metric("", "error", "browser", "error", run.error, "json");
    for (const session of run.sessions) {
      for (const row of session.observations) timing(session.first_sample, "session_load", "javascript", row);
      for (const row of session.load_report?.timings || []) timing(session.first_sample, "session_load", source, row);
      metric(session.first_sample, "session_load", "wasm_linear_memory", "memory_after_init", session.memory_after_init_bytes, memoryUnit);
      metric(session.first_sample, "session_load", "wasm_linear_memory", "memory_after_load", session.memory_after_load_bytes, memoryUnit);
      components(session.first_sample, "session_load", "after_init", session.memory_components_after_init);
      components(session.first_sample, "session_load", "after_load", session.memory_components_after_load);
      metric(session.first_sample, "session_load", source, "load_report", session.load_report, "json");
      metric(session.first_sample, "session_load", source, "build_info", session.build_info, "json");
    }
    for (const sample of run.samples) {
      const result = sample.result;
      for (const row of result.timings) timing(result.sample, "proof", source, row);
      for (const row of sample.observations) timing(result.sample, "proof", "javascript", row);
      metric(result.sample, "proof", "wasm_linear_memory", "memory_before", sample.memory_before_bytes, memoryUnit);
      metric(result.sample, "proof", "wasm_linear_memory", "memory_after", sample.memory_after_bytes, memoryUnit);
      components(result.sample, "proof", "before", sample.memory_components_before);
      components(result.sample, "proof", "after", sample.memory_components_after);
      metric(result.sample, "proof", source, "verified", result.verified, "boolean");
      metric(result.sample, "proof", source, "statement_digest", result.statement_digest);
      metric(result.sample, "proof", source, "witness_digest", result.witness_digest);
      metric(result.sample, "proof", source, "sizes", result.sizes, "json_native_engine_units");
      metric(result.sample, "proof", source, "result", result, "json");
      metric(result.sample, "proof", "browser", "captured_at", sample.captured_at, "ISO8601");
      metric(result.sample, "proof", "browser", "visibility_at_completion", sample.visibility_at_completion);
      if (sample.wrapper_request) metric(result.sample, "wrapper", "client", "wrapper_request", sample.wrapper_request, "json");
      if (sample.wrapper) {
        for (const row of sample.wrapper.stages) timing(result.sample, "wrapper", "native_backend", row);
        metric(result.sample, "wrapper", "native_backend", "wrapper", sample.wrapper, "json");
        for (const operation of ["wrap", "verify"]) {
          const value = sample.wrapper[operation];
          if (!value) continue;
          metric(result.sample, "wrapper", "native_backend", `${operation}:process_lifetime_peak_rss_bytes`, value.process_lifetime_peak_rss_bytes, "process_lifetime_high_water_RSS_bytes");
          metric(result.sample, "wrapper", "native_backend", `${operation}:groth16_proof_bytes`, value.groth16_proof_bytes, "compressed_bytes");
        }
      }
    }
  }
  const cell = value => `"${String(value ?? "").replaceAll('"', '""')}"`;
  return rows.map(row => row.map(cell).join(",")).join("\r\n") + "\r\n";
}

$("run-form").addEventListener("submit", execute);
ui.prepare.addEventListener("click", prepare);
ui.release.addEventListener("click", releasePreparation);
ui.cancel.addEventListener("click", cancel);
ui.reload.addEventListener("click", loadManifest);
ui.circuit.addEventListener("change", changeCircuit);
ui.engine.addEventListener("change", updateOptions);
ui.scheme.addEventListener("change", updateOptions);
ui.capacity.addEventListener("change", updateSummary);
ui.mode.addEventListener("change", updateSummary);
ui.samples.addEventListener("input", updateSummary);
ui.wrap.addEventListener("change", async () => {
  wrapperInfo = undefined;
  if (!ui.wrap.checked) return;
  ui.error.hidden = true;
  try { wrapperInfo = await discoverWrapper(); }
  catch (error) { showError(error); }
});
ui["json-export"].addEventListener("click", () => download(`ownership-rd-${Date.now()}.json`, "application/json", JSON.stringify({ format: 3, exported_at: new Date().toISOString(), backend_initializations: [...backendInitializations.values()].map(({ initialization, ...metadata }) => ({ ...initialization, ...metadata })), runs }, null, 2)));
ui["csv-export"].addEventListener("click", () => download(`ownership-rd-${Date.now()}.csv`, "text/csv;charset=utf-8", "\uFEFF" + csvText()));
window.addEventListener("pagehide", () => { cancel(); terminateWorker(); updateSummary(); });
loadManifest();
