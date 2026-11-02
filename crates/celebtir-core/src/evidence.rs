use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use url::Url;

/// The chain an address or fact lives on.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Chain {
    Solana,
    /// Wave 2.
    HyperEvm,
    /// Wave 2.
    HyperCore,
}

/// An on-chain address in its chain's canonical text form (base58 on Solana,
/// 0x-hex on EVM). Format checks belong to the registry and collectors.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(transparent)]
pub struct Address(String);

impl Address {
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Where a fact came from: the account read, the slot it was read at, the
/// field that held the value, and a link anyone can open to check it.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
pub struct Evidence {
    pub chain: Chain,
    pub address: Address,
    pub slot: u64,
    pub field: String,
    pub explorer_url: Url,
}

impl Evidence {
    pub fn new(chain: Chain, address: Address, slot: u64, field: impl Into<String>) -> Self {
        let explorer_url = explorer_url(chain, &address);
        Self {
            chain,
            address,
            slot,
            field: field.into(),
            explorer_url,
        }
    }

    /// Evidence for a transaction: the account it touched, its slot, and a
    /// link to the transaction itself (CHG-1).
    pub fn transaction(
        chain: Chain,
        address: Address,
        slot: u64,
        signature: &str,
        field: impl Into<String>,
    ) -> Self {
        let base = match chain {
            Chain::Solana => "https://solscan.io/tx/",
            Chain::HyperEvm => "https://hyperevmscan.io/tx/",
            Chain::HyperCore => "https://app.hyperliquid.xyz/explorer/tx/",
        };
        let mut explorer_url = Url::parse(base).expect("explorer base URLs are valid");
        explorer_url
            .path_segments_mut()
            .expect("explorer base URLs have a path")
            .pop_if_empty()
            .push(signature);
        Self {
            chain,
            address,
            slot,
            field: field.into(),
            explorer_url,
        }
    }
}

fn explorer_url(chain: Chain, address: &Address) -> Url {
    let base = match chain {
        Chain::Solana => "https://solscan.io/account/",
        Chain::HyperEvm => "https://hyperevmscan.io/address/",
        Chain::HyperCore => "https://app.hyperliquid.xyz/explorer/address/",
    };
    let mut url = Url::parse(base).expect("explorer base URLs are valid");
    url.path_segments_mut()
        .expect("explorer base URLs have a path")
        .pop_if_empty()
        .push(address.as_str());
    url
}
