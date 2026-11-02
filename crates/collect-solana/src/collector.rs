//! The whole Solana collection for a set of registry protocols (COL-15): each
//! protocol is collected on its own, so one failure never stops the others;
//! the run report lists every failure with its cause. A spent call budget
//! stops the run cleanly.

use std::collections::{BTreeMap, BTreeSet};

use celebtir_core::snapshot::UnderlyingFeeds;
use celebtir_core::snapshot::{
    CodeTransparency, Dependencies, DependencyFact, FactsFile, ProgramFacts, ProgramHistory,
    ProtocolFacts, RegistrySource, ReviewedNone, SourceKind, TokenIdentity, TokenProvenance,
};
use celebtir_core::{Address, Chain, Evidence, Fact};
use celebtir_registry::{MultisigProgram, ProgramSource, ProtocolEntry, Provenance, TokenEntry};

use crate::admin::{AdminSpec, read_admins};
use crate::control::upgrade_controls;
use crate::defillama::protocol_metrics;
use crate::deps::{TxCache, sample_dependencies};
use crate::history::upgrade_history;
use crate::idl::Idl;
use crate::mint::read_mints;
use crate::oracle::{OracleClassifier, OracleSpec, read_oracles};
use crate::program::ProgramKind;
use crate::reads::SolanaRpc;
use crate::resolve::{KnownControllers, RegistryMultisig, Resolver};
use crate::rpc::{RpcError, Transport};
use crate::scope::{ScopeChainSpec, ScopeResult, read_underlying};
use crate::stake_pool::{StakePoolSpec, read_stake_pools};
use crate::token2022::token_facts;
use crate::verify::{HttpGet, verification_status};

pub struct CollectInputs<'a> {
    pub protocols: &'a [ProtocolEntry],
    pub tokens: &'a [TokenEntry],
    /// Parsed IDLs keyed by their registry path (`idl/<file>.json`).
    pub idls: &'a BTreeMap<String, Idl>,
    pub oracles: &'a OracleClassifier,
    pub controllers: &'a KnownControllers,
    /// `registry/programs.yaml`: source facts keyed by program ID (F1).
    pub programs: &'a BTreeMap<String, ProgramSource>,
    /// Transactions sampled per core program (COL-14: 50).
    pub tx_limit: usize,
    /// New ProgramData transactions read per program per run (VAL-4).
    pub history_limit: usize,
    /// The previous run's facts, for carrying over a stale dependency list.
    pub previous: Option<&'a FactsFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    /// A protocol ID, or `token:<mint>`.
    pub scope: String,
    pub error: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewNote {
    pub protocol: String,
    pub note: String,
}

#[derive(Debug, Clone)]
pub struct CollectOutput {
    pub facts: FactsFile,
    pub failures: Vec<Failure>,
    pub review_needed: Vec<ReviewNote>,
    /// The run stopped early because the call budget ran out (COL-5).
    pub budget_exceeded: bool,
}

