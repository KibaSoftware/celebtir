//! HyperEVM controller resolution (HCOL-3 to HCOL-6, ADR 024 D3 and D4).
//!
//! A contract's upgrade path is read from its proxy kind, then the address
//! that can upgrade it is classified:
//!
//! 1. no code: a single key;
//! 2. a Safe: storage slot 0 is a Safe singleton from `controllers.yaml`,
//!    and `getThreshold`/`getOwners` answer;
//! 3. a timelock: listed in the registry (proposers confirmed), or any
//!    contract answering `getMinDelay()` (an OpenZeppelin
//!    `TimelockController`; unlisted proposers are unreadable, F10);
//! 4. a contract with an `owner()` (an OpenZeppelin `ProxyAdmin` or any
//!    `Ownable`): resolved through its owner;
//! 5. anything else: Unrecognized.
//!
//! Depth is capped at 3 and loops are detected (as COL-10). A failed read is
//! `Unavailable`, never a guess.

use std::collections::{BTreeMap, BTreeSet};

use celebtir_core::{Address, Chain, Controller, Evidence, MultisigKind};
use celebtir_registry::{EvmController, EvmControllerKind, ProxyKind, TimelockEntry, evm::eip55};
use collect_solana::rpc::Transport;
use sha3::{Digest, Keccak256};

use crate::evm::{EvmError, HyperEvm, Word};

/// `keccak256("eip1967.proxy.implementation") - 1`.
pub const IMPLEMENTATION_SLOT: &str =
    "360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc";
/// `keccak256("eip1967.proxy.admin") - 1`.
pub const ADMIN_SLOT: &str = "b53127684a568b3173ae13b9f8a6016e243e63b6e8ee1178d6a717850b5d6103";
pub const MAX_DEPTH: usize = 3;
/// Safe's sentinel for the end of its module list.
const SENTINEL: &str = "0x0000000000000000000000000000000000000001";

/// The first four bytes of `keccak256(signature)`.
pub fn selector(signature: &str) -> [u8; 4] {
    let h = Keccak256::digest(signature.as_bytes());
    [h[0], h[1], h[2], h[3]]
}

pub fn keccak(text: &str) -> Word {
    Keccak256::digest(text.as_bytes()).into()
}

fn parse_word(hex: &str) -> Word {
    let mut w = [0u8; 32];
    for (i, b) in w.iter_mut().enumerate() {
        *b = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).expect("constant hex");
    }
    w
}

/// The low 20 bytes of a word as an EIP-55 address.
pub fn word_address(w: &[u8]) -> Address {
    let hex: String = w[12..32].iter().map(|b| format!("{b:02x}")).collect();
    Address::new(eip55(&format!("0x{hex}")).expect("40 hex digits"))
}

fn is_zero(w: &[u8]) -> bool {
    w.iter().all(|b| *b == 0)
}

fn pad_address(a: &Address) -> [u8; 32] {
    let mut w = [0u8; 32];
    let hex = a.as_str().trim_start_matches("0x");
    for i in 0..20 {
        w[12 + i] = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).unwrap_or(0);
    }
    w
}

fn calldata(signature: &str, args: &[[u8; 32]]) -> Vec<u8> {
    let mut d = selector(signature).to_vec();
    for a in args {
        d.extend_from_slice(a);
    }
    d
}

fn u256_low(w: &[u8]) -> Option<u64> {
    if w.len() < 32 || !is_zero(&w[..24]) {
        return None;
    }
    Some(u64::from_be_bytes(w[24..32].try_into().ok()?))
}

/// ABI `address[]` return data.
fn address_array(data: &[u8]) -> Option<Vec<Address>> {
    let offset = u256_low(data.get(0..32)?)? as usize;
    let len = u256_low(data.get(offset..offset + 32)?)? as usize;
    (0..len)
        .map(|i| {
            let start = offset + 32 + 32 * i;
            data.get(start..start + 32).map(word_address)
        })
        .collect()
}

