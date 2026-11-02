//! TOK-9: token prices from a free source (DefiLlama coins), in one call.

use celebtir_core::Address;
use celebtir_core::snapshot::TokenMetrics;
use collect_solana::defillama::token_prices;
use collect_solana::rpc::TransportError;
use collect_solana::verify::HttpGet;
use serde_json::{Value, json};

struct Fixed(Result<Value, TransportError>, std::sync::Mutex<Vec<String>>);

impl HttpGet for Fixed {
    async fn get_json(&self, url: &str) -> Result<Value, TransportError> {
        self.1.lock().unwrap().push(url.to_string());
        self.0.clone()
    }
}

const USDC: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
const JITOSOL: &str = "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn";

#[tokio::test]
async fn tok_9_prices_come_from_defillama_coins_in_one_call() {
    let body = json!({"coins": {
        format!("solana:{USDC}"): {"price": 0.9996530288961996, "decimals": 6},
        format!("solana:{JITOSOL}"): {"price": 135.4365871758744, "decimals": 9}
    }});
    let http = Fixed(Ok(body), Default::default());
    let mints = [
        Address::new(USDC),
        Address::new(JITOSOL),
        Address::new("Unpriced111111111111111111111111111111111111"),
    ];
    let prices = token_prices(&http, &mints).await;
    assert_eq!(
        prices[&mints[0]],
        TokenMetrics::Price {
            usd_micros: 999_653
        }
    );
    assert_eq!(
        prices[&mints[1]],
        TokenMetrics::Price {
            usd_micros: 135_436_587
        }
    );
    assert!(matches!(
        prices[&mints[2]],
        TokenMetrics::Unavailable { .. }
    ));
    let urls = http.1.lock().unwrap();
    assert_eq!(urls.len(), 1);
    assert!(urls[0].starts_with("https://coins.llama.fi/prices/current/solana:"));
}

#[tokio::test(start_paused = true)]
async fn tok_9_price_outage_marks_every_token_unavailable() {
    let http = Fixed(Err(TransportError::Status(503)), Default::default());
    let mints = [Address::new(USDC)];
    let prices = token_prices(&http, &mints).await;
    assert!(
        matches!(&prices[&mints[0]], TokenMetrics::Unavailable { reason } if reason.contains("503"))
    );
}