pub async fn collect<T: Transport, H: HttpGet>(
    rpc: &SolanaRpc<T>,
    http: &H,
    inputs: &CollectInputs<'_>,
    cache: &mut TxCache,
) -> CollectOutput {
    let mut out = CollectOutput {
        facts: FactsFile::new(BTreeMap::new()),
        failures: Vec::new(),
        review_needed: Vec::new(),
        budget_exceeded: false,
    };

    // F1: one verification status per program ID, shared by every protocol.
    let mut verified: BTreeMap<Address, CodeTransparency> = BTreeMap::new();
    for entry in inputs.protocols {
        match collect_protocol(rpc, http, inputs, entry, cache, &mut verified).await {
            Ok((facts, notes, history)) => {
                out.facts.protocols.insert(entry.id.clone(), facts);
                out.facts.program_history.extend(history);
                let metrics = protocol_metrics(
                    http,
                    entry.defillama_slug.as_deref(),
                    entry.category == celebtir_registry::Category::Dex,
                )
                .await;
                out.facts.metrics.insert(entry.id.clone(), metrics);
                out.review_needed
                    .extend(notes.into_iter().map(|note| ReviewNote {
                        protocol: entry.id.clone(),
                        note,
                    }));
            }
            Err(RpcError::BudgetExceeded { budget }) => {
                out.budget_exceeded = true;
                out.failures.push(Failure {
                    scope: entry.id.clone(),
                    error: format!("daily RPC budget of {budget} calls reached"),
                });
                return out;
            }
            Err(e) => out.failures.push(Failure {
                scope: entry.id.clone(),
                error: e.to_string(),
            }),
        }
    }

    if let Err(e) = collect_tokens(rpc, http, inputs, &mut out).await {
        out.budget_exceeded |= matches!(e, RpcError::BudgetExceeded { .. });
        out.failures.push(Failure {
            scope: "tokens".into(),
            error: e.to_string(),
        });
    }
    out
}

fn multisigs(entry: &ProtocolEntry) -> Vec<RegistryMultisig> {
    entry
        .multisigs
        .iter()
        .map(|m| RegistryMultisig {
            address: m.address.clone(),
            vault_index: m.vault_index,
            program: m.program,
        })
        .collect()
}

/// Edge case 11: the feeds behind a Scope price. The Scope IDL comes from the
/// price account's owner in `oracles.yaml`; the chain's from the registry.
async fn scope_underlying<T: Transport>(
    rpc: &SolanaRpc<T>,
    inputs: &CollectInputs<'_>,
    oracle: &Address,
    chain: &celebtir_registry::ScopeChain,
) -> Result<ScopeResult, RpcError> {
    scope_underlying_at(rpc, inputs, oracle, chain, &chain.account).await
}

/// The feeds one market (`account`, e.g. a Kamino reserve) reads through
/// `oracle`, plus any direct Pyth or Switchboard override it sets (F4).
async fn scope_underlying_at<T: Transport>(
    rpc: &SolanaRpc<T>,
    inputs: &CollectInputs<'_>,
    oracle: &Address,
    chain: &celebtir_registry::ScopeChain,
    account: &Address,
) -> Result<ScopeResult, RpcError> {
    let mut out = scope_chain_feeds(rpc, inputs, oracle, chain, account).await?;
    if chain.direct_feed_fields.is_empty() {
        return Ok(out);
    }
    let Some(idl) = inputs.idls.get(&chain.idl) else {
        return Ok(out);
    };
    let read = rpc
        .get_multiple_accounts(std::slice::from_ref(account), None)
        .await?
        .remove(0);
    let Some(acct) = &read.account else {
        return Ok(out);
    };
    let mut extra = Vec::new();
    for field in &chain.direct_feed_fields {
        match idl.read_pubkey(&acct.data, field) {
            // Kamino marks an unset feed with the zero key or `nu11…`.
            Ok(Some(k)) if k != [0u8; 32] => {
                let key = crate::pda::encode(k);
                if !key.as_str().starts_with("nu11111111") {
                    extra.push(celebtir_core::snapshot::UnderlyingFeed {
                        source_type: field.rsplit('.').nth(1).unwrap_or(field).to_string(),
                        address: key,
                    });
                }
            }
            Ok(_) => {}
            Err(e) => out
                .review
                .push(format!("{account}: cannot read `{field}`: {e}")),
        }
    }
    if let UnderlyingFeeds::Listed { feeds } = out.fact.value() {
        if !extra.is_empty() {
            let mut all = feeds.clone();
            all.extend(extra);
            all.sort();
            all.dedup();
            out.fact = Fact::new(
                UnderlyingFeeds::Listed { feeds: all },
                out.fact.evidence().to_vec(),
            )
            .expect("evidence is never empty");
        }
    }
    Ok(out)
}

