//! Token-2022 extensions (TOK-4) and token facts with every authority
//! resolved like a program authority (TOK-5).

use celebtir_core::snapshot::{TokenExtensions, TokenFacts, TokenProvenance, TransferHook};
use celebtir_core::{Address, Chain, Controller, Evidence, Fact};

use crate::control::upgrade_controls;
use crate::mint::MintRead;
use crate::program::DecodeError;
use crate::reads::SolanaRpc;
use crate::resolve::Resolver;
use crate::rpc::{RpcError, Transport};

/// Extensions as stored: `Some(None)` means present with an empty authority.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedExtensions {
    pub permanent_delegate: Option<Option<Address>>,
    /// `(authority, program)`.
    pub transfer_hook: Option<(Option<Address>, Option<Address>)>,
    pub transfer_fee_authority: Option<Option<Address>>,
    pub pause_authority: Option<Option<Address>>,
    pub default_frozen: bool,
    pub mint_close_authority: Option<Option<Address>>,
    pub metadata_update_authority: Option<Option<Address>>,
    pub other: Vec<String>,
    pub unknown: Vec<u16>,
}

/// Mint extension types with no control power Celebtir rates.
fn other_name(t: u16) -> Option<&'static str> {
    Some(match t {
        2 => "transfer_fee_amount",
        4 => "confidential_transfer_mint",
        5 => "confidential_transfer_account",
        7 => "immutable_owner",
        8 => "memo_transfer",
        9 => "non_transferable",
        10 => "interest_bearing_config",
        11 => "cpi_guard",
        13 => "non_transferable_account",
        15 => "transfer_hook_account",
        16 => "confidential_transfer_fee_config",
        17 => "confidential_transfer_fee_amount",
        18 => "metadata_pointer",
        20 => "group_pointer",
        21 => "token_group",
        22 => "group_member_pointer",
        23 => "token_group_member",
        24 => "confidential_mint_burn",
        25 => "scaled_ui_amount",
        27 => "pausable_account",
        _ => return None,
    })
}

/// An `OptionalNonZeroPubkey` at `at`: all zeros means none.
fn key_at(v: &[u8], at: usize, ty: u16) -> Result<Option<Address>, DecodeError> {
    let bytes: [u8; 32] = v
        .get(at..at + 32)
        .ok_or_else(|| DecodeError(format!("extension {ty} is too short")))?
        .try_into()
        .expect("32 bytes");
    Ok((bytes != [0u8; 32]).then(|| crate::pda::encode(bytes)))
}

pub fn parse_extensions(tlv: &[u8]) -> Result<ParsedExtensions, DecodeError> {
    let mut e = ParsedExtensions::default();
    let mut at = 0;
    while at + 4 <= tlv.len() {
        let ty = u16::from_le_bytes([tlv[at], tlv[at + 1]]);
        let len = u16::from_le_bytes([tlv[at + 2], tlv[at + 3]]) as usize;
        let v = tlv
            .get(at + 4..at + 4 + len)
            .ok_or_else(|| DecodeError(format!("extension {ty} runs past the account data")))?;
        match ty {
            // Uninitialized: the rest of the buffer is padding.
            0 => break,
            1 => e.transfer_fee_authority = Some(key_at(v, 0, ty)?),
            3 => e.mint_close_authority = Some(key_at(v, 0, ty)?),
            6 => e.default_frozen = v.first() == Some(&2),
            12 => e.permanent_delegate = Some(key_at(v, 0, ty)?),
            14 => e.transfer_hook = Some((key_at(v, 0, ty)?, key_at(v, 32, ty)?)),
            19 => e.metadata_update_authority = Some(key_at(v, 0, ty)?),
            26 => e.pause_authority = Some(key_at(v, 0, ty)?),
            t => match other_name(t) {
                Some(name) => e.other.push(name.to_string()),
                None => e.unknown.push(t),
            },
        }
        at += 4 + len;
    }
    Ok(e)
}

