//! Token mints (TOK-3): owner program, supply, decimals, mint and freeze
//! authorities. Token-2022 extension bytes are kept for the TLV decoder (TOK-4).

use celebtir_core::Address;
use celebtir_core::snapshot::TokenProgram;

use crate::program::DecodeError;
use crate::reads::{AccountRead, SolanaRpc};
use crate::rpc::{RpcError, Transport};

pub const SPL_TOKEN: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
pub const TOKEN_2022: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";

/// The base Mint layout shared by both programs.
pub const MINT_LEN: usize = 82;
/// Token-2022 pads extended mints to the token-account length, then stores
/// the account type (1 = Mint) before the TLV extensions.
const ACCOUNT_TYPE_OFFSET: usize = 165;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MintRead {
    pub mint: Address,
    pub slot: u64,
    pub program: TokenProgram,
    pub supply: u64,
    pub decimals: u8,
    pub mint_authority: Option<Address>,
    pub freeze_authority: Option<Address>,
    /// Token-2022 TLV extension data (empty for classic mints).
    pub extension_data: Vec<u8>,
}

fn coption_pubkey(data: &[u8], at: usize) -> Result<Option<Address>, DecodeError> {
    let tag = u32::from_le_bytes(data[at..at + 4].try_into().expect("4 bytes"));
    match tag {
        0 => Ok(None),
        1 => Ok(Some(Address::new(
            bs58::encode(&data[at + 4..at + 36]).into_string(),
        ))),
        t => Err(DecodeError(format!("invalid COption tag {t}"))),
    }
}

pub fn decode_mint(read: &AccountRead) -> Result<MintRead, DecodeError> {
    let Some(account) = &read.account else {
        return Err(DecodeError(format!("mint {} does not exist", read.address)));
    };
    let program = match account.owner.as_str() {
        SPL_TOKEN => TokenProgram::SplToken,
        TOKEN_2022 => TokenProgram::Token2022,
        other => {
            return Err(DecodeError(format!(
                "{} is owned by {other}, not a token program",
                read.address
            )));
        }
    };
    let d = &account.data;
    if d.len() < MINT_LEN {
        return Err(DecodeError(format!(
            "mint data is {} bytes, expected at least {MINT_LEN}",
            d.len()
        )));
    }
    if d[45] != 1 {
        return Err(DecodeError("mint is not initialized".into()));
    }
    let extension_data = if program == TokenProgram::Token2022 && d.len() > ACCOUNT_TYPE_OFFSET {
        if d[ACCOUNT_TYPE_OFFSET] != 1 {
            return Err(DecodeError(format!(
                "Token-2022 account type {} is not a Mint",
                d[ACCOUNT_TYPE_OFFSET]
            )));
        }
        d[ACCOUNT_TYPE_OFFSET + 1..].to_vec()
    } else {
        Vec::new()
    };
    Ok(MintRead {
        mint: read.address.clone(),
        slot: read.slot,
        program,
        mint_authority: coption_pubkey(d, 0)?,
        supply: u64::from_le_bytes(d[36..44].try_into().expect("8 bytes")),
        decimals: d[44],
        freeze_authority: coption_pubkey(d, 46)?,
        extension_data,
    })
}

/// Every mint in one batched read (TOK-3); one result per mint, in order.
pub async fn read_mints<T: Transport>(
    rpc: &SolanaRpc<T>,
    mints: &[Address],
) -> Result<Vec<Result<MintRead, DecodeError>>, RpcError> {
    Ok(rpc
        .get_multiple_accounts(mints, None)
        .await?
        .iter()
        .map(decode_mint)
        .collect())
}