async fn scope_chain_feeds<T: Transport>(
    rpc: &SolanaRpc<T>,
    inputs: &CollectInputs<'_>,
    oracle: &Address,
    chain: &celebtir_registry::ScopeChain,
    account: &Address,
) -> Result<ScopeResult, RpcError> {
    let read = rpc
        .get_multiple_accounts(std::slice::from_ref(oracle), Some((0, 0)))
        .await?
        .remove(0);
    let evidence = vec![Evidence::new(
        Chain::Solana,
        oracle.clone(),
        read.slot,
        "owner",
    )];
    let unavailable = |reason: String| ScopeResult {
        fact: Fact::new(UnderlyingFeeds::Unavailable { reason }, evidence.clone())
            .expect("evidence is never empty"),
        review: Vec::new(),
    };
    let Some(owner) = read.account.as_ref().map(|a| a.owner.clone()) else {
        return Ok(unavailable(format!(
            "oracle account {oracle} does not exist"
        )));
    };
    let scope_idl = inputs
        .oracles
        .program(&owner)
        .and_then(|p| p.idl.as_ref())
        .and_then(|path| inputs.idls.get(path));
    let Some(scope_idl) = scope_idl else {
        return Ok(unavailable(format!(
            "oracles.yaml gives no loaded IDL for {oracle}'s owner {owner}"
        )));
    };
    let Some(chain_idl) = inputs.idls.get(&chain.idl) else {
        return Ok(unavailable(format!("IDL {} is not loaded", chain.idl)));
    };
    let spec = ScopeChainSpec {
        chain_account: account.clone(),
        chain_idl,
        feed_field: &chain.feed_field,
        chain_field: &chain.chain_field,
        scope_idl,
    };
    read_underlying(rpc, oracle, &spec).await
}

