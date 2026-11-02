//! The feeds behind a Kamino Scope price (edge case 11, slice 3.17).
//!
//! A Kamino reserve names the Scope price account it reads and a chain of up
//! to four Scope entry indices. Scope's `OracleMappings` account maps each
//! entry to a source account, a price type and 20 bytes of type-specific data.
//! `MostRecentOf` and `CappedMostRecentOf` entries take the most recent of up
//! to four other entries; their data starts with those entries as
//! `[u16; 4]` (scope `programs/scope/src/oracles/most_recent_of.rs`). Every
//! other entry is a leaf, listed with Scope's name for its type.

use std::collections::BTreeSet;

use celebtir_core::snapshot::{UnderlyingFeed, UnderlyingFeeds};
use celebtir_core::{Address, Chain, Evidence, Fact};

use crate::idl::Idl;
use crate::pda;
use crate::reads::SolanaRpc;
use crate::rpc::{RpcError, Transport};

/// Scope's "no entry" marker in a chain (`u16::MAX`), and its table size.
const NO_ENTRY: u16 = u16::MAX;
const MAX_ENTRIES: u16 = 512;
/// Nested `MostRecentOf` levels followed before giving up.
const MAX_DEPTH: usize = 3;
const REFERENCE_TYPES: [&str; 2] = ["MostRecentOf", "CappedMostRecentOf"];

/// How to find the chain for one market, from the registry.
pub struct ScopeChainSpec<'a> {
    /// The account holding the chain (Kamino: the reserve).
    pub chain_account: Address,
    pub chain_idl: &'a Idl,
    /// Field holding the Scope price account; must be the registry's oracle.
    pub feed_field: &'a str,
    /// Field holding the `[u16; N]` chain of Scope entries.
    pub chain_field: &'a str,
    pub scope_idl: &'a Idl,
}

pub struct ScopeResult {
    pub fact: Fact<UnderlyingFeeds>,
    /// Registry problems, e.g. the reserve now reads another price account.
    pub review: Vec<String>,
}

fn unavailable(reason: String, evidence: Vec<Evidence>) -> ScopeResult {
    ScopeResult {
        fact: Fact::new(UnderlyingFeeds::Unavailable { reason }, evidence)
            .expect("evidence is never empty"),
        review: Vec::new(),
    }
}

/// Reads the reserve, the Scope price account and Scope's mappings, and lists
/// the leaf feeds behind the reserve's chain.
pub async fn read_underlying<T: Transport>(
    rpc: &SolanaRpc<T>,
    oracle: &Address,
    spec: &ScopeChainSpec<'_>,
) -> Result<ScopeResult, RpcError> {
    let reads = rpc
        .get_multiple_accounts(&[spec.chain_account.clone(), oracle.clone()], None)
        .await?;
    let (chain_read, prices_read) = (&reads[0], &reads[1]);
    let mut evidence = vec![Evidence::new(
        Chain::Solana,
        spec.chain_account.clone(),
        chain_read.slot,
        spec.chain_field,
    )];
    let Some(chain_acct) = &chain_read.account else {
        return Ok(unavailable(
            format!("{} does not exist", spec.chain_account),
            evidence,
        ));
    };
    let feed = match spec
        .chain_idl
        .read_pubkey(&chain_acct.data, spec.feed_field)
    {
        Ok(Some(k)) => pda::encode(k),
        Ok(None) => {
            return Ok(unavailable(
                format!("{} has no Scope price account", spec.chain_account),
                evidence,
            ));
        }
        Err(e) => {
            return Ok(unavailable(
                format!("{}: {e}", spec.chain_account),
                evidence,
            ));
        }
    };
    if &feed != oracle {
        let reason = format!(
            "{} now reads Scope price account {feed}, not the registry's {oracle}",
            spec.chain_account
        );
        let mut out = unavailable(reason.clone(), evidence);
        out.review.push(format!("{reason}; registry review needed"));
        return Ok(out);
    }
    let chain = match spec.chain_idl.read_u16s(&chain_acct.data, spec.chain_field) {
        Ok(c) => c,
        Err(e) => {
            return Ok(unavailable(
                format!("{}: {e}", spec.chain_account),
                evidence,
            ));
        }
    };

    evidence.push(Evidence::new(
        Chain::Solana,
        oracle.clone(),
        prices_read.slot,
        "oracle_mappings",
    ));
    let Some(prices) = &prices_read.account else {
        return Ok(unavailable(format!("{oracle} does not exist"), evidence));
    };
    let mappings = match spec.scope_idl.read_pubkey(&prices.data, "oracle_mappings") {
        Ok(Some(k)) => pda::encode(k),
        Ok(None) => return Ok(unavailable(format!("{oracle} has no mappings"), evidence)),
        Err(e) => return Ok(unavailable(format!("{oracle}: {e}"), evidence)),
    };
    let mappings_read = rpc
        .get_multiple_accounts(std::slice::from_ref(&mappings), None)
        .await?
        .remove(0);
    evidence.push(Evidence::new(
        Chain::Solana,
        mappings.clone(),
        mappings_read.slot,
        "price_info_accounts, price_types, generic",
    ));
    let Some(m) = &mappings_read.account else {
        return Ok(unavailable(format!("{mappings} does not exist"), evidence));
    };

    let mut feeds = BTreeSet::new();
    for entry in chain.into_iter().filter(|e| *e != NO_ENTRY) {
        if let Err(reason) = walk(spec.scope_idl, &m.data, entry, 0, &mut feeds) {
            return Ok(unavailable(format!("{mappings}: {reason}"), evidence));
        }
    }
    Ok(ScopeResult {
        fact: Fact::new(
            UnderlyingFeeds::Listed {
                feeds: feeds.into_iter().collect(),
            },
            evidence,
        )
        .expect("evidence is never empty"),
        review: Vec::new(),
    })
}

fn walk(
    idl: &Idl,
    data: &[u8],
    entry: u16,
    depth: usize,
    out: &mut BTreeSet<UnderlyingFeed>,
) -> Result<(), String> {
    if entry >= MAX_ENTRIES {
        return Err(format!("entry {entry} is outside Scope's table"));
    }
    let price_type = idl
        .read_u8(data, &format!("price_types[{entry}]"))
        .map_err(|e| e.to_string())?;
    let name = idl
        .enum_variant("OracleType", usize::from(price_type))
        .ok_or_else(|| format!("entry {entry} has unknown price type {price_type}"))?;
    if REFERENCE_TYPES.contains(&name) {
        if depth >= MAX_DEPTH {
            return Err(format!(
                "entry {entry} nests more than {MAX_DEPTH} levels of {name}"
            ));
        }
        let generic = idl
            .read_bytes(data, &format!("generic[{entry}]"))
            .map_err(|e| e.to_string())?;
        for pair in generic[..8].chunks_exact(2) {
            let source = u16::from_le_bytes([pair[0], pair[1]]);
            if source < MAX_ENTRIES {
                walk(idl, data, source, depth + 1, out)?;
            }
        }
        return Ok(());
    }
    let account = idl
        .read_pubkey(data, &format!("price_info_accounts[{entry}]"))
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("entry {entry} has no source account"))?;
    out.insert(UnderlyingFeed {
        source_type: name.to_string(),
        address: pda::encode(account),
    });
    Ok(())
}
