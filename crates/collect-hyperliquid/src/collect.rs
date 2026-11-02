//! One HyperEVM protocol's facts from its registry entry (HCOL-3 to
//! HCOL-10). Every read is at the run's pinned block.

use std::collections::BTreeMap;

use celebtir_core::snapshot::{
    AdminFact, CodeTransparency, Dependencies, DependencyFact, OracleFact, OracleSource,
    ProgramFacts, ProtocolFacts, RegistrySource, SourceKind,
};
use celebtir_core::{Address, Chain, Evidence, Fact};
use celebtir_registry::{EvmOracle, ProtocolEntry};
use collect_solana::rpc::Transport;

use crate::resolve::{Resolved, Resolver};

/// `oracles.yaml`'s `evm:` section, keyed by lower-case address.
#[derive(Debug, Clone, Default)]
pub struct EvmOracles(BTreeMap<String, EvmOracle>);

impl EvmOracles {
    pub fn new(list: Vec<EvmOracle>) -> Self {
        Self(
            list.into_iter()
                .map(|o| (o.address.as_str().to_ascii_lowercase(), o))
                .collect(),
        )
    }

    pub fn from_yaml(text: &str) -> Result<Self, String> {
        #[derive(serde::Deserialize)]
        struct File {
            #[serde(default)]
            evm: Vec<EvmOracle>,
        }
        serde_norway::from_str::<File>(text)
            .map(|f| Self::new(f.evm))
            .map_err(|e| e.to_string())
    }

    /// HCOL-9: a known network, a protocol-run feed, or "other".
    pub fn classify(&self, address: &Address) -> OracleSource {
        match self.0.get(&address.as_str().to_ascii_lowercase()) {
            Some(o) if o.class == "protocol-run" => OracleSource::ProtocolRun,
            Some(o) => OracleSource::Network {
                name: o.name.clone(),
            },
            None => OracleSource::Other {
                owner_program: address.clone(),
            },
        }
    }
}

fn fact<T>(value: T, evidence: Vec<Evidence>) -> Fact<T> {
    Fact::new(value, evidence).expect("every resolved read carries evidence")
}

fn resolved(r: Resolved) -> Fact<celebtir_core::Controller> {
    fact(r.controller, r.evidence)
}

/// Collects one protocol. `transparency` holds the verified-source facts
/// per contract (HCOL-8); a contract missing from it is Unavailable.
pub async fn collect_protocol<T: Transport>(
    resolver: &Resolver<'_, T>,
    oracles: &EvmOracles,
    entry: &ProtocolEntry,
    transparency: &BTreeMap<Address, Fact<CodeTransparency>>,
) -> ProtocolFacts {
    let block = resolver.evm.block();
    let ev = |a: &Address, field: &str| Evidence::new(Chain::HyperEvm, a.clone(), block, field);

    let mut programs = BTreeMap::new();
    for p in &entry.programs {
        let proxy = p.proxy.unwrap_or(celebtir_registry::ProxyKind::Eip1967);
        let upgrade_control = resolved(resolver.upgrade_control(&p.id, proxy).await);
        let code_transparency = transparency.get(&p.id).cloned().unwrap_or_else(|| {
            fact(
                CodeTransparency::Unavailable {
                    reason: "verified source not checked".into(),
                },
                vec![ev(&p.id, "source")],
            )
        });
        programs.insert(
            p.id.clone(),
            ProgramFacts {
                role: p.role,
                loader: None,
                upgrade_control,
                code_transparency,
            },
        );
    }

    let mut admin = BTreeMap::new();
    for a in &entry.evm_admins {
        let key = format!("{}.{}", a.contract, a.role.as_deref().unwrap_or("owner()"));
        let r = resolver
            .admin(&a.contract, a.role.as_deref(), a.holder.as_ref())
            .await;
        admin.insert(
            key,
            AdminFact {
                account: a.contract.clone(),
                field: a.role.clone().unwrap_or_else(|| "owner()".into()),
                power: a.power,
                controller: resolved(r),
            },
        );
    }

    let mut oracle_facts = BTreeMap::new();
    for o in &entry.oracles {
        let source = match resolver.evm.code(o.account.as_str()).await {
            Ok(code) if code.is_empty() => OracleSource::Unavailable {
                reason: "no contract at this address".into(),
            },
            Ok(_) => oracles.classify(&o.account),
            Err(e) => OracleSource::Unavailable {
                reason: e.to_string(),
            },
        };
        oracle_facts.insert(
            o.account.clone(),
            OracleFact {
                market: o.market.clone(),
                role: o.role,
                bounds_primary: o.bounds_primary,
                guarded_by: o.guarded_by.clone(),
                provider: o.provider.clone(),
                source: fact(source, vec![ev(&o.account, "code + oracles.yaml")]),
                underlying: None,
                underlying_markets: Default::default(),
            },
        );
    }

    // HCOL-10: callees are declared in the registry, not traced.
    let mut callees = BTreeMap::new();
    for d in &entry.dependencies {
        let r = resolver.upgrade_control(&d.address, d.proxy).await;
        callees.insert(
            d.address.clone(),
            DependencyFact {
                upgrade_control: resolved(r),
            },
        );
    }
    let anchor = entry
        .programs
        .first()
        .map(|p| p.id.clone())
        .expect("validated entries have a program");
    let dependencies = fact(
        Dependencies::Sampled {
            callees,
            stale: false,
            declared: true,
        },
        vec![ev(&anchor, "registry dependencies")],
    );

    ProtocolFacts {
        chain: Chain::HyperEvm,
        category: Some(entry.category.as_str().to_string()),
        last_reviewed: Some(entry.review.last_reviewed.clone()),
        name: Some(entry.name.clone()),
        website: Some(entry.website.to_string()),
        reviewed_by: Some(entry.review.reviewed_by.clone()),
        sources: sources(entry),
        programs,
        admin,
        oracles: oracle_facts,
        admin_review: None,
        oracle_review: None,
        dependencies,
        tokens: BTreeMap::new(),
        min_slot: block,
        max_slot: block,
    }
}

fn sources(entry: &ProtocolEntry) -> Vec<RegistrySource> {
    let s = |kind, address: &Address, label: &str, url: &url::Url| RegistrySource {
        kind,
        address: address.clone(),
        label: label.to_string(),
        url: url.to_string(),
    };
    let mut out: Vec<RegistrySource> = entry
        .programs
        .iter()
        .map(|p| s(SourceKind::Program, &p.id, &p.name, &p.source))
        .chain(
            entry
                .safes
                .iter()
                .map(|m| s(SourceKind::Multisig, &m.address, "Safe", &m.source)),
        )
        .chain(
            entry
                .timelocks
                .iter()
                .map(|t| s(SourceKind::Multisig, &t.address, "Timelock", &t.source)),
        )
        .chain(entry.evm_admins.iter().map(|a| {
            s(
                SourceKind::Admin,
                &a.contract,
                a.role.as_deref().unwrap_or("owner()"),
                &a.source,
            )
        }))
        .chain(
            entry
                .oracles
                .iter()
                .map(|o| s(SourceKind::Oracle, &o.account, &o.market, &o.source)),
        )
        .chain(
            entry
                .dependencies
                .iter()
                .map(|d| s(SourceKind::Program, &d.address, &d.name, &d.source)),
        )
        .collect();
    out.sort();
    out.dedup();
    out
}