async fn collect_protocol<T: Transport, H: HttpGet>(
    rpc: &SolanaRpc<T>,
    http: &H,
    inputs: &CollectInputs<'_>,
    entry: &ProtocolEntry,
    cache: &mut TxCache,
    verified: &mut BTreeMap<Address, CodeTransparency>,
) -> Result<(ProtocolFacts, Vec<String>, Vec<(Address, ProgramHistory)>), RpcError> {
    let registry_multisigs = multisigs(entry);
    let resolver = Resolver::new(rpc, &registry_multisigs, inputs.controllers).with_pdas(
        inputs
            .protocols
            .iter()
            .flat_map(|p| p.authority_pdas.clone())
            .chain(inputs.tokens.iter().flat_map(|t| t.authority_pdas.clone())),
    );
    let mut review = Vec::new();

    let program_ids: Vec<Address> = entry.programs.iter().map(|p| p.id.clone()).collect();
    let controls = upgrade_controls(rpc, &resolver, &program_ids).await?;
    let mut programs = BTreeMap::new();
    let mut history = Vec::new();
    for (p, control) in entry.programs.iter().zip(controls) {
        if let (
            ProgramKind::Upgradeable { programdata },
            celebtir_core::snapshot::ProgramRole::Core,
        ) = (&control.read.kind, p.role)
        {
            let h = upgrade_history(rpc, programdata, cache, inputs.history_limit).await?;
            history.push((p.id.clone(), h));
        }
        review.extend(control.review_needed);
        let origin = Evidence::new(
            Chain::Solana,
            p.id.clone(),
            control.read.program_slot,
            "verified_build",
        );
        let status = match verified.get(&p.id) {
            Some(s) => s.clone(),
            None => {
                let s = verification_status(http, &p.id, inputs.programs.get(p.id.as_str())).await;
                verified.insert(p.id.clone(), s.clone());
                s
            }
        };
        let code_transparency = Fact::single(status, origin);
        programs.insert(
            p.id.clone(),
            ProgramFacts {
                role: p.role,
                loader: control.read.kind.loader(),
                upgrade_control: control.upgrade_control,
                code_transparency,
            },
        );
    }

    let mut specs = Vec::new();
    let mut admin = BTreeMap::new();
    for a in &entry.admin_accounts {
        match inputs.idls.get(&a.idl) {
            Some(idl) => specs.push(AdminSpec {
                account: a.account.clone(),
                field: a.field.clone(),
                power: a.power,
                idl,
            }),
            None => {
                review.push(format!(
                    "IDL {} for admin account {} is not loaded",
                    a.idl, a.account
                ));
                admin.insert(
                    format!("{}.{}", a.account, a.field),
                    celebtir_core::snapshot::AdminFact {
                        account: a.account.clone(),
                        field: a.field.clone(),
                        power: a.power,
                        controller: Fact::single(
                            celebtir_core::Controller::Unavailable {
                                reason: format!("IDL {} is not loaded", a.idl),
                            },
                            Evidence::new(Chain::Solana, a.account.clone(), 0, a.field.clone()),
                        ),
                    },
                );
            }
        }
    }
    if !specs.is_empty() {
        admin.extend(read_admins(rpc, &resolver, &specs, &program_ids).await?);
    }
    let pools: Vec<StakePoolSpec> = entry
        .stake_pools
        .iter()
        .map(|s| StakePoolSpec {
            account: s.account.clone(),
            pool_mint: s.pool_mint.clone(),
        })
        .collect();
    if !pools.is_empty() {
        admin.extend(read_stake_pools(rpc, &resolver, &pools, &program_ids).await?);
    }

    let oracle_specs: Vec<OracleSpec> = entry
        .oracles
        .iter()
        .map(|o| OracleSpec {
            account: o.account.clone(),
            market: o.market.clone(),
            role: o.role,
            bounds_primary: o.bounds_primary,
            guarded_by: o.guarded_by.clone(),
            provider: o.provider.clone(),
            verified_by: o.verified_by.clone(),
        })
        .collect();
    let mut oracles = if oracle_specs.is_empty() {
        BTreeMap::new()
    } else {
        read_oracles(rpc, &oracle_specs, inputs.oracles).await?
    };
    for o in &entry.oracles {
        let (Some(chain), Some(fact)) = (&o.scope_chain, oracles.get_mut(&o.account)) else {
            continue;
        };
        let out = scope_underlying(rpc, inputs, &o.account, chain).await?;
        review.extend(out.review);
        fact.underlying = Some(out.fact);
        for r in &chain.reserves {
            let out = scope_underlying_at(rpc, inputs, &o.account, chain, &r.account).await?;
            review.extend(out.review.into_iter().map(|n| format!("{}: {n}", r.market)));
            fact.underlying_markets.insert(r.market.clone(), out.fact);
        }
    }

    let core: Vec<Address> = entry
        .programs
        .iter()
        .filter(|p| p.role == celebtir_core::snapshot::ProgramRole::Core)
        .map(|p| p.id.clone())
        .collect();
    let previous = inputs
        .previous
        .and_then(|f| f.protocols.get(&entry.id))
        .map(|p| &p.dependencies);
    let dependencies =
        dependencies(rpc, &resolver, inputs, &core, &program_ids, previous, cache).await?;

    let tokens = entry
        .tokens
        .iter()
        .map(|t| (t.mint.clone(), t.roles.clone()))
        .collect();

    let mut facts = ProtocolFacts {
        chain: entry.chain,
        category: Some(entry.category.as_str().to_string()),
        last_reviewed: Some(entry.review.last_reviewed.clone()),
        name: Some(entry.name.clone()),
        website: Some(entry.website.to_string()),
        reviewed_by: Some(entry.review.reviewed_by.clone()),
        sources: registry_sources(entry),
        programs,
        admin,
        oracles,
        admin_review: entry.admin_reviewed_none.as_ref().map(reviewed_none),
        oracle_review: entry.oracles_reviewed_none.as_ref().map(reviewed_none),
        dependencies,
        tokens,
        min_slot: 0,
        max_slot: 0,
    };
    let slots = evidence_slots(&facts);
    facts.min_slot = slots.first().copied().unwrap_or(0);
    facts.max_slot = slots.last().copied().unwrap_or(0);
    Ok((facts, review, history))
}

