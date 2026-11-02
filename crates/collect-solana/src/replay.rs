//! Record and replay (CLI-4, QA-3): every RPC request and HTTP GET of a run
//! is saved under a directory keyed by a hash of the request, so the whole
//! pipeline can run again offline and produce the same output.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::rpc::{Transport, TransportError};
use crate::verify::HttpGet;

/// The stable part of a JSON-RPC request: method and params, not the id.
fn rpc_key(body: &Value) -> String {
    let stable = json!({"method": body["method"], "params": body["params"]});
    hex(&Sha256::digest(stable.to_string()))
}

fn http_key(url: &str) -> String {
    hex(&Sha256::digest(url))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn not_recorded(what: &str) -> TransportError {
    // Decode errors are not retried, so a missing recording fails fast.
    TransportError::Decode(format!("no recorded response for {what}"))
}

fn save(path: &Path, value: &Value) -> Result<(), TransportError> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| TransportError::Network(e.to_string()))?;
    }
    let text = celebtir_core::to_canonical_json(value)
        .map_err(|e| TransportError::Decode(e.to_string()))?;
    std::fs::write(path, text).map_err(|e| TransportError::Network(e.to_string()))
}

fn load(path: &Path) -> Option<Value> {
    serde_json::from_str::<Value>(&std::fs::read_to_string(path).ok()?).ok()
}

/// Passes requests to `inner` and saves each successful response.
pub struct Recording<T> {
    pub inner: T,
    pub dir: PathBuf,
}

impl<T: Transport> Transport for Recording<T> {
    async fn send(&self, body: Value) -> Result<Value, TransportError> {
        let response = self.inner.send(body.clone()).await?;
        let path = self
            .dir
            .join("rpc")
            .join(format!("{}.json", rpc_key(&body)));
        let mut stable = body;
        if let Some(o) = stable.as_object_mut() {
            o.remove("id");
        }
        save(&path, &json!({"request": stable, "response": response}))?;
        Ok(response)
    }
}

impl<T: HttpGet> HttpGet for Recording<T> {
    async fn get_json(&self, url: &str) -> Result<Value, TransportError> {
        let response = self.inner.get_json(url).await?;
        let path = self
            .dir
            .join("http")
            .join(format!("{}.json", http_key(url)));
        save(&path, &json!({"url": url, "response": response}))?;
        Ok(response)
    }
}

/// Answers from a recording directory; anything not recorded is an error.
pub struct Replay {
    pub dir: PathBuf,
}

impl Transport for Replay {
    async fn send(&self, body: Value) -> Result<Value, TransportError> {
        let path = self
            .dir
            .join("rpc")
            .join(format!("{}.json", rpc_key(&body)));
        load(&path)
            .map(|v| v["response"].clone())
            .ok_or_else(|| not_recorded(body["method"].as_str().unwrap_or("request")))
    }
}

impl HttpGet for Replay {
    async fn get_json(&self, url: &str) -> Result<Value, TransportError> {
        let path = self
            .dir
            .join("http")
            .join(format!("{}.json", http_key(url)));
        match load(&path) {
            Some(v) => Ok(v["response"].clone()),
            // A recorded failure (e.g. 404) replays as the same status.
            None => Err(TransportError::Status(404)),
        }
    }
}
