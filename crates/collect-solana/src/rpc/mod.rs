//! The one RPC client every read goes through (COL-1, COL-2, COL-5): a
//! token-bucket rate limit, a concurrency cap, retries with exponential
//! backoff and jitter, and a daily call budget with per-method counts.

mod http;
mod limiter;

use std::collections::BTreeMap;
use std::future::Future;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use serde::Serialize;
use serde_json::{Value, json};
use tokio::sync::Semaphore;

pub use http::HttpTransport;
use limiter::TokenBucket;

/// Sends one JSON-RPC request body and returns the response body.
pub trait Transport: Send + Sync {
    fn send(&self, body: Value) -> impl Future<Output = Result<Value, TransportError>> + Send;
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TransportError {
    #[error("HTTP status {0}")]
    Status(u16),
    #[error("request timed out")]
    Timeout,
    #[error("network error: {0}")]
    Network(String),
    #[error("response is not JSON: {0}")]
    Decode(String),
}

impl TransportError {
    /// 429, 5xx, timeouts and network errors are worth another attempt.
    fn retryable(&self) -> bool {
        match self {
            TransportError::Status(s) => *s == 429 || (500..=599).contains(s),
            TransportError::Timeout | TransportError::Network(_) => true,
            TransportError::Decode(_) => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RpcError {
    #[error("daily RPC budget of {budget} calls reached")]
    BudgetExceeded { budget: u64 },
    #[error("{method} failed after {attempts} attempts: {last}")]
    Exhausted {
        method: String,
        attempts: u32,
        last: TransportError,
    },
    #[error("{method} rejected with HTTP {status}")]
    Rejected { method: String, status: u16 },
    #[error("{method} returned RPC error {code}: {message}")]
    Rpc {
        method: String,
        code: i64,
        message: String,
    },
    #[error("{method}: {error}")]
    Malformed { method: String, error: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RpcConfig {
    /// Default 8, under the Helius free tier's 10 (COL-1).
    pub requests_per_sec: u32,
    /// Requests allowed back to back before the rate applies. 1 keeps every
    /// one-second window at or under `requests_per_sec`.
    pub burst: u32,
    pub max_concurrency: usize,
    /// Total attempts per call, including the first (COL-2).
    pub max_attempts: u32,
    pub base_backoff: Duration,
    pub max_backoff: Duration,
    pub timeout: Duration,
    /// Requests per run, counting retries (COL-5).
    pub daily_budget: u64,
    /// Seed for backoff jitter; fixed so runs are reproducible.
    pub jitter_seed: u64,
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            requests_per_sec: 8,
            burst: 1,
            max_concurrency: 4,
            max_attempts: 5,
            base_backoff: Duration::from_millis(500),
            max_backoff: Duration::from_secs(16),
            timeout: Duration::from_secs(30),
            daily_budget: 5_000,
            jitter_seed: 0x5eed_ce1e_d1a5_0001,
        }
    }
}

/// Calls made in this run, for `run.json` (COL-5).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct CallReport {
    /// Requests sent per method, retries included.
    pub calls: BTreeMap<String, u64>,
    pub retries: u64,
}

impl CallReport {
    pub fn total(&self) -> u64 {
        self.calls.values().sum()
    }
}

pub struct RpcClient<T> {
    transport: T,
    config: RpcConfig,
    limiter: TokenBucket,
    permits: Semaphore,
    used: AtomicU64,
    jitter: AtomicU64,
    report: Mutex<CallReport>,
}

impl<T: Transport> RpcClient<T> {
    pub fn new(transport: T, config: RpcConfig) -> Self {
        Self {
            limiter: TokenBucket::new(config.requests_per_sec, config.burst),
            permits: Semaphore::new(config.max_concurrency.max(1)),
            used: AtomicU64::new(0),
            jitter: AtomicU64::new(config.jitter_seed),
            report: Mutex::new(CallReport::default()),
            transport,
            config,
        }
    }

    pub fn transport(&self) -> &T {
        &self.transport
    }

    pub fn report(&self) -> CallReport {
        self.report.lock().expect("report lock").clone()
    }

    /// Calls `method` and returns the JSON-RPC `result`.
    pub async fn call(&self, method: &str, params: Value) -> Result<Value, RpcError> {
        let body = json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params});
        let mut attempt = 0;
        loop {
            attempt += 1;
            self.take_budget()?;
            self.count(method, attempt > 1);

            let outcome = {
                let _permit = self
                    .permits
                    .acquire()
                    .await
                    .expect("semaphore never closes");
                self.limiter.acquire().await;
                match tokio::time::timeout(self.config.timeout, self.transport.send(body.clone()))
                    .await
                {
                    Ok(r) => r,
                    Err(_) => Err(TransportError::Timeout),
                }
            };

            let error = match outcome {
                Ok(response) => match parse(method, response) {
                    Ok(result) => return Ok(result),
                    Err(Retry::No(e)) => return Err(e),
                    Err(Retry::Yes(e)) => e,
                },
                Err(TransportError::Status(status))
                    if !TransportError::Status(status).retryable() =>
                {
                    return Err(RpcError::Rejected {
                        method: method.to_string(),
                        status,
                    });
                }
                Err(e) if !e.retryable() => {
                    return Err(RpcError::Malformed {
                        method: method.to_string(),
                        error: e.to_string(),
                    });
                }
                Err(e) => e,
            };

            if attempt >= self.config.max_attempts {
                return Err(RpcError::Exhausted {
                    method: method.to_string(),
                    attempts: attempt,
                    last: error,
                });
            }
            tokio::time::sleep(self.backoff(attempt)).await;
        }
    }

