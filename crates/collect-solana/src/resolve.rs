//! Controller resolution (COL-8, COL-10): follow an authority key to its
//! effective controller.
//!
//! 1. The key is the Squads v4 vault of a registry multisig → read and decode
//!    the multisig; resolve a set config authority the same way.
//! 2. Else the key is on the ed25519 curve → a single key (wallet).
//! 3. Else it is a program-derived address (no private key can exist). A
//!    verified registry declaration resolves it to its controlling program;
//!    otherwise it is `Pda`, with its owner program, and raises a review flag
//!    (methodology v0.3: never a key, never "unrecognized").

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use celebtir_core::{Address, Chain, Controller, Evidence, Fact, MultisigKind};
pub use celebtir_registry::MultisigProgram;
use tokio::sync::Mutex;

use crate::pda;
use crate::reads::{AccountRead, SolanaRpc};
use crate::rpc::{RpcError, Transport};
use crate::squads;

/// Config authorities are followed at most this many levels (COL-10).
pub const MAX_DEPTH: usize = 3;

/// A multisig listed in the registry, with the vault it signs from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryMultisig {
    pub address: Address,
    /// v4 vault index, or v3 authority index.
    pub vault_index: u8,
    pub program: MultisigProgram,
}

/// `registry/controllers.yaml`: owner programs Celebtir knows by name but
/// cannot yet decode (Realms, Squads v3, …).
#[derive(Debug, Clone, Default)]
pub struct KnownControllers(BTreeMap<Address, String>);

impl KnownControllers {
    pub fn new(programs: BTreeMap<Address, String>) -> Self {
        Self(programs)
    }

    pub fn name(&self, program: &Address) -> Option<&str> {
        self.0.get(program).map(String::as_str)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolution {
    pub fact: Fact<Controller>,
    /// Problems a registry reviewer should look at, e.g. an off-curve
    /// authority that matches no registry multisig.
    pub review_needed: Vec<String>,
}

pub struct Resolver<'a, T> {
    rpc: &'a SolanaRpc<T>,
    pdas: BTreeMap<Address, Vec<celebtir_registry::AuthorityPda>>,
    /// Vault (v4) or authority (v3) address → multisig address and program.
    vaults: BTreeMap<Address, (Address, MultisigProgram)>,
    known: &'a KnownControllers,
    cache: Mutex<BTreeMap<(Address, bool), AccountRead>>,
}

struct Step {
    controller: Controller,
    evidence: Vec<Evidence>,
    review: Vec<String>,
}

type StepFuture<'f> = Pin<Box<dyn Future<Output = Result<Step, RpcError>> + Send + 'f>>;

impl<'a, T: Transport> Resolver<'a, T> {
    pub fn new(
        rpc: &'a SolanaRpc<T>,
        multisigs: &[RegistryMultisig],
        known: &'a KnownControllers,
    ) -> Self {
        let vaults = multisigs
            .iter()
            .filter_map(|m| {
                let vault = match m.program {
                    MultisigProgram::SquadsV4 => pda::squads_vault(&m.address, m.vault_index)?,
                    MultisigProgram::SquadsV3 => {
                        pda::squads_v3_authority(&m.address, u32::from(m.vault_index))?
                    }
                };
                Some((vault, (m.address.clone(), m.program)))
            })
            .collect();
        Self {
            rpc,
            pdas: BTreeMap::new(),
            vaults,
            known,
            cache: Mutex::new(BTreeMap::new()),
        }
    }

    /// Keep duplicates so conflicting declarations fail closed rather than last-write-wins.
    pub fn with_pdas(
        mut self,
        declarations: impl IntoIterator<Item = celebtir_registry::AuthorityPda>,
    ) -> Self {
        for d in declarations {
            self.pdas.entry(d.key.clone()).or_default().push(d);
        }
        self
    }

    /// Reads accounts ahead in one batched call so resolution hits the cache.
    pub async fn prefetch(&self, addresses: &[Address], full: bool) -> Result<(), RpcError> {
        let slice = (!full).then_some((0, 0));
        let reads = self.rpc.get_multiple_accounts(addresses, slice).await?;
        let mut cache = self.cache.lock().await;
        for r in reads {
            cache.insert((r.address.clone(), full), r);
        }
        Ok(())
    }

    async fn account(&self, address: &Address, full: bool) -> Result<AccountRead, RpcError> {
        if let Some(r) = self.cache.lock().await.get(&(address.clone(), full)) {
            return Ok(r.clone());
        }
        let slice = (!full).then_some((0, 0));
        let read = self
            .rpc
            .get_multiple_accounts(std::slice::from_ref(address), slice)
            .await?
            .remove(0);
        self.cache
            .lock()
            .await
            .insert((address.clone(), full), read.clone());
        Ok(read)
    }