/// What `controllers.yaml` and the registry say about EVM addresses.
#[derive(Debug, Clone, Default)]
pub struct EvmKnown {
    safe_singletons: BTreeSet<String>,
    /// Timelock (lower case) → its registry entry.
    timelocks: BTreeMap<String, TimelockEntry>,
}

impl EvmKnown {
    pub fn new(controllers: &[EvmController], timelocks: &[TimelockEntry]) -> Self {
        Self {
            safe_singletons: controllers
                .iter()
                .filter(|c| c.kind == EvmControllerKind::SafeSingleton)
                .map(|c| c.address.as_str().to_ascii_lowercase())
                .collect(),
            timelocks: timelocks
                .iter()
                .map(|t| (t.address.as_str().to_ascii_lowercase(), t.clone()))
                .collect(),
        }
    }

    /// `controllers.yaml`'s `evm:` section.
    pub fn controllers_from_yaml(text: &str) -> Result<Vec<EvmController>, String> {
        #[derive(serde::Deserialize)]
        struct File {
            #[serde(default)]
            evm: Vec<EvmController>,
        }
        serde_norway::from_str::<File>(text)
            .map(|f| f.evm)
            .map_err(|e| e.to_string())
    }
}

/// A resolved controller and every read behind it.
#[derive(Debug, Clone, PartialEq)]
pub struct Resolved {
    pub controller: Controller,
    pub evidence: Vec<Evidence>,
}

pub struct Resolver<'a, T> {
    pub evm: &'a HyperEvm<T>,
    pub known: &'a EvmKnown,
}

