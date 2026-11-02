//! Risk sentences in L2Beat's style (SCO-9), generated from facts. Every
//! template is listed in `TEMPLATES` and checked for loaded words (LCH-3).

use std::collections::BTreeMap;

use celebtir_core::snapshot::{
    AdminPower, CodeTransparency, Dependencies, OracleFact, ProgramRole, ProtocolFacts,
};
use celebtir_core::{Address, Controller};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub use crate::lint::banned_word;

use crate::factors::{
    duration, effective_delay, established, rate_admin, rate_controller, rate_delay, rate_market,
    source_name,
};
use crate::methodology::Methodology;
use crate::protocol::{Factor, FactorScore};
use crate::rating::Rating;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum RiskCategory {
    Stolen,
    Frozen,
    PriceManipulation,
}

/// Follows the rating that produced the sentence. Low produces none.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    High,
    Medium,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RiskSentence {
    pub severity: Severity,
    pub category: RiskCategory,
    pub factor: Factor,
    pub text: String,
}

const UPGRADE: &str = "Funds can be stolen if a core program is upgraded to new code by {holder}.";
const DELAY: &str = "Funds can be stolen if a harmful upgrade is approved, because it takes effect {delay}, before users can withdraw.";
const ADMIN_FUNDS: &str = "Funds can be stolen if they are moved by {holder} using `{field}`.";
const ADMIN_PAUSE: &str =
    "Funds can be frozen if the protocol is paused by {holder} using `{field}`.";
const ADMIN_PARAMS: &str =
    "Funds can be stolen if protocol parameters are changed by {holder} using `{field}`.";
const NO_SOURCE: &str = "Funds can be stolen if the deployed code of {program} does something its users cannot review, because its source is not public.";
const UNVERIFIED: &str = "Funds can be stolen if the deployed code of {program} differs from its public source, because the build is not verified.";
const ORACLE: &str = "Prices can be manipulated if {source} reports a false price for {market}.";
const DEPENDENCY: &str = "Funds can be stolen if {callee}, a program this protocol calls with user funds, is upgraded by {holder}.";

// Token page risk summary (TOK-6).
pub(crate) const TOKEN_MINT: &str = "New supply can be minted if {holder} uses the mint authority.";
pub(crate) const TOKEN_FREEZE: &str =
    "Your balance can be frozen if {holder} uses the freeze authority.";
pub(crate) const TOKEN_DELEGATE: &str =
    "Tokens can be moved from your wallet if {holder} uses the permanent delegate.";
pub(crate) const TOKEN_PAUSE: &str = "Transfers can be stopped if {holder} pauses the token.";
pub(crate) const TOKEN_HOOK: &str =
    "Transfers can be stopped if the transfer hook, controlled by {holder}, rejects them.";
pub(crate) const TOKEN_BRIDGE: &str =
    "The token can lose its backing if the {bridge} bridge stops holding {backing} on {chain}.";

/// Every template, for the LCH-3 lint.
pub const TEMPLATES: &[&str] = &[
    UPGRADE,
    DELAY,
    ADMIN_FUNDS,
    ADMIN_PAUSE,
    ADMIN_PARAMS,
    NO_SOURCE,
    UNVERIFIED,
    ORACLE,
    DEPENDENCY,
    TOKEN_MINT,
    TOKEN_FREEZE,
    TOKEN_DELEGATE,
    TOKEN_PAUSE,
    TOKEN_HOOK,
    TOKEN_BRIDGE,
];

fn versioned_template<'a>(legacy: &'a str, m: &Methodology) -> &'a str {
    if !m.at_least(0, 2) {
        return legacy;
    }
    match legacy {
        UPGRADE => "Core program code can be replaced by {holder}.",
        DELAY => {
            "An approved upgrade takes effect {delay}. Withdrawal completion within that period is not established by these configuration facts."
        }
        ADMIN_FUNDS => "Funds can be moved by {holder} using `{field}`.",
        ADMIN_PARAMS => "Protocol parameters can be changed by {holder} using `{field}`.",
        NO_SOURCE => {
            "The source of deployed program {program} is not public, so users cannot review its behavior from published source."
        }
        UNVERIFIED => {
            "The deployed code of {program} has not been verified against its public source."
        }
        DEPENDENCY => {
            "Program {callee}, which this protocol calls with user funds, can be upgraded by {holder}."
        }
        _ => legacy,
    }
}

pub(crate) fn fill(template: &str, values: &[(&str, &str)]) -> String {
    values.iter().fold(template.to_string(), |t, (k, v)| {
        t.replace(&format!("{{{k}}}"), v)
    })
}

/// "the holder of key X", "2 of 4 voters of the Squads v4 multisig X", …
fn holder(c: &Controller, m: &Methodology) -> String {
    match c {
        Controller::SingleKey { key } => format!("the holder of key {key}"),
        Controller::ProgramControlled {
            program, upgrade, ..
        } => format!(
            "program {program}, whose upgrade controller is {}",
            holder(upgrade, m)
        ),
        Controller::Multisig {
            config_authority: Some(ca),
            ..
        } if rate_controller(ca, m) == Rating::High => {
            format!("{} (the multisig's config authority)", holder(ca, m))
        }
        Controller::Multisig {
            threshold,
            voters,
            address,
            ..
        } => format!("{threshold} of {voters} voters of multisig {address}"),
        Controller::Timelock { admin: Some(a), .. } if rate_controller(a, m) == Rating::High => {
            format!("{} (the timelock's admin)", holder(a, m))
        }
        Controller::Timelock { proposer, .. } => holder(proposer, m),
        Controller::Unrecognized { key, .. } => format!("the unrecognized controller {key}"),
        Controller::Pda { key, .. } => format!(
            "whoever controls program-derived address {key} (its controlling program is not resolved)"
        ),
        Controller::Immutable => "no one".into(),
        Controller::CannotAct { address, .. } => format!("multisig {address}"),
        Controller::Unavailable { .. } => "an unreadable controller".into(),
    }
}

