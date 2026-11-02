//! Captured mainnet accounts for decoder tests (QA-1, ADR 009). Each file in
//! `fixtures/accounts/` holds one account's bytes, the slot it was read at and
//! where it came from. Capture with
//! `cargo run -p collect-solana --example capture -- <name> <address> [--slice 0:45] [--note "…"]`.

use std::path::{Path, PathBuf};

use base64::Engine;
use celebtir_core::Address;
use serde::{Deserialize, Serialize};

use crate::reads::{AccountRead, RawAccount};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccountFixture {
    pub name: String,
    pub address: Address,
    pub slot: u64,
    /// None when the account did not exist at capture time.
    pub owner: Option<Address>,
    pub lamports: u64,
    pub executable: bool,
    pub data_base64: String,
    /// `(offset, length)` when only part of the data was read.
    pub data_slice: Option<(usize, usize)>,
    /// RPC host and method; never a full URL (SNAP-5).
    pub source: String,
    pub note: String,
}

impl AccountFixture {
    pub fn from_read(
        name: &str,
        read: &AccountRead,
        slice: Option<(usize, usize)>,
        source: &str,
        note: &str,
    ) -> Self {
        let (owner, lamports, executable, data) = match &read.account {
            Some(a) => (
                Some(a.owner.clone()),
                a.lamports,
                a.executable,
                a.data.clone(),
            ),
            None => (None, 0, false, Vec::new()),
        };
        Self {
            name: name.to_string(),
            address: read.address.clone(),
            slot: read.slot,
            owner,
            lamports,
            executable,
            data_base64: base64::engine::general_purpose::STANDARD.encode(data),
            data_slice: slice,
            source: source.to_string(),
            note: note.to_string(),
        }
    }

    pub fn data(&self) -> Vec<u8> {
        base64::engine::general_purpose::STANDARD
            .decode(&self.data_base64)
            .expect("fixture data is base64")
    }

    /// The fixture as the collector would have read it.
    pub fn read(&self) -> AccountRead {
        AccountRead {
            address: self.address.clone(),
            slot: self.slot,
            account: self.owner.as_ref().map(|owner| RawAccount {
                owner: owner.clone(),
                lamports: self.lamports,
                executable: self.executable,
                data: self.data(),
            }),
        }
    }
}

/// `<repo>/fixtures/accounts`.
pub fn accounts_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/accounts")
}

/// Loads `fixtures/accounts/<name>.json`.
pub fn load(name: &str) -> AccountFixture {
    let path = accounts_dir().join(format!("{name}.json"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read fixture {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("bad fixture {}: {e}", path.display()))
}
