use crate::{
    folding, groth16,
    statement::{CircuitFixture, CircuitKind, Scheme},
};
use wasm_bindgen::prelude::*;

thread_local! {
    static LAST_PANIC: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn console_error(message: &str);
}

#[wasm_bindgen(start)]
pub fn initialize() {
    std::panic::set_hook(Box::new(|info| {
        let message = info.to_string();
        console_error(&message);
        LAST_PANIC.with(|last| *last.borrow_mut() = Some(message));
    }));
}

#[wasm_bindgen]
pub fn last_panic() -> Option<String> {
    LAST_PANIC.with(|last| last.borrow().clone())
}

fn error(error: impl std::fmt::Display) -> JsValue {
    js_sys::Error::new(&error.to_string()).into()
}

#[wasm_bindgen]
pub struct GrothSession {
    inner: groth16::Session,
}

#[wasm_bindgen]
impl GrothSession {
    #[wasm_bindgen(constructor)]
    pub fn new(
        circuit: &str,
        scheme: &str,
        capacity: u32,
        pk: &[u8],
        vk: &[u8],
    ) -> Result<GrothSession, JsValue> {
        let kind = CircuitKind::parse(circuit).map_err(error)?;
        let scheme = Scheme::parse(scheme).map_err(error)?;
        let inner = match kind {
            CircuitKind::FlatAggregation => {
                groth16::Session::load(scheme, capacity as usize, pk, vk)
            }
            CircuitKind::DemoTribute => {
                if capacity != 1 {
                    return Err(error("Demo Tribute requires capacity 1"));
                }
                groth16::Session::load_demo(scheme, pk, vk)
            }
        }
        .map_err(error)?;
        Ok(Self { inner })
    }
    pub fn load_report(&self) -> String {
        self.inner.load_report().to_string()
    }
    pub fn run(&self, sample: u32) -> Result<String, JsValue> {
        Ok(self
            .inner
            .run(u64::from(sample))
            .map_err(error)?
            .to_string())
    }
}

#[wasm_bindgen]
pub struct FoldingSession {
    inner: folding::Session,
}

#[wasm_bindgen]
impl FoldingSession {
    #[wasm_bindgen(constructor)]
    pub fn new(scheme: &str, params: &[u8]) -> Result<FoldingSession, JsValue> {
        let scheme = Scheme::parse(scheme).map_err(error)?;
        Ok(Self {
            inner: folding::Session::load(scheme, params).map_err(error)?,
        })
    }
    pub fn load_report(&self) -> String {
        self.inner.load_report().to_string()
    }
    pub fn run(&self, capacity: u32, sample: u32) -> Result<String, JsValue> {
        Ok(self
            .inner
            .run(capacity as usize, u64::from(sample))
            .map_err(error)?
            .to_string())
    }
}

#[wasm_bindgen]
pub fn memory_bytes() -> f64 {
    let memory: js_sys::WebAssembly::Memory = wasm_bindgen::memory().unchecked_into();
    js_sys::Reflect::get(&memory.buffer(), &JsValue::from_str("byteLength"))
        .expect("WebAssembly memory buffer has byteLength")
        .as_f64()
        .expect("WebAssembly memory byteLength is numeric")
}

/// Reuse the native artifact hash on LAN HTTP where SubtleCrypto is unavailable.
/// Cryptographic entropy still comes from the browser's getRandomValues.
#[wasm_bindgen]
pub fn asset_sha256(bytes: &[u8]) -> String {
    crate::sha(bytes)
}

/// The canonical Noir ABI uses the very same public fixtures as the Rust engines.
#[wasm_bindgen]
pub fn noir_fixture(circuit: &str, capacity: u32, sample: u32) -> Result<String, JsValue> {
    use std::fmt::Write;
    let kind = CircuitKind::parse(circuit).map_err(error)?;
    let fixture = crate::statement::circuit_fixture(
        kind,
        Scheme::Canonical,
        capacity as usize,
        u64::from(sample),
    )
    .map_err(error)?;
    let hex_field = |value: &outbe_protocol::Fr| {
        let mut text = String::with_capacity(66);
        text.push_str("0x");
        for byte in outbe_protocol::codec::field_to_be_bytes(value) {
            write!(text, "{byte:02x}").expect("writing to a String cannot fail");
        }
        text
    };
    let public = fixture
        .public_inputs()
        .iter()
        .map(&hex_field)
        .collect::<Vec<_>>();
    let inputs = match &fixture {
        CircuitFixture::Flat(batch) => {
            let keys = batch
                .slots
                .iter()
                .map(|(w, _)| serde_json::json!({"x":hex_field(&w.pk.x),"y":hex_field(&w.pk.y)}))
                .collect::<Vec<_>>();
            let signatures = batch
                .slots
                .iter()
                .map(|(w, _)| w.signature.as_slice())
                .collect::<Vec<_>>();
            let nonces = batch
                .slots
                .iter()
                .map(|(w, _)| hex_field(&w.nonce))
                .collect::<Vec<_>>();
            serde_json::json!({"pk":keys,"signature":signatures,"nonce":nonces,"public_inputs":public})
        }
        CircuitFixture::Demo(c) => serde_json::json!({
            "pk":{"x":hex_field(&c.witness.pk.x),"y":hex_field(&c.witness.pk.y)},
            "signature":c.witness.signature.as_slice(),
            "nonce":hex_field(&c.witness.nonce),
            "merkle_path_siblings":c.witness.merkle_path_siblings.iter().map(&hex_field).collect::<Vec<_>>(),
            "merkle_path_indices":c.witness.merkle_path_indices,
            "derived_owner":hex_field(&c.public.derived_owner),
            "nft_hash":hex_field(&c.public.nft_hash),
            "binding_hash":hex_field(&c.public.binding_hash),
            "merkle_root":hex_field(&c.public.merkle_root)
        }),
    };
    Ok(serde_json::json!({
        "circuit":kind.name(),
        "inputs":inputs,
        "public_inputs":public,
        "statement_digest":fixture.statement_digest(),
        "witness_digest":fixture.witness_digest()
    })
    .to_string())
}

#[wasm_bindgen]
pub fn build_info() -> String {
    serde_json::json!({
        "project":"ownership-rd", "version":env!("CARGO_PKG_VERSION"),
        "target":std::env::consts::ARCH,
        "execution":"single-threaded WASM in a Web Worker; arkworks parallel feature disabled; Nova uses Rayon global single-thread fallback",
        "timer":"web_time::Instant / browser Performance clock",
        "memory":"WASM linear-memory capacity, NOT RSS, heap-live bytes, or physical peak RAM",
        "fixtures":"deterministic public test keys; proof blinding uses browser entropy",
        "security":"R&D only; single-party Groth16 setup; experimental signature unaudited; Nova final proof is Spartan IPA, not Groth16 or an EVM verifier"
    }).to_string()
}
