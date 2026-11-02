//! VAL-6: daily volume and fees per protocol from DefiLlama, with 7-day
//! change; protocols without a volume metric show fees only.

use celebtir_core::snapshot::Activity;
use collect_solana::defillama::{ActivityKind, activity};
use collect_solana::rpc::TransportError;
use collect_solana::verify::HttpGet;
use serde_json::{Value, json};

const DAY: i64 = 86_400;
/// 2026-10-17T00:00:00Z.
const TODAY: i64 = 1_792_195_200;

struct Fixed(Result<Value, TransportError>, std::sync::Mutex<Vec<String>>);

impl Fixed {
    fn new(r: Result<Value, TransportError>) -> Self {
        Self(r, std::sync::Mutex::new(Vec::new()))
    }
}

impl HttpGet for Fixed {
    async fn get_json(&self, url: &str) -> Result<Value, TransportError> {
        self.1.lock().unwrap().push(url.to_string());
        self.0.clone()
    }
}

/// 40 complete days valued 100, 101, …, then today's partial day (5).
fn summary() -> Value {
    let mut chart: Vec<Value> = (0..40)
        .map(|i| json!([TODAY - (40 - i) * DAY, 100 + i]))
        .collect();
    chart.push(json!([TODAY, 5]));
    json!({"total24h": 139, "totalDataChart": chart})
}

#[tokio::test]
async fn val_6_fees_cover_the_last_30_complete_days_with_7_day_change() {
    let http = Fixed::new(Ok(summary()));
    let a = activity(&http, "kamino-lend", ActivityKind::Fees).await;
    match a {
        Activity::Usd {
            days,
            latest,
            change_7d_bp,
            ..
        } => {
            assert_eq!(days.len(), 30);
            assert_eq!(
                days.last().unwrap().date,
                "2026-10-16",
                "today's partial day is dropped"
            );
            assert_eq!(days.first().unwrap().date, "2026-09-17");
            assert_eq!(latest, 139);
            // last 7 days: 133..=139 (sum 952); previous 7: 126..=132 (sum 903).
            assert_eq!(change_7d_bp, Some((952 - 903) * 10_000 / 903));
        }
        other => panic!("{other:?}"),
    }
    assert!(http.1.lock().unwrap()[0].ends_with("/summary/fees/kamino-lend?dataType=dailyFees"));
}

#[tokio::test]
async fn val_6_dex_volume_uses_the_dexs_summary() {
    let http = Fixed::new(Ok(summary()));
    assert!(matches!(
        activity(&http, "orca", ActivityKind::Volume).await,
        Activity::Usd { .. }
    ));
    assert!(http.1.lock().unwrap()[0].ends_with("/summary/dexs/orca?dataType=dailyVolume"));
}

#[tokio::test]
async fn val_6_missing_or_short_series_is_unavailable() {
    let none = activity(
        &Fixed::new(Err(TransportError::Status(404))),
        "x",
        ActivityKind::Fees,
    )
    .await;
    assert!(matches!(none, Activity::Unavailable { .. }));
    let empty = activity(
        &Fixed::new(Ok(json!({"totalDataChart": []}))),
        "x",
        ActivityKind::Fees,
    )
    .await;
    assert!(matches!(empty, Activity::Unavailable { .. }));
}

#[tokio::test]
async fn val_6_week_over_week_with_a_zero_week_has_no_change() {
    let chart: Vec<Value> = (0..14)
        .map(|i| json!([TODAY - (14 - i) * DAY, if i < 7 { 0 } else { 10 }]))
        .collect();
    let a = activity(
        &Fixed::new(Ok(json!({"total24h": 10, "totalDataChart": chart}))),
        "x",
        ActivityKind::Fees,
    )
    .await;
    assert!(
        matches!(
            a,
            Activity::Usd {
                change_7d_bp: None,
                ..
            }
        ),
        "{a:?}"
    );
}

#[test]
fn val_6_dates_are_utc_calendar_days() {
    assert_eq!(celebtir_core::date::ymd(TODAY), "2026-10-17");
    assert_eq!(celebtir_core::date::ymd(TODAY - 1), "2026-10-16");
}
