//! Program control (COL-6, COL-7): which loader owns a program, and for
//! upgradeable programs the ProgramData header (last deploy slot, upgrade
//! authority). Only the first 36 bytes of program accounts and 45 bytes of
//! ProgramData are read, so bytecode is never downloaded.

use celebtir_core::Address;

use crate::reads::{AccountRead, SolanaRpc};
use crate::rpc::{RpcError, Transport};

pub const UPGRADEABLE_LOADER: &str = "BPFLoaderUpgradeab1e11111111111111111111111";
pub const LOADER_V1: &str = "BPFLoader1111111111111111111111111111111111";
pub const LOADER_V2: &str = "BPFLoader2111111111111111111111111111111111";
pub const LOADER_V4: &str = "LoaderV411111111111111111111111111111111111";
pub const NATIVE_LOADER: &str = "NativeLoader1111111111111111111111111111111";

/// Upgradeable program account: `u32` tag 2, then the ProgramData address.
const PROGRAM_ACCOUNT_LEN: usize = 36;
/// ProgramData header: `u32` tag 3, `u64` slot, `Option<Pubkey>` (1 + 32).
pub const PROGRAMDATA_HEADER_LEN: usize = 45;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgramKind {
    /// Owned by the upgradeable loader; control lives in ProgramData.
    Upgradeable { programdata: Address },
    /// Owned by loader v1 or v2, which cannot upgrade.
    Immutable { loader: Address },
    /// Loader v4, native or anything else: recorded, never guessed.
    UnsupportedLoader { owner: Address },
    /// The program account does not exist.
    Closed,
}

impl ProgramKind {
    /// The program account's owner, shown on the protocol page; None when
    /// the account is missing or did not decode.
    pub fn loader(&self) -> Option<Address> {
        match self {
            ProgramKind::Upgradeable { .. } => Some(Address::new(UPGRADEABLE_LOADER)),
            ProgramKind::Immutable { loader } => Some(loader.clone()),
            ProgramKind::UnsupportedLoader { owner } => Some(owner.clone()),
            ProgramKind::Closed => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramDataHeader {
    pub last_deploy_slot: u64,
    /// None when the authority was removed: the program is immutable.
    pub authority: Option<Address>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct DecodeError(pub String);

fn pubkey(bytes: &[u8]) -> Address {
    Address::new(bs58::encode(bytes).into_string())
}

fn tag(data: &[u8]) -> Option<u32> {
    data.get(..4)
        .map(|b| u32::from_le_bytes(b.try_into().expect("4 bytes")))
}

pub fn classify(read: &AccountRead) -> Result<ProgramKind, DecodeError> {
    let Some(account) = &read.account else {
        return Ok(ProgramKind::Closed);
    };
    match account.owner.as_str() {
        UPGRADEABLE_LOADER => {
            if account.data.len() < PROGRAM_ACCOUNT_LEN || tag(&account.data) != Some(2) {
                return Err(DecodeError(format!(
                    "{} is owned by the upgradeable loader but is not a Program account",
                    read.address
                )));
            }
            Ok(ProgramKind::Upgradeable {
                programdata: pubkey(&account.data[4..PROGRAM_ACCOUNT_LEN]),
            })
        }
        LOADER_V1 | LOADER_V2 => Ok(ProgramKind::Immutable {
            loader: account.owner.clone(),
        }),
        _ => Ok(ProgramKind::UnsupportedLoader {
            owner: account.owner.clone(),
        }),
    }
}

pub fn decode_programdata(data: &[u8]) -> Result<ProgramDataHeader, DecodeError> {
    if data.len() < 13 {
        return Err(DecodeError(format!(
            "ProgramData header too short: {} bytes",
            data.len()
        )));
    }
    if tag(data) != Some(3) {
        return Err(DecodeError("account is not ProgramData (tag != 3)".into()));
    }
    let last_deploy_slot = u64::from_le_bytes(data[4..12].try_into().expect("8 bytes"));
    let authority = match data[12] {
        0 => None,
        1 if data.len() >= PROGRAMDATA_HEADER_LEN => Some(pubkey(&data[13..45])),
        1 => return Err(DecodeError("ProgramData authority truncated".into())),
        other => {
            return Err(DecodeError(format!(
                "invalid Option tag {other} for the upgrade authority"
            )));
        }
    };
    Ok(ProgramDataHeader {
        last_deploy_slot,
        authority,
    })
}

/// Everything read about one program's control, with the slots of both reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramControlRead {
    pub program: Address,
    pub program_slot: u64,
    pub kind: ProgramKind,
    /// Set for upgradeable programs whose ProgramData decoded.
    pub header: Option<ProgramDataHeader>,
    pub programdata_slot: Option<u64>,
    /// Why `kind` or `header` could not be decoded; the fact becomes Unavailable.
    pub error: Option<String>,
}

/// Two batched reads for any number of programs: program accounts (36-byte
/// slice), then the ProgramData headers (45-byte slice).
pub async fn read_program_control<T: Transport>(
    rpc: &SolanaRpc<T>,
    programs: &[Address],
) -> Result<Vec<ProgramControlRead>, RpcError> {
    let program_reads = rpc
        .get_multiple_accounts(programs, Some((0, PROGRAM_ACCOUNT_LEN)))
        .await?;
    let mut out: Vec<ProgramControlRead> = program_reads
        .iter()
        .map(|r| {
            let (kind, error) = match classify(r) {
                Ok(k) => (k, None),
                Err(e) => (ProgramKind::Closed, Some(e.0)),
            };
            ProgramControlRead {
                program: r.address.clone(),
                program_slot: r.slot,
                kind,
                header: None,
                programdata_slot: None,
                error,
            }
        })
        .collect();

    let programdata: Vec<Address> = out
        .iter()
        .filter_map(|p| match &p.kind {
            ProgramKind::Upgradeable { programdata } if p.error.is_none() => {
                Some(programdata.clone())
            }
            _ => None,
        })
        .collect();
    if programdata.is_empty() {
        return Ok(out);
    }
    let headers = rpc
        .get_multiple_accounts(&programdata, Some((0, PROGRAMDATA_HEADER_LEN)))
        .await?;
    for p in &mut out {
        let ProgramKind::Upgradeable { programdata } = &p.kind else {
            continue;
        };
        let Some(read) = headers.iter().find(|h| &h.address == programdata) else {
            continue;
        };
        p.programdata_slot = Some(read.slot);
        match &read.account {
            Some(acct) if acct.owner.as_str() == UPGRADEABLE_LOADER => {
                match decode_programdata(&acct.data) {
                    Ok(h) => p.header = Some(h),
                    Err(e) => p.error = Some(e.0),
                }
            }
            Some(acct) => {
                p.error = Some(format!(
                    "ProgramData {programdata} is owned by {}, not the upgradeable loader",
                    acct.owner
                ));
            }
            None => p.error = Some(format!("ProgramData {programdata} does not exist")),
        }
    }
    Ok(out)
}
