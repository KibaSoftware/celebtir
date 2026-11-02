//! EVM address helpers for HyperEVM and HyperCore (ADR 024 D9).

use sha3::{Digest, Keccak256};

/// `0x` + 40 hex digits, any case → the EIP-55 checksummed form.
pub fn eip55(address: &str) -> Option<String> {
    let hex = address.strip_prefix("0x")?;
    if hex.len() != 40 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let lower = hex.to_ascii_lowercase();
    let hash = Keccak256::digest(lower.as_bytes());
    let mut out = String::with_capacity(42);
    out.push_str("0x");
    for (i, c) in lower.chars().enumerate() {
        let nibble = (hash[i / 2] >> if i % 2 == 0 { 4 } else { 0 }) & 0x0f;
        out.push(if c.is_ascii_alphabetic() && nibble >= 8 {
            c.to_ascii_uppercase()
        } else {
            c
        });
    }
    Some(out)
}

/// True when `address` is already in EIP-55 form.
pub fn is_checksummed(address: &str) -> bool {
    eip55(address).is_some_and(|c| c == address)
}
