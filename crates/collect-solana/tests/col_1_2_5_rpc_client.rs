//! COL-1 (rate limit + concurrency cap), COL-2 (retries with backoff),
//! COL-5 (calls per method, daily budget). Runs on tokio's paused clock with
//! a scripted transport, so it is fast and deterministic.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use collect_solana::rpc::{RpcClient, RpcConfig, RpcError, Transport, TransportError};
use serde_json::{Value, json};
use tokio::task::JoinSet;
use tokio::time::Instant;

/// Replies from a script (then `{"result": null}` forever) and records when
/// each request was sent and how many were in flight at once.
#[derive(Default)]
struct Mock {
    script: Mutex<VecDeque<Result<Value, TransportError>>>,
    sent: Mutex<Vec<(Instant, String)>>,
    in_flight: AtomicUsize,
    max_in_flight: AtomicUsize,
    latency: Duration,
}

impl Mock {
    fn scripted(replies: Vec<Result<Value, TransportError>>) -> Self {
        Self {
            script: Mutex::new(replies.into()),
            ..Default::default()
        }
    }

    fn with_latency(latency: Duration) -> Self {
        Self {
            latency,
            ..Default::default()
        }
    }

    fn times(&self) -> Vec<Instant> {
        self.sent.lock().unwrap().iter().map(|(t, _)| *t).collect()
    }
}

impl Transport for Mock {
    async fn send(&self, body: Value) -> Result<Value, TransportError> {
        let method = body["method"].as_str().unwrap_or_default().to_string();
        self.sent.lock().unwrap().push((Instant::now(), method));
        let now = self.in_flight.fetch_add(1, Ordering::SeqCst) + 1;
        self.max_in_flight.fetch_max(now, Ordering::SeqCst);
        if !self.latency.is_zero() {
            tokio::time::sleep(self.latency).await;
        }
        self.in_flight.fetch_sub(1, Ordering::SeqCst);
        self.script
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| Ok(json!({"jsonrpc": "2.0", "id": 1, "result": null})))
    }
}

fn ok(result: Value) -> Result<Value, TransportError> {
    Ok(json!({"jsonrpc": "2.0", "id": 1, "result": result}))
}

fn config() -> RpcConfig {
    RpcConfig::default()
}

#[tokio::test(start_paused = true)]
async fn col_1_thousand_calls_never_exceed_the_configured_rate() {
    let cfg = RpcConfig {
        max_concurrency: 16,
        ..config()
    };
    assert_eq!(cfg.requests_per_sec, 8, "default is 8/s, under Helius's 10");
    let client = Arc::new(RpcClient::new(Mock::default(), cfg));
    let mut set = JoinSet::new();
    for _ in 0..1000 {
        let c = client.clone();
        set.spawn(async move { c.call("getSlot", json!([])).await.unwrap() });
    }
    while let Some(r) = set.join_next().await {
        r.unwrap();
    }
    let mut times = client.transport().times();
    times.sort();
    assert_eq!(times.len(), 1000);
    for (i, start) in times.iter().enumerate() {
        let in_window = times[i..]
            .iter()
            .take_while(|t| **t < *start + Duration::from_secs(1))
            .count();
        assert!(
            in_window <= 8,
            "{in_window} requests within 1 s starting at call {i}"
        );
    }
    let span = *times.last().unwrap() - times[0];
    assert!(
        span <= Duration::from_secs(125),
        "limiter is too slow: {span:?}"
    );
}

#[tokio::test(start_paused = true)]
async fn col_1_concurrency_is_capped() {
    let cfg = RpcConfig {
        max_concurrency: 3,
        requests_per_sec: 1000,
        ..config()
    };
    let client = Arc::new(RpcClient::new(
        Mock::with_latency(Duration::from_millis(500)),
        cfg,
    ));
    let mut set = JoinSet::new();
    for _ in 0..20 {
        let c = client.clone();
        set.spawn(async move { c.call("getSlot", json!([])).await.unwrap() });
    }
    while set.join_next().await.is_some() {}
    assert_eq!(client.transport().max_in_flight.load(Ordering::SeqCst), 3);
}

#[tokio::test(start_paused = true)]
async fn col_2_mocked_429_succeeds_on_retry() {
    let mock = Mock::scripted(vec![Err(TransportError::Status(429)), ok(json!(42))]);
    let client = RpcClient::new(mock, config());
    let started = Instant::now();
    assert_eq!(client.call("getSlot", json!([])).await.unwrap(), json!(42));
    assert!(
        Instant::now() - started > Duration::ZERO,
        "a retry waits before resending"
    );
    let report = client.report();
    assert_eq!(report.calls["getSlot"], 2);
    assert_eq!(report.retries, 1);
}