pub(crate) fn severity(r: Rating) -> Option<Severity> {
    match r {
        Rating::High => Some(Severity::High),
        Rating::Medium => Some(Severity::Medium),
        _ => None,
    }
}

/// The sentences for every High or Medium factor, most severe first.
pub fn risks(
    p: &ProtocolFacts,
    factors: &BTreeMap<Factor, FactorScore>,
    m: &Methodology,
) -> Vec<RiskSentence> {
    let mut out: Vec<RiskSentence> = Vec::new();
    let mut push = |factor: Factor, rating: Rating, category: RiskCategory, text: String| {
        if let Some(severity) = severity(rating) {
            if !out.iter().any(|r| r.text == text) {
                out.push(RiskSentence {
                    severity,
                    category,
                    factor,
                    text,
                });
            }
        }
    };
    let rating = |f: Factor| factors[&f].rating;
    let core: Vec<(&Address, &celebtir_core::snapshot::ProgramFacts)> = p
        .programs
        .iter()
        .filter(|(_, f)| f.role == ProgramRole::Core)
        .collect();

    let r = rating(Factor::UpgradeControl);
    for (_, f) in &core {
        let c = f.upgrade_control.value();
        if rate_controller(c, m) == r {
            push(
                Factor::UpgradeControl,
                r,
                RiskCategory::Stolen,
                fill(versioned_template(UPGRADE, m), &[("holder", &holder(c, m))]),
            );
        }
    }

    let r = rating(Factor::UpgradeDelay);
    for (_, f) in &core {
        if let Some(d) = effective_delay(f.upgrade_control.value()) {
            if rate_delay(d, m) == r {
                let delay = if d == 0 {
                    "immediately".to_string()
                } else {
                    format!("after {}", duration(d))
                };
                push(
                    Factor::UpgradeDelay,
                    r,
                    RiskCategory::Stolen,
                    fill(versioned_template(DELAY, m), &[("delay", &delay)]),
                );
            }
        }
    }

    let r = rating(Factor::AdminPowers);
    for a in p.admin.values() {
        if rate_admin(a, m).0 != r {
            continue;
        }
        let (template, category) = match a.power {
            AdminPower::Funds => (ADMIN_FUNDS, RiskCategory::Stolen),
            AdminPower::Pause => (ADMIN_PAUSE, RiskCategory::Frozen),
            AdminPower::Parameters => (ADMIN_PARAMS, RiskCategory::Stolen),
        };
        let h = holder(a.controller.value(), m);
        push(
            Factor::AdminPowers,
            r,
            category,
            fill(
                versioned_template(template, m),
                &[("holder", &h), ("field", &a.field)],
            ),
        );
    }

    let r = rating(Factor::CodeTransparency);
    for (addr, f) in &core {
        let template = match (f.code_transparency.value(), r) {
            (CodeTransparency::NoPublicSource, Rating::High) => NO_SOURCE,
            (CodeTransparency::PublicSourceUnverified { .. }, Rating::Medium) => UNVERIFIED,
            _ => continue,
        };
        push(
            Factor::CodeTransparency,
            r,
            RiskCategory::Stolen,
            fill(
                versioned_template(template, m),
                &[("program", addr.as_str())],
            ),
        );
    }

    let r = rating(Factor::OracleDependency);
    let mut markets: BTreeMap<&str, Vec<&OracleFact>> = BTreeMap::new();
    for o in p.oracles.values() {
        markets.entry(o.market.as_str()).or_default().push(o);
    }
    for (market, feeds) in &markets {
        let (mr, _) = rate_market(market, feeds, m);
        if mr != r {
            continue;
        }
        let primary = feeds
            .iter()
            .find(|f| f.role == celebtir_core::snapshot::OracleRole::Primary)
            .unwrap_or(&feeds[0]);
        let weak = feeds
            .iter()
            .find(|f| established(f.source.value(), m).is_none())
            .unwrap_or(primary);
        let source = match r {
            Rating::High => source_name(weak.source.value()),
            _ => format!(
                "its only oracle network ({})",
                source_name(primary.source.value())
            ),
        };
        push(
            Factor::OracleDependency,
            r,
            RiskCategory::PriceManipulation,
            fill(ORACLE, &[("source", &source), ("market", market)]),
        );
    }

    let r = rating(Factor::Dependencies);
    if let Dependencies::Sampled { callees, .. } = p.dependencies.value() {
        for (callee, dep) in callees {
            let c = dep.upgrade_control.value();
            if rate_controller(c, m) == r {
                push(
                    Factor::Dependencies,
                    r,
                    RiskCategory::Stolen,
                    fill(
                        versioned_template(DEPENDENCY, m),
                        &[("holder", &holder(c, m)), ("callee", callee.as_str())],
                    ),
                );
            }
        }
    }

    out.sort_by_key(|r| (r.severity, r.factor));
    out
}
