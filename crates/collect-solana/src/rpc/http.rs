use celebtir_core::RpcEndpoint;
use serde_json::Value;

use super::{Transport, TransportError};

/// JSON-RPC over HTTPS. Errors never include the URL, which may carry a key
/// (SNAP-5).
pub struct HttpTransport {
    client: reqwest::Client,
    endpoint: RpcEndpoint,
}

impl HttpTransport {
    pub fn new(endpoint: RpcEndpoint) -> Self {
        Self {
            client: reqwest::Client::new(),
            endpoint,
        }
    }

    pub fn endpoint(&self) -> &RpcEndpoint {
        &self.endpoint
    }
}

impl Transport for HttpTransport {
    async fn send(&self, body: Value) -> Result<Value, TransportError> {
        let response = self
            .client
            .post(self.endpoint.url())
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    TransportError::Timeout
                } else {
                    TransportError::Network(e.without_url().to_string())
                }
            })?;
        let status = response.status().as_u16();
        if !(200..300).contains(&status) {
            return Err(TransportError::Status(status));
        }
        response
            .json::<Value>()
            .await
            .map_err(|e| TransportError::Decode(e.without_url().to_string()))
    }
}