    /// Resolves `key`, citing `origin` (where the key was read) first.
    pub async fn resolve(&self, key: &Address, origin: Evidence) -> Result<Resolution, RpcError> {
        let step = self.step(key.clone(), Vec::new()).await?;
        let mut evidence = vec![origin];
        for e in step.evidence {
            if !evidence.contains(&e) {
                evidence.push(e);
            }
        }
        Ok(Resolution {
            fact: Fact::new(step.controller, evidence).expect("origin evidence is always present"),
            review_needed: step.review,
        })
    }

    fn step(&self, key: Address, chain: Vec<Address>) -> StepFuture<'_> {
        Box::pin(async move {
            let done = |controller| Step {
                controller,
                evidence: Vec::new(),
                review: Vec::new(),
            };
            if chain.contains(&key) {
                let path: Vec<&str> = chain.iter().chain([&key]).map(Address::as_str).collect();
                return Ok(done(Controller::Unavailable {
                    reason: format!("config authority loop: {}", path.join(" → ")),
                }));
            }
            if chain.len() > MAX_DEPTH {
                return Ok(done(Controller::Unavailable {
                    reason: format!("config authority chain deeper than {MAX_DEPTH} levels"),
                }));
            }

            if let Some((ms, program)) = self.vaults.get(&key).cloned() {
                return self.multisig(key, ms, program, chain).await;
            }

            let Some(bytes) = pda::decode(&key) else {
                return Ok(done(Controller::Unavailable {
                    reason: format!("{key} is not a valid address"),
                }));
            };
            if pda::is_on_curve(&bytes) {
                return Ok(done(Controller::SingleKey { key }));
            }

            if let Some(declarations) = self.pdas.get(&key) {
                let declaration = &declarations[0];
                let valid = declarations.iter().all(|d| d == declaration)
                    && crate::pda::declared_address(declaration).as_ref() == Some(&key);
                if !valid {
                    return Ok(Step {
                        controller: Controller::Pda {
                            key: key.clone(),
                            owner_program: None,
                        },
                        evidence: vec![],
                        review: vec![format!(
                            "PDA declaration mismatch or conflict for {key}; registry review needed"
                        )],
                    });
                }
                let account = self.account(&declaration.program, false).await?;
                if !account.account.as_ref().is_some_and(|a| a.executable) {
                    return Ok(Step {
                        controller: Controller::Unavailable {
                            reason: "declared controlling program is missing or not executable"
                                .into(),
                        },
                        evidence: vec![Evidence::new(
                            Chain::Solana,
                            declaration.program.clone(),
                            account.slot,
                            "executable",
                        )],
                        review: vec![format!(
                            "PDA controlling program {} requires review",
                            declaration.program
                        )],
                    });
                }
                let reads = crate::program::read_program_control(
                    self.rpc,
                    std::slice::from_ref(&declaration.program),
                )
                .await?;
                let read = &reads[0];
                let mut evidence = vec![Evidence::new(
                    Chain::Solana,
                    declaration.program.clone(),
                    read.program_slot,
                    format!(
                        "PDA derivation verified; registry source: {}",
                        declaration.source
                    ),
                )];
                let mut review = vec![];
                let upgrade = if let Some(error) = &read.error {
                    Controller::Unavailable {
                        reason: error.clone(),
                    }
                } else {
                    match &read.kind {
                        crate::program::ProgramKind::Immutable { .. } => Controller::Immutable,
                        crate::program::ProgramKind::Upgradeable { programdata } => {
                            evidence.push(Evidence::new(
                                Chain::Solana,
                                programdata.clone(),
                                read.programdata_slot.unwrap_or(read.program_slot),
                                "upgrade_authority",
                            ));
                            match &read.header {
                                Some(header) => match &header.authority {
                                    None => Controller::Immutable,
                                    Some(authority) => {
                                        let mut path = chain.clone();
                                        path.push(key.clone());
                                        let step = self.step(authority.clone(), path).await?;
                                        evidence.extend(step.evidence);
                                        review.extend(step.review);
                                        step.controller
                                    }
                                },
                                None => Controller::Unavailable {
                                    reason: "ProgramData was not read".into(),
                                },
                            }
                        }
                        crate::program::ProgramKind::UnsupportedLoader { owner } => {
                            Controller::Unrecognized {
                                key: declaration.program.clone(),
                                owner_program: Some(owner.clone()),
                            }
                        }
                        crate::program::ProgramKind::Closed => Controller::Unavailable {
                            reason: "controlling program is closed".into(),
                        },
                    }
                };
                return Ok(Step {
                    controller: Controller::ProgramControlled {
                        key,
                        program: declaration.program.clone(),
                        upgrade: Box::new(upgrade),
                    },
                    evidence,
                    review,
                });
            }

            let read = self.account(&key, false).await?;
            let owner = read.account.as_ref().map(|a| a.owner.clone());
            let evidence = vec![Evidence::new(
                Chain::Solana,
                key.clone(),
                read.slot,
                "owner",
            )];
            let review = match &owner {
                // A controller program `controllers.yaml` names (Realms, …).
                Some(o) if self.known.name(o).is_some() => Vec::new(),
                Some(o) => vec![format!(
                    "authority {key} is a program-derived address owned by {o}; its controlling program is not resolved by a registry declaration; registry review needed"
                )],
                None => vec![format!(
                    "authority {key} is a program-derived address with no account (possibly the vault of a multisig missing from the registry); registry review needed"
                )],
            };
            Ok(Step {
                controller: Controller::Pda {
                    key,
                    owner_program: owner,
                },
                evidence,
                review,
            })
        })
    }

