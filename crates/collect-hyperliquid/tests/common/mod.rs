//! Synthetic HyperEVM fixtures for resolver and collector tests.
#![allow(dead_code)]

use celebtir_core::Address;
use celebtir_registry::{EvmController, EvmControllerKind, TimelockEntry};
use collect_hyperliquid::evm::hex;
use collect_hyperliquid::fixtures::{EvmCall, EvmFixture};
use collect_hyperliquid::resolve::{ADMIN_SLOT, EvmKnown, selector};
use serde_json::{Value, json};

pub const BLOCK: u64 = 1_000;
pub const PROXY: &str = "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed";
pub const ADMIN: &str = "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359";
pub const SAFE: &str = "0xdbF03B407c01E7cD3CBea99509d93f8DDDC8C6FB";
pub const KEY: &str = "0xD1220A0cf47c7B9Be7A2E6BA89F429762e7b9aDb";
pub const TIMELOCK: &str = "0x52908400098527886E0F7030069857D2E4169EE7";
pub const MODULE: &str = "0x8617E340B3D01FA5F11F306F4090FD50E238070D";
pub const SINGLETON: &str = "0x41675C099F32341bf84BFc5382aF534df5C7461a";

pub fn tag() -> String {
    format!("0x{BLOCK:x}")
}

pub fn word_of_address(a: &str) -> String {
    format!("0x{:0>64}", a.trim_start_matches("0x").to_ascii_lowercase())
}

pub fn uint(n: u64) -> String {
    format!("0x{n:064x}")
}

pub fn addresses(list: &[&str]) -> String {
    let mut s = format!("0x{:064x}{:064x}", 32, list.len());
    for a in list {
        s.push_str(&format!(
            "{:0>64}",
            a.trim_start_matches("0x").to_ascii_lowercase()
        ));
    }
    s
}

pub fn pad(a: &str) -> String {
    format!("{:0>64}", a.trim_start_matches("0x").to_ascii_lowercase())
}

/// Synthetic fixture builder.
#[derive(Default)]
pub struct Chain(pub Vec<EvmCall>);

impl Chain {
    pub fn put(mut self, method: &str, params: Value, result: Option<&str>) -> Self {
        self.0.push(EvmCall {
            method: method.into(),
            params,
            result: result.map(|r| json!(r)),
            error: result
                .is_none()
                .then(|| json!({"code": 3, "message": "execution reverted"})),
        });
        self
    }
    pub fn code(self, a: &str, code: &str) -> Self {
        self.put("eth_getCode", json!([a, tag()]), Some(code))
    }
    pub fn storage(self, a: &str, slot: &str, v: &str) -> Self {
        self.put("eth_getStorageAt", json!([a, slot, tag()]), Some(v))
    }
    pub fn call(self, to: &str, data: &str, out: Option<&str>) -> Self {
        self.put("eth_call", json!([{"to": to, "data": data}, tag()]), out)
    }
    pub fn sig(self, to: &str, sig: &str, args: &str, out: Option<&str>) -> Self {
        let data = format!("{}{args}", hex(&selector(sig)));
        self.call(to, &data, out)
    }
    pub fn contract(self, a: &str) -> Self {
        self.code(a, "0x6080")
            .storage(a, &format!("0x{}", "00".repeat(32)), &uint(0))
    }
    pub fn safe(self, a: &str, threshold: u64, owners: &[&str], modules: &[&str]) -> Self {
        self.code(a, "0x6080")
            .storage(
                a,
                &format!("0x{}", "00".repeat(32)),
                &word_of_address(SINGLETON),
            )
            .sig(a, "getThreshold()", "", Some(&uint(threshold)))
            .sig(a, "getOwners()", "", Some(&addresses(owners)))
            .sig(
                a,
                "getModulesPaginated(address,uint256)",
                &format!(
                    "{}{:064x}",
                    pad("0x0000000000000000000000000000000000000001"),
                    10
                ),
                Some(&format!("{}{}", addresses(modules), "00".repeat(32))),
            )
    }
    pub fn fixture(self) -> EvmFixture {
        EvmFixture {
            host: "synthetic".into(),
            chain_id: 999,
            block: BLOCK,
            calls: self.0,
            note: "synthetic".into(),
        }
    }
}

pub fn known(timelocks: &[TimelockEntry]) -> EvmKnown {
    EvmKnown::new(
        &[EvmController {
            address: Address::new(SINGLETON),
            kind: EvmControllerKind::SafeSingleton,
            name: "safe-1.4.1".into(),
            source: "https://github.com/safe-global/safe-deployments"
                .parse()
                .unwrap(),
        }],
        timelocks,
    )
}

pub fn admin_slot() -> String {
    format!("0x{ADMIN_SLOT}")
}
