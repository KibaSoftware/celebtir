//! Oracle classification (COL-12): read each oracle account's owner program
//! and look it up in `registry/oracles.yaml`. Unknown owners are "other".

use std::collections::BTreeMap;

use celebtir_core::snapshot::{OracleFact, OracleRole, OracleSource};
use celebtir_core::{Address, Chain, Evidence, Fact};
use serde::Deserialize;

use crate::reads::SolanaRpc;
use crate::rpc::{RpcError, Transport};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OracleClass {
    /// An oracle network, e.g. pyth, switchboard, chainlink.
    Network,
    /// A feed or aggregator run by a protocol (Kamino Scope, …).
    ProtocolRun,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OracleProgram {
    pub program: Address,
    pub class: OracleClass,
    /// Network name, matched against the methodology's established networks.
    pub name: String,
    pub source: String,
    /// Vendored IDL for reading the program's accounts (Scope: the mappings
    /// behind a price, edge case 11), relative to `registry/`.
    #[serde(default)]
    pub idl: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct OraclesFile {
    programs: Vec<OracleProgram>,
    /// HyperEVM entries (HREG-3), read by the Hyperliquid collector.
    #[serde(default, rename = "evm")]
    _evm: Option<serde::de::IgnoredAny>,
}

/// Owner program → classification, from `registry/oracles.yaml`.
#[derive(Debug, Clone, Default)]
pub struct OracleClassifier(BTreeMap<Address, OracleProgram>);

impl OracleClassifier {
    pub fn new(programs: Vec<OracleProgram>) -> Self {
        Self(
            programs
                .into_iter()
                .map(|p| (p.program.clone(), p))
                .collect(),
        )
    }

    pub fn from_yaml(text: &str) -> Result<Self, String> {
        let f: OraclesFile = serde_norway::from_str(text).map_err(|e| e.to_string())?;
        Ok(Self::new(f.programs))
    }

    pub fn program(&self, owner: &Address) -> Option<&OracleProgram> {
        self.0.get(owner)
    }

    pub fn programs(&self) -> impl Iterator<Item = &OracleProgram> {
        self.0.values()
    }

    pub fn classify(&self, owner: &Address) -> OracleSource {
        match self.0.get(owner) {
            Some(p) if p.class == OracleClass::Network => OracleSource::Network {
                name: p.name.clone(),
            },
            Some(_) => OracleSource::ProtocolRun,
            None => OracleSource::Other {
                owner_program: owner.clone(),
            },
        }
    }
}

/// One registry `oracles` entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OracleSpec {
    pub account: Address,
    pub market: String,
    pub role: OracleRole,
    /// The registry says this feed bounds the market's primary (v0.3, D3).
    pub bounds_primary: bool,
    pub guarded_by: Vec<String>,
    pub provider: Option<String>,
    /// The feed is a report ID verified by this program (Data Streams).
    pub verified_by: Option<Address>,
}

/// One batched owner-only read for all of a protocol's oracle accounts.
pub async fn read_oracles<T: Transport>(
    rpc: &SolanaRpc<T>,
    specs: &[OracleSpec],
    classifier: &OracleClassifier,
) -> Result<BTreeMap<Address, OracleFact>, RpcError> {
    // A verified report is classified by its verifier program, which must be
    // executable; anything else by the oracle account's owner (COL-12).
    let accounts: Vec<Address> = specs
        .iter()
        .map(|s| s.verified_by.clone().unwrap_or_else(|| s.account.clone()))
        .collect();
    let reads = rpc.get_multiple_accounts(&accounts, Some((0, 0))).await?;
    Ok(specs
        .iter()
        .zip(reads)
        .map(|(spec, read)| {
            let source = match (&spec.verified_by, &read.account) {
                (Some(v), Some(a)) if a.executable => classifier.classify(v),
                (Some(v), _) => OracleSource::Unavailable {
                    reason: format!("verifier program {v} is missing or not executable"),
                },
                (None, Some(a)) => classifier.classify(&a.owner),
                (None, None) => OracleSource::Unavailable {
                    reason: format!("oracle account {} does not exist", spec.account),
                },
            };
            let (address, field) = match &spec.verified_by {
                Some(v) => (
                    v.clone(),
                    format!("executable; verifies feed {}", spec.account),
                ),
                None => (spec.account.clone(), "owner".to_string()),
            };
            (
                spec.account.clone(),
                OracleFact {
                    market: spec.market.clone(),
                    role: spec.role,
                    bounds_primary: spec.bounds_primary,
                    guarded_by: spec.guarded_by.clone(),
                    provider: spec.provider.clone(),
                    source: Fact::single(
                        source,
                        Evidence::new(Chain::Solana, address, read.slot, field),
                    ),
                    underlying: None,
                    underlying_markets: BTreeMap::new(),
                },
            )
        })
        .collect())
}