    async fn multisig(
        &self,
        vault: Address,
        ms: Address,
        program: MultisigProgram,
        mut chain: Vec<Address>,
    ) -> Result<Step, RpcError> {
        let read = self.account(&ms, true).await?;
        let mut evidence = vec![Evidence::new(
            Chain::Solana,
            ms.clone(),
            read.slot,
            "threshold, time_lock, members, config_authority",
        )];
        let (decoded, kind) = match program {
            MultisigProgram::SquadsV4 => (squads::decode_multisig(&read), MultisigKind::SquadsV4),
            MultisigProgram::SquadsV3 => {
                (squads::decode_v3_multisig(&read), MultisigKind::SquadsV3)
            }
        };
        let decoded = match decoded {
            Ok(d) => d,
            Err(e) => {
                return Ok(Step {
                    controller: Controller::Unavailable {
                        reason: format!("multisig {ms}: {e}"),
                    },
                    evidence,
                    review: Vec::new(),
                });
            }
        };
        let voters = decoded.voter_count();
        let mut signers: Vec<Address> = decoded.voters().cloned().collect();
        signers.sort();
        signers.dedup();
        let threshold = decoded.threshold;

        let mut review = Vec::new();
        // Edge case 5: a signer that is not a wallet key (on-curve) is
        // resolved too, so the graph can show who stands behind it.
        let mut nested_signers = BTreeMap::new();
        for signer in &signers {
            let on_curve = pda::decode(signer).is_some_and(|b| pda::is_on_curve(&b));
            if on_curve {
                continue;
            }
            let mut nested_chain = chain.clone();
            nested_chain.push(vault.clone());
            let inner = self.step(signer.clone(), nested_chain).await?;
            for e in inner.evidence {
                if !evidence.contains(&e) {
                    evidence.push(e);
                }
            }
            review.extend(inner.review);
            nested_signers.insert(signer.clone(), Box::new(inner.controller));
        }

        let config_authority = match &decoded.config_authority {
            None => None,
            Some(ca) => {
                chain.push(vault);
                let inner = self.step(ca.clone(), chain).await?;
                evidence.extend(inner.evidence);
                review.extend(inner.review);
                Some(Box::new(inner.controller))
            }
        };

        let controller = if config_authority.is_none() && (voters == 0 || threshold > voters) {
            Controller::CannotAct {
                address: ms,
                threshold,
                voters,
            }
        } else {
            Controller::Multisig {
                kind,
                address: ms,
                threshold,
                voters,
                time_lock_secs: decoded.time_lock_secs,
                config_authority,
                signers,
                nested_signers,
            }
        };
        Ok(Step {
            controller,
            evidence,
            review,
        })
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ControllersFile {
    programs: Vec<ControllerProgram>,
    /// HyperEVM entries (HREG-3), read by the Hyperliquid collector.
    #[serde(default, rename = "evm")]
    _evm: Option<serde::de::IgnoredAny>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ControllerProgram {
    program: Address,
    name: String,
    #[allow(dead_code)]
    source: String,
}

impl KnownControllers {
    /// `registry/controllers.yaml`: `programs: [{program, name, source}]`.
    pub fn from_yaml(text: &str) -> Result<Self, String> {
        let f: ControllersFile = serde_norway::from_str(text).map_err(|e| e.to_string())?;
        Ok(Self::new(
            f.programs
                .into_iter()
                .map(|p| (p.program, p.name))
                .collect(),
        ))
    }
}
