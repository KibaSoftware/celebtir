//! Hyperliquid collector (wave 2, ADR 024): HyperEVM contract reads over
//! JSON-RPC and, from slice 9.10, HIP-3 perp DEXs from the info API.

pub mod collect;
pub mod evm;
pub mod fixtures;
pub mod hip3;
pub mod pipeline;
pub mod resolve;
pub mod verify;