/// Builds a mint's facts: authorities and extension authorities resolved,
/// a transfer hook program's upgrade control read and resolved.
pub async fn token_facts<T: Transport>(
    rpc: &SolanaRpc<T>,
    resolver: &Resolver<'_, T>,
    mint: &MintRead,
    symbol: &str,
    provenance: TokenProvenance,
) -> Result<(TokenFacts, Vec<String>), RpcError> {
    let ev = |field: &str| Evidence::new(Chain::Solana, mint.mint.clone(), mint.slot, field);
    // Unresolved authorities raise registry review notes, as for protocols.
    let notes: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());
    let resolve = |key: Option<Address>, field: &'static str| {
        let origin = ev(field);
        let notes = &notes;
        async move {
            match key {
                None => Ok::<_, RpcError>(Fact::single(Controller::Immutable, origin)),
                Some(k) => {
                    let r = resolver.resolve(&k, origin).await?;
                    notes
                        .lock()
                        .expect("notes lock")
                        .extend(r.review_needed.into_iter().map(|n| format!("{field}: {n}")));
                    Ok(r.fact)
                }
            }
        }
    };

    let mint_authority = resolve(mint.mint_authority.clone(), "mint_authority").await?;
    let freeze_authority = resolve(mint.freeze_authority.clone(), "freeze_authority").await?;

    let mut evidence = vec![ev("extensions")];
    let extensions = match parse_extensions(&mint.extension_data) {
        Err(e) => {
            // Never guess: undecodable TLV shows up as an unknown extension.
            let mut x = TokenExtensions::default();
            x.other.push(format!("undecodable extension data: {e}"));
            x.unknown.push(u16::MAX);
            x
        }
        Ok(p) => {
            let mut out = TokenExtensions {
                default_frozen: p.default_frozen,
                other: p.other,
                unknown: p.unknown,
                ..TokenExtensions::default()
            };
            // Each present authority is resolved; resolution evidence beyond
            // the mint read itself joins the extensions fact.
            let mut controller = async |key: Option<Option<Address>>, field: &'static str| {
                let Some(k) = key else {
                    return Ok::<_, RpcError>(None);
                };
                let fact = resolve(k, field).await?;
                evidence.extend(fact.evidence().iter().skip(1).cloned());
                Ok(Some(fact.value().clone()))
            };
            out.permanent_delegate = controller(p.permanent_delegate, "permanent_delegate").await?;
            out.transfer_fee_authority =
                controller(p.transfer_fee_authority, "transfer_fee_authority").await?;
            out.pause_authority = controller(p.pause_authority, "pause_authority").await?;
            out.mint_close_authority =
                controller(p.mint_close_authority, "mint_close_authority").await?;
            out.metadata_update_authority =
                controller(p.metadata_update_authority, "metadata_update_authority").await?;
            if let Some((authority, program)) = p.transfer_hook {
                let authority = resolve(authority, "transfer_hook_authority").await?;
                let program_controller = match &program {
                    None => None,
                    Some(prog) => {
                        let pc = upgrade_controls(rpc, resolver, std::slice::from_ref(prog))
                            .await?
                            .remove(0)
                            .upgrade_control;
                        evidence.extend(pc.evidence().iter().cloned());
                        Some(pc.value().clone())
                    }
                };
                out.transfer_hook = Some(TransferHook {
                    program,
                    authority: authority.value().clone(),
                    program_controller,
                });
            }
            out
        }
    };

    let facts = TokenFacts {
        symbol: symbol.to_string(),
        program: mint.program,
        supply: mint.supply,
        decimals: mint.decimals,
        mint_authority,
        freeze_authority,
        extensions: Fact::new(extensions, evidence).expect("mint evidence is always present"),
        provenance: Fact::single(provenance, ev("mint")),
        identity: None,
    };
    Ok((facts, notes.into_inner().expect("notes lock")))
}