fn reviewed_none(r: &celebtir_registry::ReviewedNone) -> ReviewedNone {
    ReviewedNone {
        source: r.source.to_string(),
        reviewed_on: r.reviewed_on.clone(),
        note: r.note.clone(),
    }
}

/// ADR 006: each direct callee's upgrade control is read and resolved.
async fn dependencies<T: Transport>(
    rpc: &SolanaRpc<T>,
    resolver: &Resolver<'_, T>,
    inputs: &CollectInputs<'_>,
    core: &[Address],
    own: &[Address],
    previous: Option<&Fact<Dependencies>>,
    cache: &mut TxCache,
) -> Result<Fact<Dependencies>, RpcError> {
    let sample = sample_dependencies(rpc, core, own, cache, inputs.tx_limit).await?;
    let first_core = core.first().cloned().unwrap_or_else(|| Address::new(""));
    let origin = Evidence::new(
        Chain::Solana,
        first_core,
        sample.newest_slot.unwrap_or(0),
        format!("transaction sample ({} transactions)", sample.transactions),
    );
    if sample.transactions == 0 {
        // Edge case: keep the previous list, marked stale, with its old evidence.
        if let Some(prev) = previous {
            if let Dependencies::Sampled { callees, .. } = prev.value() {
                let stale = Dependencies::Sampled {
                    callees: callees.clone(),
                    stale: true,
                    declared: false,
                };
                return Ok(Fact::new(stale, prev.evidence().to_vec())
                    .expect("previous facts carry evidence"));
            }
        }
        return Ok(Fact::single(
            Dependencies::Unavailable {
                reason: "no readable transactions in the sample window".into(),
            },
            origin,
        ));
    }
    let callees: Vec<Address> = sample.callees.into_iter().collect();
    let controls = if callees.is_empty() {
        Vec::new()
    } else {
        upgrade_controls(rpc, resolver, &callees).await?
    };
    let callees = controls
        .into_iter()
        .map(|c| {
            (
                c.read.program.clone(),
                DependencyFact {
                    upgrade_control: c.upgrade_control,
                },
            )
        })
        .collect();
    Ok(Fact::single(
        Dependencies::Sampled {
            callees,
            stale: false,
            declared: false,
        },
        origin,
    ))
}

async fn collect_tokens<T: Transport, H: HttpGet>(
    rpc: &SolanaRpc<T>,
    http: &H,
    inputs: &CollectInputs<'_>,
    out: &mut CollectOutput,
) -> Result<(), RpcError> {
    if inputs.tokens.is_empty() {
        return Ok(());
    }
    let all_multisigs: Vec<RegistryMultisig> =
        inputs.protocols.iter().flat_map(multisigs).collect();
    let resolver = Resolver::new(rpc, &all_multisigs, inputs.controllers).with_pdas(
        inputs
            .protocols
            .iter()
            .flat_map(|p| p.authority_pdas.clone())
            .chain(inputs.tokens.iter().flat_map(|t| t.authority_pdas.clone())),
    );
    let mints: Vec<Address> = inputs.tokens.iter().map(|t| t.mint.clone()).collect();
    out.facts.token_metrics = crate::defillama::token_prices(http, &mints).await;
    for (entry, read) in inputs.tokens.iter().zip(read_mints(rpc, &mints).await?) {
        let mint = match read {
            Ok(m) => m,
            Err(e) => {
                out.failures.push(Failure {
                    scope: format!("token:{}", entry.mint),
                    error: e.0,
                });
                continue;
            }
        };
        let provenance = match &entry.provenance {
            Provenance::Native { .. } => TokenProvenance::Native,
            Provenance::Bridged {
                bridge,
                backing_chain,
                backing_address,
                ..
            } => TokenProvenance::Bridged {
                bridge: bridge.clone(),
                backing_chain: backing_chain.clone(),
                backing_address: backing_address.clone(),
            },
        };
        let (mut facts, notes) =
            token_facts(rpc, &resolver, &mint, &entry.symbol, provenance).await?;
        out.review_needed
            .extend(notes.into_iter().map(|note| ReviewNote {
                protocol: format!("token:{}", entry.mint),
                note,
            }));
        facts.identity = Some(TokenIdentity {
            kind: serde_json::to_value(entry.kind)
                .ok()
                .and_then(|v| v.as_str().map(str::to_string))
                .unwrap_or_else(|| "other".into()),
            issuer: entry.issuer.name.clone(),
            issuer_url: entry.issuer.source.to_string(),
            source: entry.source.to_string(),
        });
        out.facts.tokens.insert(entry.mint.clone(), facts);
    }
    Ok(())
}

