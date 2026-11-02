use std::fmt;

use schemars::JsonSchema;
use serde::{Serialize, Serializer};
use url::Url;

/// An RPC URL that only ever shows its host (SNAP-5). Provider keys live in
/// the query string or the path, so Display, Debug and Serialize all print
/// the host alone; only [`RpcEndpoint::url`] returns the full URL, for making
/// requests.
#[derive(Clone, PartialEq, Eq)]
pub struct RpcEndpoint {
    url: String,
    host: String,
}

impl RpcEndpoint {
    pub fn new(url: impl Into<String>) -> Self {
        let url = url.into();
        let host = Url::parse(&url)
            .ok()
            .and_then(|u| u.host_str().map(str::to_string))
            .unwrap_or_else(|| "invalid-rpc-url".to_string());
        Self { url, host }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn host(&self) -> &str {
        &self.host
    }
}

impl fmt::Display for RpcEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.host)
    }
}

impl fmt::Debug for RpcEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("RpcEndpoint").field(&self.host).finish()
    }
}

impl Serialize for RpcEndpoint {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.host)
    }
}

impl JsonSchema for RpcEndpoint {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "RpcHost".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        String::json_schema(generator)
    }
}
