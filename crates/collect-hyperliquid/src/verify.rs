//! HCOL-8: verified source for HyperEVM contracts (ADR 024 D5). Sourcify
//! first (keyless); then the Etherscan v2 API for chain 999 when a key is
//! set. For a proxy, the implementation behind the EIP-1967 slot is checked.
//! The Etherscan key goes into the request only, never into a fact or an
//! error message (SNAP-5).

use celebtir_core::snapshot::CodeTransparency;
use celebtir_core::{Address, Chain, Evidence, Fact};
use celebtir_registry::ProxyKind;
use collect_solana::rpc::Transport;
use collect_solana::verify::{HttpGet, get_with_retry};
use serde_json::Value;

use crate::evm::HyperEvm;
use crate::resolve::{IMPLEMENTATION_SLOT, word_address};

pub const SOURCIFY: &str = "https://sourcify.dev/server/v2/contract/999";
pub const ETHERSCAN: &str = "https://api.etherscan.io/v2/api";

/// The address whose code runs: the implementation for a proxy, else itself.
pub async fn code_address<T: Transport>(
    evm: &HyperEvm<T>,
    contract: &Address,
    proxy: ProxyKind,
) -> Result<Address, String> {
    if proxy == ProxyKind::None {
        return Ok(contract.clone());
    }
    let mut slot = [0u8; 32];
    for (i, b) in slot.iter_mut().enumerate() {
        *b = u8::from_str_radix(&IMPLEMENTATION_SLOT[2 * i..2 * i + 2], 16).expect("constant");
    }
    let w = evm
        .storage(contract.as_str(), &slot)
        .await
        .map_err(|e| e.to_string())?;
    if w.iter().all(|b| *b == 0) {
        return Err("implementation slot empty".into());
    }
    Ok(word_address(&w))
}

enum Check {
    Verified(String),
    NotVerified,
    Failed(String),
}

async fn sourcify<H: HttpGet>(http: &H, code: &Address) -> Check {
    match get_with_retry(http, &format!("{SOURCIFY}/{code}")).await {
        Ok(v) => match v.get("match").and_then(Value::as_str) {
            Some(m) => Check::Verified(m.to_string()),
            None => Check::NotVerified,
        },
        // Sourcify answers 404 for an unknown contract.
        Err(collect_solana::rpc::TransportError::Status(404)) => Check::NotVerified,
        Err(e) => Check::Failed(format!("Sourcify: {e}")),
    }
}

async fn etherscan<H: HttpGet>(http: &H, code: &Address, key: &str) -> Check {
    let url = format!(
        "{ETHERSCAN}?chainid=999&module=contract&action=getsourcecode&address={code}&apikey={key}"
    );
    match get_with_retry(http, &url).await {
        Ok(v) => {
            let source = v["result"][0]["SourceCode"].as_str().unwrap_or_default();
            if v["status"] == "1" && !source.is_empty() {
                Check::Verified("etherscan".into())
            } else if v["status"] == "1" {
                Check::NotVerified
            } else {
                // The message can echo the request; keep a fixed text.
                Check::Failed("Etherscan: request refused".into())
            }
        }
        Err(e) => Check::Failed(format!("Etherscan: {e}")),
    }
}

/// HCOL-8 for one contract, at the run's block.
pub async fn verified_source<T: Transport, H: HttpGet>(
    evm: &HyperEvm<T>,
    http: &H,
    etherscan_key: Option<&str>,
    contract: &Address,
    proxy: ProxyKind,
) -> Fact<CodeTransparency> {
    let ev =
        |a: &Address, field: &str| Evidence::new(Chain::HyperEvm, a.clone(), evm.block(), field);
    let code = match code_address(evm, contract, proxy).await {
        Ok(c) => c,
        Err(reason) => {
            return Fact::single(
                CodeTransparency::Unavailable { reason },
                ev(contract, "eip1967.implementation"),
            );
        }
    };
    let mut failures = Vec::new();
    let mut checked_unverified = Vec::new();
    for (name, check) in [
        ("sourcify", Some(sourcify(http, &code).await)),
        (
            "etherscan",
            match etherscan_key {
                Some(k) => Some(etherscan(http, &code, k).await),
                None => None,
            },
        ),
    ] {
        match check {
            Some(Check::Verified(how)) => {
                let repo = if name == "sourcify" {
                    format!("https://repo.sourcify.dev/999/{code}")
                } else {
                    format!("https://hyperevmscan.io/address/{code}#code")
                };
                return Fact::single(
                    CodeTransparency::Verified {
                        repo,
                        commit: format!("{name}:{how}"),
                        on_chain_hash: String::new(),
                        executable_hash: String::new(),
                        warning: None,
                    },
                    ev(&code, &format!("{name} verified source")),
                );
            }
            Some(Check::NotVerified) => checked_unverified.push(name),
            Some(Check::Failed(e)) => failures.push(e),
            None => {}
        }
    }
    // Only a source that answered "not verified" counts as an answer; with
    // Etherscan unchecked, a Sourcify miss alone stays Unknown (ADR 024 D5).
    let value = if checked_unverified.len() == 2 {
        CodeTransparency::NoPublicSource
    } else {
        let mut why = failures;
        if checked_unverified.contains(&"sourcify") {
            why.push("not verified on Sourcify".into());
        }
        if etherscan_key.is_none() {
            why.push("Etherscan not checked (no CELEBTIR_ETHERSCAN_KEY)".into());
        }
        CodeTransparency::Unavailable {
            reason: why.join("; "),
        }
    };
    Fact::single(value, ev(&code, "verified source"))
}
