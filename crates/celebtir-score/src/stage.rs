//! The Stage decision (SCO-6): every requirement is checked and reported, and
//! the Stage is the highest level whose requirements, and all lower ones,
//! hold. The unmet requirements of the next level are what is "missing".

use std::collections::BTreeMap;

use celebtir_core::snapshot::{AdminPower, CodeTransparency, ProgramRole, ProtocolFacts};
use celebtir_core::{Address, Controller};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::factors::{duration, effective_delay};
use crate::methodology::{Methodology, meets_ratio};
use crate::protocol::{Factor, FactorScore};
use crate::rating::Rating;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
pub enum Stage {
    #[serde(rename = "unrated")]
    Unrated,
    #[serde(rename = "stage_0")]
    Stage0,
    #[serde(rename = "stage_1")]
    Stage1,
    #[serde(rename = "stage_2")]
    Stage2,
}

impl Stage {
    pub fn label(self) -> &'static str {
        match self {
            Stage::Unrated => "Unrated",
            Stage::Stage0 => "Stage 0",
            Stage::Stage1 => "Stage 1",
            Stage::Stage2 => "Stage 2",
        }
    }

    fn next(self) -> Option<Stage> {
        match self {
            Stage::Unrated => Some(Stage::Stage0),
            Stage::Stage0 => Some(Stage::Stage1),
            Stage::Stage1 => Some(Stage::Stage2),
            Stage::Stage2 => None,
        }
    }
}

/// One Stage requirement and whether this protocol meets it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Requirement {
    /// Stable ID, e.g. `s1.upgrade_delay`.
    pub id: String,
    /// The Stage this requirement unlocks.
    pub stage: Stage,
    pub description: String,
    pub met: bool,
    /// Why it is or is not met, naming programs and values.
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct StageResult {
    pub stage: Stage,
    /// The next Stage up, if any.
    pub next: Option<Stage>,
    /// Unmet requirements of `next`, as L2Beat's "items missing for Stage N".
    pub missing: Vec<Requirement>,
    /// Every requirement of every Stage, met or not.
    pub requirements: Vec<Requirement>,
}

fn req(id: &str, stage: Stage, description: &str, met: bool, detail: String) -> Requirement {
    Requirement {
        id: id.into(),
        stage,
        description: description.into(),
        met,
        detail,
    }
}

fn core_programs(p: &ProtocolFacts) -> Vec<(&Address, &celebtir_core::snapshot::ProgramFacts)> {
    p.programs
        .iter()
        .filter(|(_, f)| f.role == ProgramRole::Core)
        .collect()
}

/// Is any controller in this tree unreadable? A weak multisig whose config
/// authority cannot be read still rates High, but the protocol cannot be
/// fully assessed, so it is Unrated (SCO-7).
fn has_unreadable_part(c: &Controller) -> bool {
    match c {
        Controller::Unavailable { .. } | Controller::CannotAct { .. } | Controller::Pda { .. } => {
            true
        }

        Controller::Multisig {
            threshold,
            voters,
            config_authority,
            ..
        } => {
            *voters == 0
                || threshold > voters
                || config_authority.as_deref().is_some_and(has_unreadable_part)
        }
        Controller::Timelock {
            proposer, admin, ..
        } => has_unreadable_part(proposer) || admin.as_deref().is_some_and(has_unreadable_part),
        _ => false,
    }
}

/// Does any controller in this tree have a single key as config authority?
fn has_single_key_config_authority(c: &Controller) -> bool {
    match c {
        Controller::Multisig {
            config_authority: Some(ca),
            ..
        } => matches!(**ca, Controller::SingleKey { .. }) || has_single_key_config_authority(ca),
        Controller::Timelock {
            proposer, admin, ..
        } => {
            has_single_key_config_authority(proposer)
                || admin.as_deref().is_some_and(|a| {
                    matches!(a, Controller::SingleKey { .. }) || has_single_key_config_authority(a)
                })
        }
        _ => false,
    }
}

fn is_emergency_multisig(c: &Controller, m: &Methodology) -> bool {
    matches!(c, Controller::Multisig { threshold, voters, config_authority: None, .. }
        if *voters >= m.stage1.emergency_min_voters
            && meets_ratio(*threshold, *voters, m.stage1.emergency_min_threshold_pct))
}

