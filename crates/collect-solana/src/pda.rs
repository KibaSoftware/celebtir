//! Address helpers: program-derived addresses and the ed25519 curve check
//! used to tell a wallet (on curve) from a PDA (off curve) (COL-8).

use celebtir_core::Address;
use curve25519_dalek::edwards::CompressedEdwardsY;
use sha2::{Digest, Sha256};

use crate::squads::{SQUADS_V3_PROGRAM, SQUADS_V4_PROGRAM};

/// 32 raw bytes of a base58 address, or None if it is not one.
pub fn decode(address: &Address) -> Option<[u8; 32]> {
    bs58::decode(address.as_str())
        .into_vec()
        .ok()?
        .try_into()
        .ok()
}

pub fn encode(bytes: [u8; 32]) -> Address {
    Address::new(bs58::encode(bytes).into_string())
}

/// True for keys that have a private key (wallets); false for PDAs.
pub fn is_on_curve(bytes: &[u8; 32]) -> bool {
    CompressedEdwardsY(*bytes).decompress().is_some()
}

/// `Pubkey::find_program_address`: the first off-curve hash, trying bumps
/// from 255 down.
pub fn find_program_address(seeds: &[&[u8]], program: &[u8; 32]) -> Option<([u8; 32], u8)> {
    (0..=255u8).rev().find_map(|bump| {
        let mut h = Sha256::new();
        for s in seeds {
            h.update(s);
        }
        h.update([bump]);
        h.update(program);
        h.update(b"ProgramDerivedAddress");
        let out: [u8; 32] = h.finalize().into();
        (!is_on_curve(&out)).then_some((out, bump))
    })
}

/// The Squads v3 authority a multisig signs with:
/// seeds `["squad", multisig, index (u32 LE), "authority"]`.
pub fn squads_v3_authority(multisig: &Address, authority_index: u32) -> Option<Address> {
    let ms = decode(multisig)?;
    let program = decode(&Address::new(SQUADS_V3_PROGRAM))?;
    let index = authority_index.to_le_bytes();
    let (authority, _) = find_program_address(&[b"squad", &ms, &index, b"authority"], &program)?;
    Some(encode(authority))
}

/// The Squads v4 vault a multisig signs with:
/// seeds `["multisig", multisig, "vault", [index]]`.
pub fn squads_vault(multisig: &Address, vault_index: u8) -> Option<Address> {
    let ms = decode(multisig)?;
    let program = decode(&Address::new(SQUADS_V4_PROGRAM))?;
    let (vault, _) = find_program_address(&[b"multisig", &ms, b"vault", &[vault_index]], &program)?;
    Some(encode(vault))
}

/// Re-derive a registry declaration, enforcing Solana's seed limits.
pub fn declared_address(d: &celebtir_registry::AuthorityPda) -> Option<Address> {
    if d.seeds.len() > 15 || !matches!(d.source.scheme(), "https" | "http") {
        return None;
    }
    let seeds: Option<Vec<Vec<u8>>> = d
        .seeds
        .iter()
        .map(|s| match s {
            celebtir_registry::PdaSeed::Literal(s) => Some(s.as_bytes().to_vec()),
            celebtir_registry::PdaSeed::Address(a) => decode(a).map(|b| b.to_vec()),
        })
        .collect();
    let seeds = seeds?;
    if seeds.iter().any(|s| s.len() > 32) {
        return None;
    }
    let refs: Vec<&[u8]> = seeds.iter().map(Vec::as_slice).collect();
    let (key, _) = find_program_address(&refs, &decode(&d.program)?)?;
    Some(encode(key))
}