impl<T: Transport> Resolver<'_, T> {
    fn ev(&self, address: &Address, field: &str) -> Evidence {
        Evidence::new(Chain::HyperEvm, address.clone(), self.evm.block(), field)
    }

    async fn word_call(
        &self,
        to: &Address,
        sig: &str,
        args: &[[u8; 32]],
    ) -> Result<Vec<u8>, EvmError> {
        self.evm.call(to.as_str(), &calldata(sig, args)).await
    }

    /// HCOL-3: who can change the code of `contract`.
    pub async fn upgrade_control(&self, contract: &Address, proxy: ProxyKind) -> Resolved {
        match proxy {
            ProxyKind::None => Resolved {
                controller: Controller::Immutable,
                evidence: vec![self.ev(contract, "proxy: none (registry)")],
            },
            ProxyKind::Eip1967 => {
                let field = "eip1967.admin";
                match self
                    .evm
                    .storage(contract.as_str(), &parse_word(ADMIN_SLOT))
                    .await
                {
                    Err(e) => self.unavailable(contract, field, e.to_string()),
                    Ok(w) if is_zero(&w) => {
                        self.unavailable(contract, field, "admin slot empty".into())
                    }
                    Ok(w) => {
                        let mut r = self
                            .classify(&word_address(&w), 1, &mut BTreeSet::new())
                            .await;
                        r.evidence.insert(0, self.ev(contract, field));
                        r
                    }
                }
            }
            ProxyKind::UupsRoleRegistry => {
                // `_authorizeUpgrade` → `roleRegistry.onlyProtocolUpgrader`,
                // which requires the role registry's `owner()`.
                let field = "roleRegistry()";
                match self.word_call(contract, "roleRegistry()", &[]).await {
                    Ok(d) if d.len() == 32 && !is_zero(&d) => {
                        let registry = word_address(&d);
                        match self.word_call(&registry, "owner()", &[]).await {
                            Ok(o) if o.len() == 32 && !is_zero(&o) => {
                                let mut r = self
                                    .classify(&word_address(&o), 1, &mut BTreeSet::new())
                                    .await;
                                r.evidence.insert(
                                    0,
                                    self.ev(&registry, "owner() (onlyProtocolUpgrader)"),
                                );
                                r.evidence.insert(0, self.ev(contract, field));
                                r
                            }
                            Ok(_) => self.unavailable(
                                &registry,
                                "owner()",
                                "role registry owner() returned no address".into(),
                            ),
                            Err(e) => self.unavailable(&registry, "owner()", e.to_string()),
                        }
                    }
                    Ok(_) => self.unavailable(
                        contract,
                        field,
                        "roleRegistry() returned no address".into(),
                    ),
                    Err(e) => self.unavailable(contract, field, e.to_string()),
                }
            }
            ProxyKind::Uups => {
                // The proxy delegates `owner()` to its implementation.
                let field = "uups.owner()";
                match self.word_call(contract, "owner()", &[]).await {
                    Ok(d) if d.len() == 32 && !is_zero(&d) => {
                        let mut r = self
                            .classify(&word_address(&d), 1, &mut BTreeSet::new())
                            .await;
                        r.evidence.insert(0, self.ev(contract, field));
                        r
                    }
                    Ok(_) => {
                        self.unavailable(contract, field, "owner() returned no address".into())
                    }
                    Err(e) => self.unavailable(contract, field, e.to_string()),
                }
            }
        }
    }

    /// HCOL-7: who holds an admin power on `contract`: its `owner()`, or a
    /// registry-named `holder` of `role`, confirmed with `hasRole`.
    pub async fn admin(
        &self,
        contract: &Address,
        role: Option<&str>,
        holder: Option<&Address>,
    ) -> Resolved {
        match (role, holder) {
            (Some(role), Some(holder)) => {
                let id = if role == "DEFAULT_ADMIN_ROLE" {
                    [0u8; 32]
                } else {
                    keccak(role)
                };
                let field = format!("hasRole({role}, {holder})");
                match self
                    .word_call(
                        contract,
                        "hasRole(bytes32,address)",
                        &[id, pad_address(holder)],
                    )
                    .await
                    .ok()
                    .and_then(|d| u256_low(&d))
                {
                    Some(1) => {
                        let mut r = self.classify(holder, 1, &mut BTreeSet::new()).await;
                        r.evidence.insert(0, self.ev(contract, &field));
                        r
                    }
                    Some(_) => {
                        self.unavailable(contract, &field, format!("{holder} does not hold {role}"))
                    }
                    None => self.unavailable(contract, &field, "hasRole() failed".into()),
                }
            }
            _ => match self.word_call(contract, "owner()", &[]).await {
                Ok(d) if d.len() == 32 && is_zero(&d) => Resolved {
                    controller: Controller::Immutable,
                    evidence: vec![self.ev(contract, "owner() (zero: renounced)")],
                },
                Ok(d) if d.len() == 32 => {
                    let mut r = self
                        .classify(&word_address(&d), 1, &mut BTreeSet::new())
                        .await;
                    r.evidence.insert(0, self.ev(contract, "owner()"));
                    r
                }
                Ok(_) => {
                    self.unavailable(contract, "owner()", "owner() returned no address".into())
                }
                Err(e) => self.unavailable(contract, "owner()", e.to_string()),
            },
        }
    }

    fn unavailable(&self, at: &Address, field: &str, reason: String) -> Resolved {
        Resolved {
            controller: Controller::Unavailable { reason },
            evidence: vec![self.ev(at, field)],
        }
    }

    /// HCOL-4: classify the address that holds control.
    pub async fn classify(
        &self,
        address: &Address,
        depth: usize,
        seen: &mut BTreeSet<String>,
    ) -> Resolved {
        Box::pin(self.classify_inner(address, depth, seen)).await
    }

    async fn classify_inner(
        &self,
        address: &Address,
        depth: usize,
        seen: &mut BTreeSet<String>,
    ) -> Resolved {
        let key = address.as_str().to_ascii_lowercase();
        if !seen.insert(key.clone()) {
            return self.unavailable(address, "controller", format!("loop at {address}"));
        }
        if depth > MAX_DEPTH {
            return self.unavailable(
                address,
                "controller",
                format!("deeper than {MAX_DEPTH} levels"),
            );
        }
        let code = match self.evm.code(address.as_str()).await {
            Ok(c) => c,
            Err(e) => return self.unavailable(address, "code", e.to_string()),
        };
        if code.is_empty() {
            return Resolved {
                controller: Controller::SingleKey {
                    key: address.clone(),
                },
                evidence: vec![self.ev(address, "code (empty: a key)")],
            };
        }

        if let Some(r) = self.try_safe(address, depth, seen).await {
            return r;
        }
        if let Some(t) = self.known.timelocks.get(&key) {
            return self.timelock(address, t, depth, seen).await;
        }
        // F10: an unlisted TimelockController still publishes its delay; its
        // proposers are not enumerable, so they stay unreadable until the
        // registry lists candidates to confirm with `hasRole`.
        if let Ok(d) = self.word_call(address, "getMinDelay()", &[]).await
            && d.len() == 32
            && let Some(min_delay_secs) = u256_low(&d)
        {
            return Resolved {
                controller: Controller::Timelock {
                    address: address.clone(),
                    min_delay_secs,
                    proposer: Box::new(Controller::Unavailable {
                        reason: "timelock proposers are not enumerable and none is listed in the registry".into(),
                    }),
                    admin: None,
                },
                evidence: vec![self.ev(address, "getMinDelay()")],
            };
        }
        // An Ownable contract (ProxyAdmin): control passes to its owner.
        if let Ok(d) = self.word_call(address, "owner()", &[]).await
            && d.len() == 32
        {
            if is_zero(&d) {
                // Ownership renounced: nobody can call the owner functions.
                return Resolved {
                    controller: Controller::Immutable,
                    evidence: vec![self.ev(address, "owner() (zero: renounced)")],
                };
            }
            let mut r = self.classify(&word_address(&d), depth + 1, seen).await;
            r.evidence.insert(0, self.ev(address, "owner()"));
            return r;
        }
        Resolved {
            controller: Controller::Unrecognized {
                key: address.clone(),
                owner_program: None,
            },
            evidence: vec![self.ev(address, "code (not a known controller)")],
        }
    }

    async fn try_safe(
        &self,
        address: &Address,
        depth: usize,
        seen: &mut BTreeSet<String>,
    ) -> Option<Resolved> {
        let slot0 = self.evm.storage(address.as_str(), &[0u8; 32]).await.ok()?;
        let singleton = word_address(&slot0);
        if !self
            .known
            .safe_singletons
            .contains(&singleton.as_str().to_ascii_lowercase())
        {
            return None;
        }
        let threshold = self
            .word_call(address, "getThreshold()", &[])
            .await
            .ok()
            .and_then(|d| u256_low(&d));
        let owners = self
            .word_call(address, "getOwners()", &[])
            .await
            .ok()
            .and_then(|d| address_array(&d));
        let (Some(threshold), Some(mut owners)) = (threshold, owners) else {
            return Some(self.unavailable(
                address,
                "getThreshold(), getOwners()",
                "Safe read failed".into(),
            ));
        };
        owners.sort();
        let voters = owners.len() as u16;
        let threshold = u16::try_from(threshold).unwrap_or(u16::MAX);
        let mut evidence = vec![self.ev(address, "getThreshold(), getOwners()")];

        // HCOL-5: an enabled module can execute without signatures, so it
        // acts as a config authority.
        let modules = self
            .word_call(
                address,
                "getModulesPaginated(address,uint256)",
                &[pad_address(&Address::new(SENTINEL)), {
                    let mut w = [0u8; 32];
                    w[31] = 10;
                    w
                }],
            )
            .await;
        let config_authority = match modules {
            Ok(d) => match address_array(&d) {
                Some(m) if m.is_empty() => None,
                Some(mut m) => {
                    m.sort();
                    evidence.push(self.ev(address, "getModulesPaginated()"));
                    // One module is the usual case; several are each an
                    // authority, and the first sorted one is resolved.
                    let r = self.classify(&m[0], depth + 1, seen).await;
                    evidence.extend(r.evidence);
                    Some(Box::new(r.controller))
                }
                None => Some(Box::new(Controller::Unavailable {
                    reason: "Safe modules unreadable".into(),
                })),
            },
            Err(e) => Some(Box::new(Controller::Unavailable {
                reason: format!("Safe modules unreadable: {e}"),
            })),
        };

        let controller = if config_authority.is_none() && (voters == 0 || threshold > voters) {
            Controller::CannotAct {
                address: address.clone(),
                threshold,
                voters,
            }
        } else {
            Controller::Multisig {
                kind: MultisigKind::Safe,
                address: address.clone(),
                threshold,
                voters,
                time_lock_secs: 0,
                config_authority,
                signers: owners,
                nested_signers: Default::default(),
            }
        };
        Some(Resolved {
            controller,
            evidence,
        })
    }

    /// HCOL-6: delay, confirmed proposers and any outside admin.
    async fn timelock(
        &self,
        address: &Address,
        entry: &TimelockEntry,
        depth: usize,
        seen: &mut BTreeSet<String>,
    ) -> Resolved {
        let delay = match self.word_call(address, "getMinDelay()", &[]).await {
            Ok(d) => u256_low(&d),
            Err(_) => None,
        };
        let Some(min_delay_secs) = delay else {
            return self.unavailable(address, "getMinDelay()", "getMinDelay() failed".into());
        };
        let mut evidence = vec![self.ev(address, "getMinDelay()")];
        let proposer_role = keccak("PROPOSER_ROLE");

        // The weakest proposer is the one that matters; without a scorer
        // here, a single key beats a multisig and anything unreadable wins.
        let mut proposer: Option<Controller> = None;
        let mut proposers = entry.proposers.clone();
        proposers.sort();
        for p in &proposers {
            let confirmed = self
                .word_call(
                    address,
                    "hasRole(bytes32,address)",
                    &[proposer_role, pad_address(p)],
                )
                .await
                .ok()
                .and_then(|d| u256_low(&d));
            evidence.push(self.ev(address, &format!("hasRole(PROPOSER_ROLE, {p})")));
            let c = if confirmed == Some(1) {
                let r = self.classify(p, depth + 1, &mut seen.clone()).await;
                evidence.extend(r.evidence);
                r.controller
            } else {
                Controller::Unavailable {
                    reason: format!("{p} does not hold PROPOSER_ROLE"),
                }
            };
            proposer = Some(match proposer {
                None => c,
                Some(prev) => weaker(prev, c),
            });
        }
        let proposer = proposer.unwrap_or(Controller::Unavailable {
            reason: "no proposer in the registry".into(),
        });

        // An admin other than the timelock itself can rewrite roles and the
        // delay immediately (ADR 024 D4). Only registry-named candidates can
        // be checked, because roles are not enumerable.
        let mut admin = None;
        for p in &proposers {
            let holds = self
                .word_call(
                    address,
                    "hasRole(bytes32,address)",
                    &[[0u8; 32], pad_address(p)],
                )
                .await
                .ok()
                .and_then(|d| u256_low(&d));
            if holds == Some(1) {
                evidence.push(self.ev(address, &format!("hasRole(DEFAULT_ADMIN_ROLE, {p})")));
                let r = self.classify(p, depth + 1, &mut seen.clone()).await;
                evidence.extend(r.evidence);
                admin = Some(Box::new(r.controller));
                break;
            }
        }

        Resolved {
            controller: Controller::Timelock {
                address: address.clone(),
                min_delay_secs,
                proposer: Box::new(proposer),
                admin,
            },
            evidence,
        }
    }
}

/// Order used to keep the weakest of several proposers: unreadable, then a
/// single key or unknown contract, then a multisig with fewer voters.
fn weaker(a: Controller, b: Controller) -> Controller {
    fn rank(c: &Controller) -> (u8, u16) {
        match c {
            Controller::Unavailable { .. } | Controller::CannotAct { .. } => (0, 0),
            Controller::SingleKey { .. }
            | Controller::Unrecognized { .. }
            | Controller::Pda { .. } => (1, 0),
            Controller::Multisig { voters, .. } => (2, *voters),
            Controller::Timelock { .. } => (3, 0),
            Controller::Immutable => (4, 0),
            Controller::ProgramControlled { .. } => (0, 0),
        }
    }
    if rank(&b) < rank(&a) { b } else { a }
}
