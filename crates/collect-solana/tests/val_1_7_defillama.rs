//! VAL-1 (daily TVL per protocol from DefiLlama, stored in the snapshot) and
//! VAL-7 (a failed fetch hides the number; it never fails the run or
//! changes a rating).

use std::collections::VecDeque;
use std::sync::Mutex;

use celebtir_core::snapshot::Metric;
use collect_solana::defillama::{DEFILLAMA_API, protocol_metrics};
use collect_solana::rpc::TransportError;
use collect_solana::verify::HttpGet;
use serde_json::{Value, json};

struct Llama {
    replies: Mutex<VecDeque<Result<Value, TransportError>>>,
    urls: Mutex<Vec<String>>,
}

impl Llama {
    fn new(replies: Vec<Result<Value, TransportError>>) -> Self {
        Self {
            replies: Mutex::new(replies.into()),
            urls: Mutex::new(Vec::new()),
        }
    }
}

impl HttpGet for Llama {
    async fn get_json(&self, url: &str) -> Result<Value, TransportError> {
        self.urls.lock().unwrap().push(url.to_string());
        self.replies
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or(Err(TransportError::Status(404)))
    }
}

#[tokio::test]
async fn val_1_tvl_comes_from_defillama_in_whole_dollars() {
    let http = Llama::new(vec![Ok(json!(1_310_434_543.0003347))]);
    let m = protocol_metrics(&http, Some("kamino-lend"), false).await;
    assert_eq!(m.defillama_slug.as_deref(), Some("kamino-lend"));
    assert_eq!(
        m.tvl,
        Metric::Usd {
            value: 1_310_434_543
        }
    );
    assert_eq!(
        http.urls.lock().unwrap()[0],
        format!("{DEFILLAMA_API}/tvl/kamino-lend")
    );
}

#[tokio::test(start_paused = true)]
async fn val_7_outage_hides_the_number_and_does_not_fail() {
    let http = Llama::new(vec![Err(TransportError::Status(503)); 3]);
    let m = protocol_metrics(&http, Some("kamino-lend"), false).await;
    assert!(
        matches!(&m.tvl, Metric::Unavailable { reason } if reason.contains("503")),
        "{:?}",
        m.tvl
    );
}

#[tokio::test]
async fn val_7_missing_slug_or_nonsense_value_is_unavailable() {
    let http = Llama::new(vec![]);
    let m = protocol_metrics(&http, None, false).await;
    assert!(matches!(&m.tvl, Metric::Unavailable { reason } if reason.contains("slug")));
    assert!(http.urls.lock().unwrap().is_empty());

    for bad in [json!("lots"), json!(-5.0), json!({"tvl": 1})] {
        let http = Llama::new(vec![Ok(bad.clone())]);
        let m = protocol_metrics(&http, Some("x"), false).await;
        assert!(matches!(m.tvl, Metric::Unavailable { .. }), "{bad}");
    }
}
