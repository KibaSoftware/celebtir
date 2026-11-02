//! Squads multisig accounts (COL-9).
//!
//! v4 `Multisig`: discriminator, create key, config authority, threshold,
//! time lock, transaction indices, optional rent collector, bump, members.
//! v3 (squads-mpl) `Ms`: discriminator, threshold, authority index,
//! transaction and change indices, bump, create key, external-execute flag,
//! keys. v3 has no time lock and no config authority: every member votes, and
//! changing the multisig itself takes a vote.

use celebtir_core::Address;

use crate::program::DecodeError;
use crate::reads::AccountRead;

pub const SQUADS_V4_PROGRAM: &str = "SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf";
pub const SQUADS_V3_PROGRAM: &str = "SMPLecH534NA9acpos4G6x7uf3LWbCAwZQE9e8ZekMu";

/// First 8 bytes of sha256("account:Multisig").
pub const MULTISIG_DISCRIMINATOR: [u8; 8] = [0xe0, 0x74, 0x79, 0xba, 0x44, 0xa1, 0x4f, 0xec];

/// First 8 bytes of sha256("account:Ms") (Squads v3).
pub const V3_MULTISIG_DISCRIMINATOR: [u8; 8] = [0x46, 0x76, 0x09, 0x6c, 0xfe, 0xd7, 0x1f, 0x78];

/// Member permission bits.
pub mod permission {
    pub const INITIATE: u8 = 1 << 0;
    pub const VOTE: u8 = 1 << 1;
    pub const EXECUTE: u8 = 1 << 2;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    pub key: Address,
    pub permissions: u8,
}

impl Member {
    pub fn can_vote(&self) -> bool {
        self.permissions & permission::VOTE != 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SquadsMultisig {
    pub address: Address,
    pub create_key: Address,
    /// None when unset (the default all-zero key): changes need a vote.
    pub config_authority: Option<Address>,
    pub threshold: u16,
    pub time_lock_secs: u32,
    pub members: Vec<Member>,
}

impl SquadsMultisig {
    /// Members with the Vote permission; only they count toward the threshold.
    pub fn voters(&self) -> impl Iterator<Item = &Address> {
        self.members.iter().filter(|m| m.can_vote()).map(|m| &m.key)
    }

    pub fn voter_count(&self) -> u16 {
        u16::try_from(self.voters().count()).unwrap_or(u16::MAX)
    }
}

struct Cursor<'a> {
    data: &'a [u8],
    at: usize,
}

impl<'a> Cursor<'a> {
    fn take(&mut self, n: usize, what: &str) -> Result<&'a [u8], DecodeError> {
        let end = self
            .at
            .checked_add(n)
            .filter(|e| *e <= self.data.len())
            .ok_or_else(|| {
                DecodeError(format!("multisig account truncated while reading {what}"))
            })?;
        let s = &self.data[self.at..end];
        self.at = end;
        Ok(s)
    }

    fn pubkey(&mut self, what: &str) -> Result<[u8; 32], DecodeError> {
        Ok(self.take(32, what)?.try_into().expect("32 bytes"))
    }

    fn u8(&mut self, what: &str) -> Result<u8, DecodeError> {
        Ok(self.take(1, what)?[0])
    }
}

fn address(bytes: [u8; 32]) -> Address {
    Address::new(bs58::encode(bytes).into_string())
}

pub fn decode_multisig(read: &AccountRead) -> Result<SquadsMultisig, DecodeError> {
    let Some(account) = &read.account else {
        return Err(DecodeError(format!(
            "multisig {} does not exist",
            read.address
        )));
    };
    if account.owner.as_str() != SQUADS_V4_PROGRAM {
        return Err(DecodeError(format!(
            "{} has owner {}, not the Squads v4 program",
            read.address, account.owner
        )));
    }
    let mut c = Cursor {
        data: &account.data,
        at: 0,
    };
    if c.take(8, "discriminator")? != MULTISIG_DISCRIMINATOR {
        return Err(DecodeError(format!(
            "{} is a Squads account but its discriminator is not Multisig",
            read.address
        )));
    }
    let create_key = address(c.pubkey("create_key")?);
    let config = c.pubkey("config_authority")?;
    let threshold = u16::from_le_bytes(c.take(2, "threshold")?.try_into().expect("2 bytes"));
    let time_lock_secs = u32::from_le_bytes(c.take(4, "time_lock")?.try_into().expect("4 bytes"));
    c.take(16, "transaction indices")?;
    match c.u8("rent_collector tag")? {
        0 => {}
        1 => {
            c.pubkey("rent_collector")?;
        }
        t => {
            return Err(DecodeError(format!(
                "invalid rent_collector Option tag {t}"
            )));
        }
    }
    c.u8("bump")?;
    let count = u32::from_le_bytes(c.take(4, "member count")?.try_into().expect("4 bytes"));
    let mut members = Vec::new();
    for i in 0..count {
        let key = address(c.pubkey(&format!("member {i}"))?);
        let permissions = c.u8(&format!("member {i} permissions"))?;
        members.push(Member { key, permissions });
    }
    Ok(SquadsMultisig {
        address: read.address.clone(),
        create_key,
        config_authority: (config != [0u8; 32]).then(|| address(config)),
        threshold,
        time_lock_secs,
        members,
    })
}

/// Decodes a Squads v3 `Ms` account. Every key votes; there is no time lock
/// and no config authority.
pub fn decode_v3_multisig(read: &AccountRead) -> Result<SquadsMultisig, DecodeError> {
    let Some(account) = &read.account else {
        return Err(DecodeError(format!(
            "multisig {} does not exist",
            read.address
        )));
    };
    if account.owner.as_str() != SQUADS_V3_PROGRAM {
        return Err(DecodeError(format!(
            "{} has owner {}, not the Squads v3 program",
            read.address, account.owner
        )));
    }
    let mut c = Cursor {
        data: &account.data,
        at: 0,
    };
    if c.take(8, "discriminator")? != V3_MULTISIG_DISCRIMINATOR {
        return Err(DecodeError(format!(
            "{} is a Squads v3 account but its discriminator is not Ms",
            read.address
        )));
    }
    let threshold = u16::from_le_bytes(c.take(2, "threshold")?.try_into().expect("2 bytes"));
    c.take(2, "authority_index")?;
    c.take(8, "transaction and change indices")?;
    c.u8("bump")?;
    let create_key = address(c.pubkey("create_key")?);
    c.u8("allow_external_execute")?;
    let count = u32::from_le_bytes(c.take(4, "key count")?.try_into().expect("4 bytes"));
    let mut members = Vec::new();
    for i in 0..count {
        let key = address(c.pubkey(&format!("key {i}"))?);
        members.push(Member {
            key,
            permissions: permission::INITIATE | permission::VOTE | permission::EXECUTE,
        });
    }
    Ok(SquadsMultisig {
        address: read.address.clone(),
        create_key,
        config_authority: None,
        threshold,
        time_lock_secs: 0,
        members,
    })
}
