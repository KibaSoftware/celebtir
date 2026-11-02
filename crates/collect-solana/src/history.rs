//! Upgrade history (VAL-4): every transaction on a program's ProgramData
//! account is listed and its upgradeable-loader instructions classified.
//! Upgrades run through a multisig arrive as inner instructions, so both
//! levels are scanned. Results are cached by signature between runs.

use celebtir_core::Address;
use celebtir_core::snapshot::{ProgramHistory, UpgradeEvent, UpgradeKind};
use serde_json::Value;

use crate::deps::TxCache;
use crate::program::{DecodeError, UPGRADEABLE_LOADER};
use crate::reads::SolanaRpc;
use crate::rpc::{RpcError, Transport};

/// Signatures per page, and pages read at most (10,000 transactions).
const PAGE: usize = 1_000;
const MAX_PAGES: usize = 10;

/// `(kind, new authority)` for each loader instruction in `tx` that touches
/// `programdata`, in instruction order.
pub fn programdata_events(
    tx: &Value,
    programdata: &Address,
) -> Result<Vec<(UpgradeKind, Option<Address>)>, DecodeError> {
    let bad = |m: &str| DecodeError(m.to_string());
    let msg = &tx["transaction"]["message"];
    let mut keys: Vec<&str> = msg["accountKeys"]
        .as_array()
        .ok_or_else(|| bad("transaction has no account keys"))?
        .iter()
        .filter_map(Value::as_str)
        .collect();
    for list in ["writable", "readonly"] {
        if let Some(extra) = tx["meta"]["loadedAddresses"][list].as_array() {
            keys.extend(extra.iter().filter_map(Value::as_str));
        }
    }
    let top = msg["instructions"].as_array().into_iter().flatten();
    let inner = tx["meta"]["innerInstructions"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|g| g["instructions"].as_array())
        .flatten();

    let mut out = Vec::new();
    for ix in top.chain(inner) {
        let key = |i: &Value| i.as_u64().and_then(|i| keys.get(i as usize).copied());
        if key(&ix["programIdIndex"]) != Some(UPGRADEABLE_LOADER) {
            continue;
        }
        let accounts: Vec<&str> = ix["accounts"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(key)
            .collect();
        if !accounts.contains(&programdata.as_str()) {
            continue;
        }
        let data = bs58::decode(ix["data"].as_str().unwrap_or_default())
            .into_vec()
            .map_err(|_| bad("loader instruction data is not base58"))?;
        let tag = data
            .get(..4)
            .map(|b| u32::from_le_bytes(b.try_into().expect("4 bytes")))
            .ok_or_else(|| bad("loader instruction data too short"))?;
        let event = match tag {
            2 => (UpgradeKind::Deploy, None),
            3 => (UpgradeKind::Upgrade, None),
            // SetAuthority: [account, current, new?]; Checked: [account, current, new].
            4 | 7 => match accounts.get(2) {
                Some(new) => (UpgradeKind::SetAuthority, Some(Address::new(*new))),
                None => (UpgradeKind::RemoveAuthority, None),
            },
            5 => (UpgradeKind::Close, None),
            6 | 9 => (UpgradeKind::Extend, None),
            // Buffer writes and migrations are not changes to this program.
            _ => continue,
        };
        out.push(event);
    }
    Ok(out)
}

/// One ProgramData account's history: cached transactions plus at most
/// `fetch_limit` newly read ones (newest first), so busy accounts fill in
/// over several runs within the call budget.
pub async fn upgrade_history<T: Transport>(
    rpc: &SolanaRpc<T>,
    programdata: &Address,
    cache: &mut TxCache,
    fetch_limit: usize,
) -> Result<ProgramHistory, RpcError> {
    // Pages sized to the work allowed this run, so a busy account costs a few
    // small pages a day while its history fills in.
    let page_size = fetch_limit.saturating_mul(2).clamp(100, PAGE);
    let prefix = format!("{programdata}:");
    let mut before: Option<String> = None;
    let (mut listed, mut fetched) = (0u64, 0usize);
    let (mut exhausted, mut left_unread) = (false, false);
    for _ in 0..MAX_PAGES {
        // Retain cached evidence, but never claim a complete fresh scan.
        if fetch_limit == 0 {
            break;
        }
        let page = rpc
            .get_signatures_page(programdata, page_size, before.as_deref())
            .await?;
        exhausted = page.len() < page_size;
        before = page.last().map(|s| s.signature.clone());
        listed += page.len() as u64;
        for sig in &page {
            let key = format!("{prefix}{}", sig.signature);
            if cache.upgrades.contains_key(&key) {
                continue;
            }
            // A failed transaction cannot have changed the program.
            if sig.failed {
                cache.upgrades.insert(key, Vec::new());
                continue;
            }
            if fetched >= fetch_limit {
                left_unread = true;
                continue;
            }
            fetched += 1;
            let tx = match rpc.get_transaction(&sig.signature).await {
                Ok(Some(tx)) => tx,
                Ok(None) | Err(RpcError::Rpc { .. } | RpcError::Malformed { .. }) => {
                    left_unread = true;
                    continue;
                }
                Err(e) => return Err(e),
            };
            let Ok(found) = programdata_events(&tx, programdata) else {
                left_unread = true;
                continue;
            };
            let date = tx["blockTime"].as_i64().map(celebtir_core::date::ymd);
            let mapped = found
                .into_iter()
                .map(|(kind, new_authority)| UpgradeEvent {
                    signature: sig.signature.clone(),
                    slot: sig.slot,
                    date: date.clone(),
                    kind,
                    new_authority,
                })
                .collect();
            cache.upgrades.insert(key, mapped);
        }
        if exhausted || (left_unread && fetched >= fetch_limit) {
            break;
        }
    }

    // Events from every transaction read so far, this run or earlier.
    let cached: Vec<&Vec<UpgradeEvent>> = cache
        .upgrades
        .range(prefix.clone()..)
        .take_while(|(k, _)| k.starts_with(&prefix))
        .map(|(_, v)| v)
        .collect();
    let scanned = cached.len() as u64;
    let mut events: Vec<UpgradeEvent> = cached.into_iter().flatten().cloned().collect();
    events.sort_by(|a, b| (a.slot, &a.signature).cmp(&(b.slot, &b.signature)));
    Ok(ProgramHistory {
        events,
        scanned,
        total: listed.max(scanned),
        complete: exhausted && !left_unread,
    })
}