    fn take_budget(&self) -> Result<(), RpcError> {
        let budget = self.config.daily_budget;
        self.used
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |u| {
                (u < budget).then_some(u + 1)
            })
            .map(|_| ())
            .map_err(|_| RpcError::BudgetExceeded { budget })
    }

    fn count(&self, method: &str, retry: bool) {
        let mut r = self.report.lock().expect("report lock");
        *r.calls.entry(method.to_string()).or_default() += 1;
        if retry {
            r.retries += 1;
        }
    }

    /// Equal jitter: half the exponential step, plus a random part of the other half.
    fn backoff(&self, attempt: u32) -> Duration {
        let step = self
            .config
            .base_backoff
            .saturating_mul(2u32.saturating_pow(attempt - 1))
            .min(self.config.max_backoff);
        let half = step / 2;
        let x = self.next_random();
        let extra_nanos = (half.as_nanos() as u64).checked_mul(x % 1_000).unwrap_or(0) / 1_000;
        half + Duration::from_nanos(extra_nanos)
    }

    /// xorshift64*, shared across callers so concurrent retries spread out.
    fn next_random(&self) -> u64 {
        let mut x = self.jitter.load(Ordering::Relaxed);
        loop {
            let mut y = x;
            y ^= y >> 12;
            y ^= y << 25;
            y ^= y >> 27;
            match self
                .jitter
                .compare_exchange_weak(x, y, Ordering::Relaxed, Ordering::Relaxed)
            {
                Ok(_) => return y.wrapping_mul(0x2545_f491_4f6c_dd1d),
                Err(current) => x = current,
            }
        }
    }
}

enum Retry {
    Yes(TransportError),
    No(RpcError),
}

/// JSON-RPC errors are final, except "node behind" and provider rate limits.
fn parse(method: &str, mut response: Value) -> Result<Value, Retry> {
    if let Some(err) = response.get("error") {
        let code = err.get("code").and_then(Value::as_i64).unwrap_or(0);
        let message = err
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        if matches!(code, -32005 | -32429) {
            return Err(Retry::Yes(TransportError::Network(message)));
        }
        return Err(Retry::No(RpcError::Rpc {
            method: method.to_string(),
            code,
            message,
        }));
    }
    match response.get_mut("result") {
        Some(result) => Ok(result.take()),
        None => Err(Retry::No(RpcError::Malformed {
            method: method.to_string(),
            error: "response has neither result nor error".into(),
        })),
    }
}
