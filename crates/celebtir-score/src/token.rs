//! Token scoring (TOK-6, SCO-10): four factors on the same engine, no Stage.
//! Rules are in ADR 013.

use std::collections::BTreeMap;

use celebtir_core::Controller;
use celebtir_core::snapshot::{TokenFacts, TokenProvenance};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::factors::{describe_in, rate_controller};
use crate::methodology::Methodology;
use crate::protocol::FactorScore;
use crate::rating::{Rating, weakest};
use crate::sentences::{
    Severity, TOKEN_BRIDGE, TOKEN_DELEGATE, TOKEN_FREEZE, TOKEN_HOOK, TOKEN_MINT, TOKEN_PAUSE,
    fill, severity,
};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum TokenFactor {
    SupplyControl,
    FreezeAndSeize,
    TransferRestrictions,
    Provenance,
}

impl TokenFactor {
    pub const ALL: [TokenFactor; 4] = [
        TokenFactor::SupplyControl,
        TokenFactor::FreezeAndSeize,
        TokenFactor::TransferRestrictions,
        TokenFactor::Provenance,
    ];

    pub fn title(self) -> &'static str {
        match self {
            TokenFactor::SupplyControl => "Supply control",
            TokenFactor::FreezeAndSeize => "Freeze and seize powers",
            TokenFactor::TransferRestrictions => "Transfer restrictions",
            TokenFactor::Provenance => "Provenance",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TokenScore {
    pub symbol: String,
    pub factors: BTreeMap<TokenFactor, FactorScore>,
    /// The token page's risk summary, most severe first (TOK-6).
    #[serde(default)]
    pub risks: Vec<TokenRisk>,
}

/// "Your balance can be frozen if…": one per power rated High or Medium.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TokenRisk {
    pub severity: Severity,
    pub factor: TokenFactor,
    pub text: String,
}

/// A power held by `c`: absent → Low, held by a multisig that meets the
/// Medium bar → Medium, a single key or weaker → High.
fn held(c: &Controller, m: &Methodology) -> Rating {
    match c {
        Controller::Immutable => Rating::Low,
        Controller::ProgramControlled { upgrade, .. } if m.at_least(0, 2) => {
            rate_controller(upgrade, m)
        }
        c => match rate_controller(c, m) {
            Rating::Low | Rating::Medium => Rating::Medium,
            r => r,
        },
    }
}

fn next_step(f: TokenFactor, r: Rating) -> String {
    match (f, r) {
        (_, Rating::Low) => "none: LOW is the lowest risk level".into(),
        (_, Rating::Unknown) => "rated once Celebtir can read the missing data".into(),
        (_, Rating::NotApplicable) => "not applicable: nothing to improve".into(),
        (TokenFactor::SupplyControl, Rating::High) => {
            "hold the mint authority in a multisig → MEDIUM".into()
        }
        (TokenFactor::SupplyControl, _) => "renounce the mint authority → LOW".into(),
        (TokenFactor::FreezeAndSeize, Rating::High) => {
            "remove any permanent delegate and hold freeze or pause powers in a multisig → MEDIUM"
                .into()
        }
        (TokenFactor::FreezeAndSeize, _) => {
            "renounce the freeze and pause authorities → LOW".into()
        }
        (TokenFactor::TransferRestrictions, Rating::High) => {
            "make the transfer hook program immutable or multisig-controlled → MEDIUM".into()
        }
        (TokenFactor::TransferRestrictions, _) => {
            "remove transfer hooks, fees and default-frozen accounts → LOW".into()
        }
        (TokenFactor::Provenance, _) => {
            "bridged tokens stay MEDIUM; see the bridge's own rating".into()
        }
    }
}

fn factor(f: TokenFactor, rating: Rating, reason: String, t: &TokenFacts) -> FactorScore {
    let evidence = match f {
        TokenFactor::SupplyControl => t.mint_authority.evidence(),
        TokenFactor::FreezeAndSeize => t.freeze_authority.evidence(),
        TokenFactor::TransferRestrictions => t.extensions.evidence(),
        TokenFactor::Provenance => t.provenance.evidence(),
    };
    FactorScore {
        rating,
        reason,
        next_step: next_step(f, rating),
        evidence: evidence.to_vec(),
        registry_sources: Vec::new(),
    }
}

pub fn score_token(t: &TokenFacts, m: &Methodology) -> TokenScore {
    let lowered;
    let t = if m.at_least(0, 3) {
        t
    } else {
        lowered = crate::legacy::lower_token(t);
        &lowered
    };
    let ext = t.extensions.value();
    let mut factors = BTreeMap::new();
    let mut risks: Vec<(Rating, TokenFactor, String)> = Vec::new();
    let mut risk = |r: Rating, f: TokenFactor, template: &str, holder: &Controller| {
        risks.push((r, f, fill(template, &[("holder", &describe_in(holder, m))])));
    };

    let mint = t.mint_authority.value();
    let (r, why) = match mint {
        Controller::Immutable => (
            Rating::Low,
            "mint authority renounced: supply is fixed".into(),
        ),
        c => {
            risk(held(c, m), TokenFactor::SupplyControl, TOKEN_MINT, c);
            (
                held(c, m),
                format!("new supply can be minted by {}", describe_in(c, m)),
            )
        }
    };
    factors.insert(
        TokenFactor::SupplyControl,
        factor(TokenFactor::SupplyControl, r, why, t),
    );

    let mut powers: Vec<(Rating, String)> = Vec::new();
    if let Some(d) = &ext.permanent_delegate {
        if *d != Controller::Immutable {
            risk(Rating::High, TokenFactor::FreezeAndSeize, TOKEN_DELEGATE, d);
            powers.push((
                Rating::High,
                format!(
                    "permanent delegate {} can move tokens out of any wallet",
                    describe_in(d, m)
                ),
            ));
        }
    }
    let freeze = t.freeze_authority.value();
    if *freeze != Controller::Immutable {
        risk(
            held(freeze, m),
            TokenFactor::FreezeAndSeize,
            TOKEN_FREEZE,
            freeze,
        );
        powers.push((
            held(freeze, m),
            format!("balances can be frozen by {}", describe_in(freeze, m)),
        ));
    }
    if let Some(p) = &ext.pause_authority {
        if *p != Controller::Immutable {
            risk(held(p, m), TokenFactor::FreezeAndSeize, TOKEN_PAUSE, p);
            powers.push((
                held(p, m),
                format!("transfers can be paused by {}", describe_in(p, m)),
            ));
        }
    }
    factors.insert(
        TokenFactor::FreezeAndSeize,
        summarize(
            TokenFactor::FreezeAndSeize,
            powers,
            "no freeze, pause or seize power",
            t,
        ),
    );

    let mut limits: Vec<(Rating, String)> = Vec::new();
    if !ext.unknown.is_empty() {
        let types: Vec<String> = ext.unknown.iter().map(u16::to_string).collect();
        limits.push((
            Rating::Unknown,
            format!("undecoded extension types {}", types.join(", ")),
        ));
    }
    if let Some(h) = &ext.transfer_hook {
        match (&h.program, &h.program_controller) {
            (Some(program), Some(pc)) => limits.push((
                {
                    let r = at_least(
                        weakest([held(&h.authority, m), held(pc, m)]).unwrap_or(Rating::Unknown),
                        Rating::Medium,
                    );
                    risk(r, TokenFactor::TransferRestrictions, TOKEN_HOOK, pc);
                    r
                },
                format!(
                    "transfer hook {program} can block transfers; hook program is {}",
                    describe_in(pc, m)
                ),
            )),
            // No hook program yet: only a restriction if someone can set one.
            _ if h.authority == Controller::Immutable => {}
            _ => limits.push((
                at_least(held(&h.authority, m), Rating::Medium),
                format!(
                    "a transfer hook that can block transfers can be set by {}",
                    describe_in(&h.authority, m)
                ),
            )),
        }
    }
    if let Some(a) = &ext.transfer_fee_authority {
        limits.push((
            at_least(held(a, m), Rating::Medium),
            format!("transfer fee can be changed by {}", describe_in(a, m)),
        ));
    }
    if ext.default_frozen {
        limits.push((
            Rating::Medium,
            "new token accounts start frozen (allowlist)".into(),
        ));
    }
    factors.insert(
        TokenFactor::TransferRestrictions,
        summarize(
            TokenFactor::TransferRestrictions,
            limits,
            "no transfer restrictions",
            t,
        ),
    );

    let (r, why) = match t.provenance.value() {
        TokenProvenance::Native => (Rating::Low, "native to Solana".to_string()),
        TokenProvenance::Bridged {
            bridge,
            backing_chain,
            backing_address,
        } => {
            risks.push((
                Rating::Medium,
                TokenFactor::Provenance,
                fill(
                    TOKEN_BRIDGE,
                    &[
                        ("bridge", bridge),
                        ("backing", backing_address),
                        ("chain", backing_chain),
                    ],
                ),
            ));
            (
                Rating::Medium,
                format!("bridged through {bridge}, backed by {backing_address} on {backing_chain}"),
            )
        }
    };
    factors.insert(
        TokenFactor::Provenance,
        factor(TokenFactor::Provenance, r, why, t),
    );

    let mut risks: Vec<TokenRisk> = risks
        .into_iter()
        .filter_map(|(r, factor, text)| {
            severity(r).map(|severity| TokenRisk {
                severity,
                factor,
                text,
            })
        })
        .collect();
    if m.at_least(0, 2) {
        for (factor, assessment) in &factors {
            if let Some(severity) = severity(assessment.rating) {
                if !risks.iter().any(|risk| risk.factor == *factor) {
                    risks.push(TokenRisk {
                        severity,
                        factor: *factor,
                        text: assessment.reason.clone(),
                    });
                }
            }
        }
    }
    // Stable: High first, then in the order the powers were read.
    risks.sort_by_key(|r| r.severity);
    TokenScore {
        symbol: t.symbol.clone(),
        factors,
        risks,
    }
}

/// Weakest of several findings; Low with `none` when there are no findings.
fn summarize(
    f: TokenFactor,
    findings: Vec<(Rating, String)>,
    none: &str,
    t: &TokenFacts,
) -> FactorScore {
    let Some(r) = weakest(findings.iter().map(|(r, _)| *r)) else {
        return factor(f, Rating::Low, none.into(), t);
    };
    let why = findings
        .iter()
        .filter(|(fr, _)| *fr == r)
        .map(|(_, w)| w.as_str())
        .collect::<Vec<_>>()
        .join("; ");
    factor(f, r, why, t)
}

/// Raises a known rating below `floor` to `floor`: a restriction that exists is
/// never Low. Unknown stays Unknown.
fn at_least(r: Rating, floor: Rating) -> Rating {
    match (r.level(), floor.level()) {
        (Some(a), Some(b)) if a < b => floor,
        _ => r,
    }
}