#[tokio::test(start_paused = true)]
async fn col_2_5xx_and_timeouts_are_retried_up_to_5_attempts() {
    let mock = Mock::scripted(vec![
        Err(TransportError::Status(503)),
        Err(TransportError::Timeout),
        Err(TransportError::Network("reset".into())),
        Err(TransportError::Status(502)),
        Err(TransportError::Status(500)),
        ok(json!("never reached")),
    ]);
    let client = RpcClient::new(mock, config());
    let err = client.call("getAccountInfo", json!([])).await.unwrap_err();
    assert!(
        matches!(err, RpcError::Exhausted { attempts: 5, .. }),
        "{err:?}"
    );
    assert_eq!(client.report().calls["getAccountInfo"], 5);
}

#[tokio::test(start_paused = true)]
async fn col_2_mocked_400_fails_at_once_and_the_client_keeps_working() {
    let mock = Mock::scripted(vec![Err(TransportError::Status(400)), ok(json!(7))]);
    let client = RpcClient::new(mock, config());
    let err = client.call("getAccountInfo", json!([])).await.unwrap_err();
    assert!(
        matches!(err, RpcError::Rejected { status: 400, .. }),
        "{err:?}"
    );
    assert_eq!(
        client.report().calls["getAccountInfo"],
        1,
        "no retry on 400"
    );
    assert_eq!(client.call("getSlot", json!([])).await.unwrap(), json!(7));
}

#[tokio::test(start_paused = true)]
async fn col_2_json_rpc_errors_are_reported_not_retried() {
    let mock = Mock::scripted(vec![Ok(
        json!({"jsonrpc": "2.0", "id": 1, "error": {"code": -32602, "message": "Invalid param"}}),
    )]);
    let client = RpcClient::new(mock, config());
    let err = client.call("getAccountInfo", json!([])).await.unwrap_err();
    assert!(matches!(err, RpcError::Rpc { code: -32602, .. }), "{err:?}");
}

#[tokio::test(start_paused = true)]
async fn col_2_backoff_grows_between_attempts() {
    let mock = Mock::scripted(vec![
        Err(TransportError::Status(429)),
        Err(TransportError::Status(429)),
        Err(TransportError::Status(429)),
        ok(json!(1)),
    ]);
    let cfg = RpcConfig {
        requests_per_sec: 1000,
        ..config()
    };
    let client = RpcClient::new(mock, cfg);
    client.call("getSlot", json!([])).await.unwrap();
    let t = client.transport().times();
    let gaps: Vec<Duration> = t.windows(2).map(|w| w[1] - w[0]).collect();
    // Equal jitter: each wait is at least half of base * 2^(attempt-1).
    let base = RpcConfig::default().base_backoff;
    for (i, g) in gaps.iter().enumerate() {
        assert!(*g >= base * 2u32.pow(i as u32) / 2, "gap {i} = {g:?}");
    }
}

#[tokio::test(start_paused = true)]
async fn col_5_budget_of_10_halts_cleanly_with_a_budget_error() {
    let client = RpcClient::new(
        Mock::default(),
        RpcConfig {
            daily_budget: 10,
            ..config()
        },
    );
    for _ in 0..10 {
        client.call("getAccountInfo", json!([])).await.unwrap();
    }
    let err = client.call("getAccountInfo", json!([])).await.unwrap_err();
    assert!(
        matches!(err, RpcError::BudgetExceeded { budget: 10 }),
        "{err:?}"
    );
    assert_eq!(
        client.transport().times().len(),
        10,
        "the 11th call is never sent"
    );
}

#[tokio::test(start_paused = true)]
async fn col_5_report_counts_calls_per_method() {
    let client = RpcClient::new(Mock::default(), config());
    for _ in 0..3 {
        client.call("getMultipleAccounts", json!([])).await.unwrap();
    }
    client.call("getSlot", json!([])).await.unwrap();
    let r = client.report();
    assert_eq!(r.calls["getMultipleAccounts"], 3);
    assert_eq!(r.calls["getSlot"], 1);
    assert_eq!(r.total(), 4);
}

#[tokio::test(start_paused = true)]
async fn col_1_requests_are_json_rpc_2() {
    let client = RpcClient::new(Mock::default(), config());
    client
        .call("getSlot", json!([{"commitment": "finalized"}]))
        .await
        .unwrap();
    let sent = client.transport().sent.lock().unwrap().clone();
    assert_eq!(sent[0].1, "getSlot");
}
