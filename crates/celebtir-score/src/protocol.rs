//! Protocol-level scoring: per-program factors, then the weakest core program
//! decides each protocol factor (SCO-4).

use std::collections::BTreeMap;

use celebtir_core::snapshot::{
    Dependencies, FactsFile, ProgramFacts, ProgramRole, ProtocolFacts, TokenRole,
};
use celebtir_core::{Address, Evidence};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::factors;
use crate::methodology::Methodology;
use crate::rating::{Assessment, Rating, weakest};
use crate::sentences::{self, RiskSentence};
use crate::stage::{self, StageResult};
use crate::token::{TokenScore, score_token};
use crate::value::{
    ControlClass, ValueByControl, control_class_with_methodology, value_by_control_with_methodology,
};

/// Version of the `scores.json` format written by v0.1 and v0.2; v0.3 writes
/// [`Methodology::scores_schema_version`].
pub const SCORES_SCHEMA_VERSION: &str = "0.1.0";

/// The six factors, in PRD order.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Factor {
    UpgradeControl,
    UpgradeDelay,
    AdminPowers,
    CodeTransparency,
    OracleDependency,
    Dependencies,
}

impl Factor {
    pub const ALL: [Factor; 6] = [
        Factor::UpgradeControl,
        Factor::UpgradeDelay,
        Factor::AdminPowers,
        Factor::CodeTransparency,
        Factor::OracleDependency,
        Factor::Dependencies,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Factor::UpgradeControl => "Upgrade control",
            Factor::UpgradeDelay => "Upgrade delay",
            Factor::AdminPowers => "Admin powers",
            Factor::CodeTransparency => "Code transparency",
            Factor::OracleDependency => "Oracle dependency",
            Factor::Dependencies => "Dependencies",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FactorScore {
    pub rating: Rating,
    pub reason: String,
    /// What would raise the rating one level (SCO-5).
    pub next_step: String,
    pub evidence: Vec<Evidence>,
    /// Registry source pages behind a reviewed-none finding, which has no
    /// on-chain read to cite (v0.3).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub registry_sources: Vec<String>,
}

impl From<Assessment> for FactorScore {
    fn from(a: Assessment) -> Self {
        Self {
            rating: a.rating,
            reason: a.reason,
            next_step: a.next_step,
            evidence: a.evidence,
            registry_sources: a.sources,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProgramScore {
    pub role: ProgramRole,
    pub upgrade_control: FactorScore,
    pub upgrade_delay: FactorScore,
    pub code_transparency: FactorScore,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProtocolScore {
    pub stage: StageResult,
    /// The class of the weakest core program's controller (VAL-2).
    pub control_class: ControlClass,
    /// That controller in a few words, for tables.
    pub controller_summary: String,
    /// L2Beat-style risk sentences, most severe first (SCO-9).
    pub risks: Vec<RiskSentence>,
    pub factors: BTreeMap<Factor, FactorScore>,
    /// Every program, core and periphery, with its own ratings.
    pub programs: BTreeMap<Address, ProgramScore>,
    /// Tokens the protocol uses, with their worst rating (TOK-7).
    pub tokens: BTreeMap<Address, TokenExposure>,
    /// Each admin field's own rating, keyed like the facts' `admin` map; the
    /// Admin powers factor is their weakest link (UI-21).
    #[serde(default)]
    pub admin: BTreeMap<String, NodeRating>,
    /// Each sampled callee's upgrade-control rating (UI-12, UI-21).
    #[serde(default)]
    pub callees: BTreeMap<Address, NodeRating>,
}

/// One graph node's rating, for its colour and label (UI-21).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct NodeRating {
    pub rating: Rating,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TokenExposure {
    pub symbol: String,
    pub roles: Vec<TokenRole>,
    /// The token's weakest factor rating; Unknown if it was not scored.
    pub worst: Rating,
    /// Set when the protocol accepts a High-rated token as collateral.
    pub warning: Option<String>,
}

/// `data/snapshots/<date>/scores.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ScoresFile {
    pub schema_version: String,
    pub methodology_version: String,
    pub protocols: BTreeMap<String, ProtocolScore>,
    /// Keyed by mint address (SCO-10).
    pub tokens: BTreeMap<Address, TokenScore>,
    /// TVL by control class for the day (VAL-2).
    pub value_by_control: ValueByControl,
    /// HIP-3 DEXs, keyed by name: listed, never rated in v0.1 (HSCO-4).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub hip3_dexs: BTreeMap<String, UnratedEntry>,
    /// Protocols on chains the methodology does not rate (v0.3 coverage):
    /// listed with their reason, never given ratings, a Stage or a class.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub unassessed: BTreeMap<String, UnratedEntry>,
}

/// Something the methodology lists without rating (HSCO-4, ADR 024 D1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct UnratedEntry {
    pub rated: bool,
    pub reason: String,
}

/// The reason HIP-3 DEXs carry no ratings or Stage (ADR 024 D1, D2).
pub const HIP3_UNRATED: &str = "methodology v0.1 has no HIP-3 rules";

/// Scores every protocol in a facts file (SCO-1: no I/O, no clock).
pub fn score(facts: &FactsFile, m: &Methodology) -> ScoresFile {
    let lowered;
    let facts = if m.at_least(0, 3) {
        facts
    } else {
        lowered = crate::legacy::lower(facts);
        &lowered
    };
    let tokens: BTreeMap<Address, TokenScore> = facts
        .tokens
        .iter()
        .map(|(mint, t)| (mint.clone(), score_token(t, m)))
        .collect();
    let protocols = facts
        .protocols
        .iter()
        .filter(|(_, p)| m.assessed(p.chain))
        .map(|(id, p)| {
            let mut s = score_protocol(p, m);
            s.tokens = exposure(p, &tokens);
            (id.clone(), s)
        })
        .collect();
    let unassessed = facts
        .protocols
        .iter()
        .filter(|(_, p)| !m.assessed(p.chain))
        .map(|(id, p)| {
            (
                id.clone(),
                UnratedEntry {
                    rated: false,
                    reason: format!(
                        "methodology v{} does not yet rate {} protocols; facts are listed unrated",
                        m.version,
                        chain_name(p.chain)
                    ),
                },
            )
        })
        .collect();
    let mut assessed_facts = facts.clone();
    assessed_facts.protocols.retain(|_, p| m.assessed(p.chain));
    let hip3_reason = if m.at_least(0, 3) {
        format!("methodology v{} has no HIP-3 rules", m.version)
    } else {
        HIP3_UNRATED.to_string()
    };
    ScoresFile {
        schema_version: m.scores_schema_version().to_string(),
        methodology_version: m.version.clone(),
        protocols,
        tokens,
        value_by_control: value_by_control_with_methodology(&assessed_facts, m),
        unassessed,
        hip3_dexs: match &facts.hip3 {
            Some(celebtir_core::snapshot::Hip3Snapshot::Listed { dexs }) => dexs
                .keys()
                .map(|name| {
                    (
                        name.clone(),
                        UnratedEntry {
                            rated: false,
                            reason: hip3_reason.clone(),
                        },
                    )
                })
                .collect(),
            _ => BTreeMap::new(),
        },
    }
}

fn chain_name(chain: celebtir_core::Chain) -> &'static str {
    match chain {
        celebtir_core::Chain::Solana => "Solana",
        celebtir_core::Chain::HyperEvm => "HyperEVM",
        celebtir_core::Chain::HyperCore => "HyperCore",
    }
}

/// TOK-7: each token the protocol uses, its worst rating, and a warning when a
/// High-rated token is accepted as collateral.
fn exposure(
    p: &ProtocolFacts,
    tokens: &BTreeMap<Address, TokenScore>,
) -> BTreeMap<Address, TokenExposure> {
    p.tokens
        .iter()
        .map(|(mint, roles)| {
            let score = tokens.get(mint);
            let worst = score
                .and_then(|s| weakest(s.factors.values().map(|f| f.rating)))
                .unwrap_or(Rating::Unknown);
            let symbol = score.map_or_else(|| mint.to_string(), |s| s.symbol.clone());
            let warning = match score {
                Some(s) if worst == Rating::High && roles.contains(&TokenRole::Collateral) => {
                    let why: Vec<String> = s
                        .factors
                        .iter()
                        .filter(|(_, f)| f.rating == Rating::High)
                        .map(|(k, f)| format!("{}: {}", k.title(), f.reason))
                        .collect();
                    Some(format!(
                        "accepts {symbol} as collateral, which is rated High ({})",
                        why.join("; ")
                    ))
                }
                _ => None,
            };
            (
                mint.clone(),
                TokenExposure {
                    symbol,
                    roles: roles.clone(),
                    worst,
                    warning,
                },
            )
        })
        .collect()
}

fn score_program(p: &ProgramFacts, m: &Methodology) -> ProgramScore {
    ProgramScore {
        role: p.role,
        upgrade_control: factors::upgrade_control(&p.upgrade_control, m).into(),
        upgrade_delay: factors::upgrade_delay(&p.upgrade_control, m).into(),
        code_transparency: factors::code_transparency(&p.code_transparency).into(),
    }
}

pub fn score_protocol(p: &ProtocolFacts, m: &Methodology) -> ProtocolScore {
    let lowered;
    let p = if m.at_least(0, 3) {
        p
    } else {
        lowered = crate::legacy::lower_protocol(p);
        &lowered
    };
    let programs: BTreeMap<Address, ProgramScore> = p
        .programs
        .iter()
        .map(|(a, f)| (a.clone(), score_program(f, m)))
        .collect();

    let mut factors = BTreeMap::new();
    factors.insert(
        Factor::UpgradeControl,
        weakest_core(&programs, |s| &s.upgrade_control),
    );
    factors.insert(
        Factor::UpgradeDelay,
        weakest_core(&programs, |s| &s.upgrade_delay),
    );
    factors.insert(
        Factor::CodeTransparency,
        weakest_core(&programs, |s| &s.code_transparency),
    );
    let v03 = m.at_least(0, 3);
    let (admin_review, oracle_review) = if v03 {
        (p.admin_review.as_ref(), p.oracle_review.as_ref())
    } else {
        (None, None)
    };
    factors.insert(
        Factor::AdminPowers,
        factors::admin_powers_reviewed(&p.admin, admin_review, m).into(),
    );
    factors.insert(
        Factor::OracleDependency,
        factors::oracle_dependency_reviewed(&p.oracles, oracle_review, m).into(),
    );
    factors.insert(
        Factor::Dependencies,
        factors::dependencies(&p.dependencies, m).into(),
    );

    if p.oracles.is_empty()
        && p.category
            .as_ref()
            .is_some_and(|c| m.oracle_dependency.required_for_categories.contains(c))
    {
        factors.insert(
            Factor::OracleDependency,
            FactorScore {
                rating: Rating::Unknown,
                reason: if v03 {
                    "price feeds not yet listed in the registry; lending and perpetuals always depend on prices".into()
                } else {
                    "price feeds not yet listed in the registry".into()
                },
                next_step: "verify and register the protocol's price feeds".into(),
                evidence: vec![],
                registry_sources: vec![],
            },
        );
    }

    // v0.1/v0.2: "no admin keys" and "no price feeds" cited the core
    // programs' reads (SCO-5). v0.3 never borrows unrelated reads (F3): a
    // reviewed-none finding cites its registry source, a gap cites nothing.
    let program_evidence: Vec<Evidence> = p
        .programs
        .values()
        .filter(|f| f.role == ProgramRole::Core)
        .flat_map(|f| f.upgrade_control.evidence().iter().cloned())
        .collect();
    for (factor, score) in factors.iter_mut() {
        let own_evidence_only =
            v03 && matches!(factor, Factor::AdminPowers | Factor::OracleDependency);
        if score.evidence.is_empty() && !own_evidence_only {
            score.evidence = program_evidence.clone();
        }
    }

    let stage = stage::decide(p, &factors, m);
    let weakest = p
        .programs
        .values()
        .filter(|f| f.role == ProgramRole::Core)
        .map(|f| f.upgrade_control.value())
        .min_by_key(|c| control_class_with_methodology(c, m));
    let control_class = weakest.map_or(ControlClass::Unrecognized, |c| {
        control_class_with_methodology(c, m)
    });
    let controller_summary =
        weakest.map_or_else(|| "No core programs".to_string(), factors::short_describe);
    let risks = sentences::risks(p, &factors, m);
    ProtocolScore {
        stage,
        control_class,
        controller_summary,
        risks,
        factors,
        programs,
        tokens: BTreeMap::new(),
        admin: p
            .admin
            .iter()
            .map(|(k, a)| {
                let (rating, reason) = factors::rate_admin(a, m);
                (k.clone(), NodeRating { rating, reason })
            })
            .collect(),
        callees: match p.dependencies.value() {
            Dependencies::Sampled { callees, .. } => callees
                .iter()
                .map(|(a, d)| {
                    let c = d.upgrade_control.value();
                    let rating = factors::rate_controller(c, m);
                    (
                        a.clone(),
                        NodeRating {
                            rating,
                            reason: factors::describe_in(c, m),
                        },
                    )
                })
                .collect(),
            Dependencies::Unavailable { .. } => BTreeMap::new(),
        },
    }
}

/// The rating of the weakest core program; its reason names the program(s).
fn weakest_core(
    programs: &BTreeMap<Address, ProgramScore>,
    pick: impl Fn(&ProgramScore) -> &FactorScore,
) -> FactorScore {
    let core: Vec<(&Address, &FactorScore)> = programs
        .iter()
        .filter(|(_, s)| s.role == ProgramRole::Core)
        .map(|(a, s)| (a, pick(s)))
        .collect();
    let Some(rating) = weakest(core.iter().map(|(_, s)| s.rating)) else {
        return FactorScore {
            rating: Rating::Unknown,
            reason: "no core programs in the facts".into(),
            next_step: "list the protocol's core programs in the registry".into(),
            evidence: Vec::new(),
            registry_sources: Vec::new(),
        };
    };
    let worst: Vec<&(&Address, &FactorScore)> =
        core.iter().filter(|(_, s)| s.rating == rating).collect();
    let reason = if core.len() == 1 {
        worst[0].1.reason.clone()
    } else {
        worst
            .iter()
            .map(|(a, s)| format!("{}: {}", a, s.reason))
            .collect::<Vec<_>>()
            .join("; ")
    };
    FactorScore {
        rating,
        reason,
        next_step: worst[0].1.next_step.clone(),
        evidence: worst
            .iter()
            .flat_map(|(_, s)| s.evidence.iter().cloned())
            .collect(),
        registry_sources: Vec::new(),
    }
}
