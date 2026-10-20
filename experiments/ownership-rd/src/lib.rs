//! Public-fixture ownership experiments, not production cryptography.
// Reuse the original experimental port verbatim: one canonical gadget source.
#[path = "../../groth16-tribute/src/circuit.rs"]
pub mod baseline;
#[path = "../../groth16-tribute/src/poseidon2.rs"]
pub mod poseidon2;
#[path = "../../groth16-tribute/src/poseidon2_constants.rs"]
mod poseidon2_constants;

pub mod folding;
pub mod groth16;
pub mod statement;
#[cfg(feature = "wasm")]
mod wasm;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

pub type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

pub fn measured<T>(
    rows: &mut Vec<Value>,
    stage: &str,
    operation: impl FnOnce() -> Result<T>,
) -> Result<T> {
    let start = web_time::Instant::now();
    let result = operation()?;
    let elapsed_ns = u64::try_from(start.elapsed().as_nanos())?;
    rows.push(json!({"stage":stage,"elapsed_ns":elapsed_ns}));
    Ok(result)
}

pub fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