/// Every evidence slot in a protocol's facts, sorted (COL-3).
fn evidence_slots(p: &ProtocolFacts) -> Vec<u64> {
    let mut slots: BTreeSet<u64> = BTreeSet::new();
    let mut add = |ev: &[Evidence]| slots.extend(ev.iter().map(|e| e.slot).filter(|s| *s > 0));
    for f in p.programs.values() {
        add(f.upgrade_control.evidence());
        add(f.code_transparency.evidence());
    }
    for a in p.admin.values() {
        add(a.controller.evidence());
    }
    for o in p.oracles.values() {
        add(o.source.evidence());
    }
    add(p.dependencies.evidence());
    if let Dependencies::Sampled { callees, .. } = p.dependencies.value() {
        for c in callees.values() {
            add(c.upgrade_control.evidence());
        }
    }
    slots.into_iter().collect()
}

/// The registry's source link for every address it lists, sorted (SNAP-2).
fn registry_sources(entry: &ProtocolEntry) -> Vec<RegistrySource> {
    let source =
        |kind, address: &Address, label: &str, url: &dyn std::fmt::Display| RegistrySource {
            kind,
            address: address.clone(),
            label: label.to_string(),
            url: url.to_string(),
        };
    let mut out: Vec<RegistrySource> = entry
        .programs
        .iter()
        .map(|p| source(SourceKind::Program, &p.id, &p.name, &p.source))
        .chain(entry.multisigs.iter().map(|m| {
            // F12: the label is the program the reader decodes it with.
            let label = match m.program {
                MultisigProgram::SquadsV4 => "Squads v4",
                MultisigProgram::SquadsV3 => "Squads v3",
            };
            source(SourceKind::Multisig, &m.address, label, &m.source)
        }))
        .chain(
            entry
                .admin_accounts
                .iter()
                .map(|a| source(SourceKind::Admin, &a.account, &a.field, &a.source)),
        )
        .chain(
            entry
                .oracles
                .iter()
                .map(|o| source(SourceKind::Oracle, &o.account, &o.market, &o.source)),
        )
        .chain(
            entry
                .stake_pools
                .iter()
                .map(|s| source(SourceKind::Admin, &s.account, "stake pool", &s.source)),
        )
        .collect();
    // F4: report verifiers are also called programs. Reuse the sourced
    // oracle registration so dependency views can name them without
    // confusing the verifier with a protocol-owned core program.
    for oracle in &entry.oracles {
        if let Some(verifier) = &oracle.verified_by {
            out.push(source(
                SourceKind::Program,
                verifier,
                &format!(
                    "{} verifier",
                    oracle.provider.as_deref().unwrap_or("Oracle report")
                ),
                &oracle.source,
            ));
        }
    }
    out.sort();
    out.dedup();
    out
}
