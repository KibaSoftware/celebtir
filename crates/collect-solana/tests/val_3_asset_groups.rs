//! VAL-3: TVL split by asset group (stablecoins, SOL and liquid staking
//! tokens, BTC, other) where DefiLlama's per-token breakdown allows.

use celebtir_core::snapshot::{AssetGroups, Metric};
use collect_solana::defillama::{AssetGroup, asset_groups, classify};
use collect_solana::rpc::TransportError;
use collect_solana::verify::HttpGet;
use serde_json::{Value, json};

struct Fixed(Result<Value, TransportError>);

impl HttpGet for Fixed {
    async fn get_json(&self, _: &str) -> Result<Value, TransportError> {
        self.0.clone()
    }
}

fn kamino() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/http/defillama/kamino-lend-protocol.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

const KAMINO_TVL: u64 = 1_310_434_543;

#[test]
fn val_3_symbols_are_grouped() {
    for s in [
        "USDC",
        "USDT",
        "PYUSD",
        "USDE",
        "SYRUPUSDC",
        "SUSDAI",
        "REUSD",
        "EURC",
        "FDUSD",
    ] {
        assert_eq!(classify(s), AssetGroup::Stablecoins, "{s}");
    }
    for s in [
        "SOL", "WSOL", "JITOSOL", "MSOL", "DSOL", "BONKSOL", "INF", "JUPSOL",
    ] {
        assert_eq!(classify(s), AssetGroup::Sol, "{s}");
    }
    for s in ["CBBTC", "WBTC", "XBTC", "FBTC", "ZBTC"] {
        assert_eq!(classify(s), AssetGroup::Btc, "{s}");
    }
    for s in ["JLP", "ONYC", "JUP", "BONK", "WETH"] {
        assert_eq!(classify(s), AssetGroup::Other, "{s}");
    }
}

#[tokio::test]
async fn val_3_lending_protocol_shows_four_groups_summing_to_its_tvl() {
    let groups = asset_groups(
        &Fixed(Ok(kamino())),
        "kamino-lend",
        &Metric::Usd { value: KAMINO_TVL },
    )
    .await;
    match groups {
        AssetGroups::Usd {
            stablecoins,
            sol,
            btc,
            other,
        } => {
            assert_eq!(stablecoins + sol + btc + other, KAMINO_TVL);
            assert!(
                sol > btc && stablecoins > 0 && btc > 0 && other > 0,
                "{stablecoins} {sol} {btc} {other}"
            );
        }
        other => panic!("{other:?}"),
    }
}

#[tokio::test]
async fn val_3_breakdown_far_from_tvl_is_unavailable() {
    let groups = asset_groups(
        &Fixed(Ok(kamino())),
        "kamino-lend",
        &Metric::Usd {
            value: KAMINO_TVL * 2,
        },
    )
    .await;
    assert!(
        matches!(&groups, AssetGroups::Unavailable { reason } if reason.contains("differs")),
        "{groups:?}"
    );
}

#[tokio::test]
async fn val_3_missing_breakdown_or_outage_is_unavailable() {
    let no_breakdown = asset_groups(
        &Fixed(Ok(json!({"chainTvls": {}}))),
        "x",
        &Metric::Usd { value: 10 },
    )
    .await;
    assert!(matches!(no_breakdown, AssetGroups::Unavailable { .. }));
    let outage = asset_groups(
        &Fixed(Err(TransportError::Status(404))),
        "x",
        &Metric::Usd { value: 10 },
    )
    .await;
    assert!(matches!(outage, AssetGroups::Unavailable { .. }));
    let no_tvl = asset_groups(
        &Fixed(Ok(kamino())),
        "x",
        &Metric::Unavailable {
            reason: "down".into(),
        },
    )
    .await;
    assert!(
        matches!(no_tvl, AssetGroups::Unavailable { .. }),
        "groups need a TVL to sum to"
    );
}

#[tokio::test]
async fn val_3_a_group_netted_below_zero_is_unavailable() {
    let breakdown = json!({"chainTvls": {"Solana": {"tokensInUsd": [{"date": 1, "tokens": {"USDC": -50.0, "SOL": 100.0}}]}}});
    let groups = asset_groups(&Fixed(Ok(breakdown)), "x", &Metric::Usd { value: 50 }).await;
    assert!(
        matches!(&groups, AssetGroups::Unavailable { reason } if reason.contains("below zero")),
        "{groups:?}"
    );
}