fn rating_ok(factors: &BTreeMap<Factor, FactorScore>, f: Factor) -> (bool, String) {
    let s = &factors[&f];
    // Unknown is neutral here; it fails `s1.no_unknown` instead (SCO-7).
    (
        s.rating != Rating::High,
        format!("{}: {}", s.rating.label(), s.reason),
    )
}

pub fn decide(
    p: &ProtocolFacts,
    factors: &BTreeMap<Factor, FactorScore>,
    m: &Methodology,
) -> StageResult {
    let core = core_programs(p);
    let mut rs = Vec::new();

    // ── Out of Unrated ──────────────────────────────────────────────────────
    rs.push(req(
        "s0.core_programs",
        Stage::Stage0,
        "core programs identified in the registry",
        !core.is_empty(),
        format!("{} core programs", core.len()),
    ));
    let no_source: Vec<String> = core
        .iter()
        .filter(|(_, f)| {
            matches!(
                f.code_transparency.value(),
                CodeTransparency::NoPublicSource
            )
        })
        .map(|(a, _)| a.to_string())
        .collect();
    rs.push(req(
        "s0.public_source",
        Stage::Stage0,
        "public source for every core program",
        no_source.is_empty(),
        if no_source.is_empty() {
            "all core programs have public source".into()
        } else {
            format!("no public source: {}", no_source.join(", "))
        },
    ));
    let unknown_control: Vec<String> = core
        .iter()
        .filter(|(_, f)| has_unreadable_part(f.upgrade_control.value()))
        .map(|(a, _)| a.to_string())
        .collect();
    rs.push(req(
        "s0.upgrade_control_known",
        Stage::Stage0,
        "upgrade control readable for every core program",
        unknown_control.is_empty(),
        if unknown_control.is_empty() {
            "upgrade control read for all core programs".into()
        } else {
            format!("upgrade control unknown: {}", unknown_control.join(", "))
        },
    ));

    // ── Stage 1: Guarded ────────────────────────────────────────────────────
    let (ok, detail) = rating_ok(factors, Factor::UpgradeControl);
    rs.push(req(
        "s1.upgrade_control",
        Stage::Stage1,
        "upgrade control rated Low or Medium",
        ok,
        detail,
    ));

    let min = m.stage1.min_upgrade_delay_secs;
    let short_delay: Vec<String> = core
        .iter()
        .filter_map(|(a, f)| {
            let c = f.upgrade_control.value();
            match (c, effective_delay(c)) {
                (Controller::Immutable, _) | (_, None) => None,
                (_, Some(d)) if d >= min || is_emergency_multisig(c, m) => None,
                (_, Some(d)) => Some(format!("{a}: effective time lock {}", duration(d))),
            }
        })
        .collect();
    rs.push(req(
        "s1.upgrade_delay",
        Stage::Stage1,
        "upgrade delay of the Stage 1 minimum, unless upgrades need an emergency multisig",
        short_delay.is_empty(),
        if short_delay.is_empty() {
            format!(
                "every core program waits {}+ or needs an emergency multisig",
                duration(min)
            )
        } else {
            format!(
                "{}; needs {}+ or an emergency multisig of {}+ voters at {}%+",
                short_delay.join("; "),
                duration(min),
                m.stage1.emergency_min_voters,
                m.stage1.emergency_min_threshold_pct
            )
        },
    ));

    let (ok, detail) = rating_ok(factors, Factor::AdminPowers);
    rs.push(req(
        "s1.admin_powers",
        Stage::Stage1,
        "admin powers rated Low or Medium",
        ok,
        detail,
    ));

    let mut single_key_ca: Vec<String> = core
        .iter()
        .filter(|(_, f)| has_single_key_config_authority(f.upgrade_control.value()))
        .map(|(a, _)| format!("program {a}"))
        .collect();
    single_key_ca.extend(
        p.admin
            .iter()
            .filter(|(_, a)| has_single_key_config_authority(a.controller.value()))
            .map(|(k, _)| format!("admin {k}")),
    );
    rs.push(req(
        "s1.config_authority",
        Stage::Stage1,
        "no multisig has a single-key config authority",
        single_key_ca.is_empty(),
        if single_key_ca.is_empty() {
            "no single-key config authority".into()
        } else {
            format!(
                "single-key config authority on {}",
                single_key_ca.join(", ")
            )
        },
    ));

    // ADR 001: verification is required from Stage 1, not for Stage 0.
    let unverified: Vec<String> = core
        .iter()
        .filter(|(_, f)| {
            matches!(
                f.code_transparency.value(),
                CodeTransparency::PublicSourceUnverified { .. } | CodeTransparency::NoPublicSource
            )
        })
        .map(|(a, _)| a.to_string())
        .collect();
    rs.push(req(
        "s1.verified_builds",
        Stage::Stage1,
        "verified builds for all core programs",
        unverified.is_empty(),
        if unverified.is_empty() {
            "all core programs verified".into()
        } else {
            format!("not verified: {}", unverified.join(", "))
        },
    ));

    let unknown: Vec<&str> = Factor::ALL
        .iter()
        .filter(|f| factors[f].rating == Rating::Unknown)
        .map(|f| f.title())
        .collect();
    rs.push(req(
        "s1.no_unknown",
        Stage::Stage1,
        "every factor could be read (Unknown caps the Stage at 0)",
        unknown.is_empty(),
        if unknown.is_empty() {
            "no Unknown factors".into()
        } else {
            format!("Unknown: {}", unknown.join(", "))
        },
    ));

    // ── Stage 2: User-protected ─────────────────────────────────────────────
    let gov = m.stage2.governance_min_delay_secs;
    let upgradeable: Vec<String> = core
        .iter()
        .filter(|(_, f)| match f.upgrade_control.value() {
            Controller::Immutable => false,
            c @ Controller::Timelock { .. } => effective_delay(c).unwrap_or(0) < gov,
            _ => true,
        })
        .map(|(a, _)| a.to_string())
        .collect();
    rs.push(req(
        "s2.immutable_or_governance",
        Stage::Stage2,
        "core programs immutable, or upgradeable only through on-chain governance with a long delay",
        upgradeable.is_empty(),
        if upgradeable.is_empty() {
            "no core program can be upgraded quickly".into()
        } else {
            format!(
                "upgradeable without {}+ governance delay: {}",
                duration(gov),
                upgradeable.join(", ")
            )
        },
    ));

    // A pause can freeze withdrawals, so it counts as freezing funds (ADR 012).
    let fund_keys: Vec<(&String, AdminPower)> = p
        .admin
        .iter()
        .filter(|(_, a)| {
            matches!(a.power, AdminPower::Funds | AdminPower::Pause)
                && !matches!(a.controller.value(), Controller::Immutable)
        })
        .map(|(k, a)| (k, a.power))
        .collect();
    if m.at_least(0, 3) {
        // H9, D6: name the power; issuer freeze keys are rated on token pages.
        let list: Vec<String> = fund_keys
            .iter()
            .map(|(k, power)| match power {
                AdminPower::Pause => format!("{k} can pause (funds frozen in place)"),
                _ => format!("{k} can move funds"),
            })
            .collect();
        rs.push(req(
            "s2.no_fund_control",
            Stage::Stage2,
            "no protocol admin key can move user funds or pause the protocol (token-issuer freeze keys are rated on token pages, not here)",
            list.is_empty(),
            if list.is_empty() {
                "no protocol admin key can move funds or pause".into()
            } else {
                list.join("; ")
            },
        ));
    } else {
        let keys: Vec<String> = fund_keys.iter().map(|(k, _)| (*k).clone()).collect();
        rs.push(req(
            "s2.no_fund_control",
            Stage::Stage2,
            "no key can move or freeze user funds",
            keys.is_empty(),
            if keys.is_empty() {
                "no admin key can move or freeze funds".into()
            } else {
                format!("can move or freeze funds: {}", keys.join(", "))
            },
        ));
    }

    let all_met = |s: Stage| rs.iter().filter(|r| r.stage == s).all(|r| r.met);
    let stage = if !all_met(Stage::Stage0) {
        Stage::Unrated
    } else if !all_met(Stage::Stage1) {
        Stage::Stage0
    } else if !all_met(Stage::Stage2) {
        Stage::Stage1
    } else {
        Stage::Stage2
    };
    let next = stage.next();
    let missing = rs
        .iter()
        .filter(|r| Some(r.stage) == next && !r.met)
        .cloned()
        .collect();
    StageResult {
        stage,
        next,
        missing,
        requirements: rs,
    }
}
