//! The six factor rules (SCO-3), one function each. Each rates one fact or one
//! set of facts. Aggregation across programs (weakest link) happens in the
//! protocol scorer.

use std::collections::BTreeMap;

use celebtir_core::snapshot::{
    AdminFact, AdminPower, CodeTransparency, Dependencies, OracleFact, OracleRole, OracleSource,
    ReviewedNone,
};
use celebtir_core::{Address, Controller, Fact, MultisigKind};

use crate::methodology::{Methodology, meets_ratio};
use crate::rating::{Assessment, Rating, weakest};

// ── Controllers ─────────────────────────────────────────────────────────────

/// A short description of a controller for reasons and the CLI.
pub fn describe(c: &Controller) -> String {
    match c {
        Controller::Immutable => "immutable".into(),
        Controller::ProgramControlled {
            program, upgrade, ..
        } => format!(
            "controlled by program {}; upgrade control: {}",
            program,
            describe(upgrade)
        ),
        Controller::SingleKey { key } => format!("single key {}", short(key)),
        Controller::Multisig {
            kind,
            threshold,
            voters,
            time_lock_secs,
            config_authority,
            ..
        } => {
            let mut s = format!(
                "{} multisig, {threshold} of {voters} voters, time lock {}",
                multisig_name(*kind),
                duration(u64::from(*time_lock_secs))
            );
            if *voters == 0 || threshold > voters {
                s.push_str(" (cannot act: the threshold can never be reached)");
            }
            if let Some(ca) = config_authority {
                s.push_str(&format!("; config authority: {}", describe(ca)));
            }
            s
        }
        Controller::Timelock {
            min_delay_secs,
            proposer,
            admin,
            ..
        } => {
            let mut s = format!(
                "timelock of {} proposed by {}",
                duration(*min_delay_secs),
                describe(proposer)
            );
            if let Some(a) = admin {
                s.push_str(&format!("; admin: {}", describe(a)));
            }
            s
        }
        Controller::CannotAct {
            threshold, voters, ..
        } => format!("multisig that cannot act ({threshold} needed, {voters} voters)"),
        Controller::Unrecognized { key, owner_program } => match owner_program {
            Some(owner) => format!(
                "unrecognized controller {} owned by program {}",
                short(key),
                short(owner)
            ),
            None => format!("unrecognized controller {}", short(key)),
        },
        // Legacy wording; v0.1/v0.2 never see this variant (legacy.rs).
        Controller::Pda { key, .. } => format!("unrecognized controller {}", short(key)),
        Controller::Unavailable { reason } => format!("unavailable ({reason})"),
    }
}

/// The controller description a methodology publishes. v0.3 names full
/// addresses (the site shortens and links them), says "no time lock" for
/// Squads v3, and calls an unresolved off-curve address what it is.
pub fn describe_in(c: &Controller, m: &Methodology) -> String {
    if !m.at_least(0, 3) {
        return describe(c);
    }
    match c {
        Controller::Immutable => "immutable".into(),
        Controller::ProgramControlled {
            key,
            program,
            upgrade,
        } => format!(
            "program-derived address {key} of program {program}, whose upgrade control is {}",
            describe_in(upgrade, m)
        ),
        Controller::SingleKey { key } => format!("single key {key}"),
        Controller::Multisig {
            kind,
            address,
            threshold,
            voters,
            time_lock_secs,
            config_authority,
            ..
        } => {
            let lock = match (kind, *time_lock_secs) {
                (MultisigKind::SquadsV3, _) => "no time lock (Squads v3)".to_string(),
                (_, 0) => "no time lock".to_string(),
                (_, t) => format!("time lock {}", duration(u64::from(t))),
            };
            let mut s = format!(
                "{} multisig {address}, {threshold} of {voters} voters, {lock}",
                multisig_name(*kind)
            );
            if *voters == 0 || threshold > voters {
                s.push_str(" (cannot act: the threshold can never be reached)");
            }
            if let Some(ca) = config_authority {
                s.push_str(&format!("; config authority: {}", describe_in(ca, m)));
            }
            s
        }
        Controller::Timelock {
            address,
            min_delay_secs,
            proposer,
            admin,
        } => {
            let mut s = format!(
                "timelock {address} of {} proposed by {}",
                duration(*min_delay_secs),
                describe_in(proposer, m)
            );
            if let Some(a) = admin {
                s.push_str(&format!("; admin: {}", describe_in(a, m)));
            }
            s
        }
        Controller::CannotAct {
            address,
            threshold,
            voters,
        } => format!("multisig {address} that cannot act ({threshold} needed, {voters} voters)"),
        Controller::Unrecognized { key, owner_program } => match owner_program {
            Some(owner) => format!("unrecognized controller {key} owned by program {owner}"),
            None => format!("unrecognized controller {key}"),
        },
        Controller::Pda { key, .. } => format!(
            "program-derived address {key} (no private key exists; controlling program not resolved)"
        ),
        Controller::Unavailable { reason } => format!("unavailable ({reason})"),
    }
}

fn multisig_name(kind: MultisigKind) -> &'static str {
    match kind {
        MultisigKind::SquadsV4 => "Squads v4",
        MultisigKind::SquadsV3 => "Squads v3",
        MultisigKind::Safe => "Safe",
    }
}

fn short(a: &Address) -> String {
    let s = a.as_str();
    if s.len() > 12 {
        format!("{}…{}", &s[..4], &s[s.len() - 4..])
    } else {
        s.to_string()
    }
}

/// `432000` → `5 days`, `86400` → `24 h`, `3600` → `1 h`, `0` → `0 s`.
pub fn duration(secs: u64) -> String {
    match secs {
        s if s >= 2 * 86_400 && s % 86_400 == 0 => {
            let d = s / 86_400;
            format!("{d} day{}", if d == 1 { "" } else { "s" })
        }
        s if s >= 3_600 && s % 3_600 == 0 => format!("{} h", s / 3_600),
        s => format!("{s} s"),
    }
}

/// Upgrade-control rating of a controller on its own (used for programs,
/// admin keys and dependencies alike).
pub(crate) fn rate_controller(c: &Controller, m: &Methodology) -> Rating {
    let uc = &m.upgrade_control;
    match c {
        Controller::Immutable => Rating::Low,
        Controller::ProgramControlled { upgrade, .. } => {
            if !m.at_least(0, 2) {
                Rating::High
            } else {
                rate_controller(upgrade, m)
            }
        }
        Controller::SingleKey { .. } => Rating::High,
        Controller::Multisig {
            threshold,
            voters,
            config_authority,
            ..
        } => {
            let own = if *voters == 0 || threshold > voters {
                Rating::Unknown
            } else if *voters >= uc.low_min_voters
                && meets_ratio(*threshold, *voters, uc.low_min_threshold_pct)
            {
                Rating::Low
            } else if *voters >= uc.medium_min_voters
                && meets_ratio(*threshold, *voters, uc.medium_min_threshold_pct)
            {
                Rating::Medium
            } else {
                Rating::High
            };
            // The config authority can rewrite members and threshold, so the
            // multisig is only as strong as it (ADR 005).
            match config_authority {
                Some(ca) => weakest([own, rate_controller(ca, m)]).unwrap_or(Rating::Unknown),
                None => own,
            }
        }
        // The admin can grant itself the proposer role (ADR 024 D4).
        Controller::Timelock {
            proposer, admin, ..
        } => match admin {
            Some(a) => weakest([rate_controller(proposer, m), rate_controller(a, m)])
                .unwrap_or(Rating::Unknown),
            None => rate_controller(proposer, m),
        },
        Controller::CannotAct { .. } | Controller::Unavailable { .. } => Rating::Unknown,
        // ADR 002: unsupported controllers are not shown as safe.
        Controller::Unrecognized { .. } => Rating::High,
        // v0.3 (D1): no key exists, and who can sign for it is not known.
        Controller::Pda { .. } => Rating::Unknown,
    }
}

/// Seconds between approval and effect; None when it cannot be known.
/// `Immutable` has no delay because nothing can change.
pub(crate) fn effective_delay(c: &Controller) -> Option<u64> {
    match c {
        Controller::Immutable => None,
        Controller::ProgramControlled { upgrade, .. } => effective_delay(upgrade),
        Controller::SingleKey { .. } | Controller::Unrecognized { .. } => Some(0),
        Controller::Multisig {
            time_lock_secs,
            config_authority,
            ..
        } => {
            let own = u64::from(*time_lock_secs);
            match config_authority {
                // ADR 005: the config authority acts without the time lock.
                Some(ca) => effective_delay(ca).map(|d| d.min(own)),
                None => Some(own),
            }
        }
        Controller::Timelock {
            min_delay_secs,
            admin,
            ..
        } => match admin {
            // ADR 024 D4: the admin can set the delay to zero without waiting.
            Some(a) => effective_delay(a).map(|d| d.min(*min_delay_secs)),
            None => Some(*min_delay_secs),
        },
        Controller::CannotAct { .. } | Controller::Unavailable { .. } | Controller::Pda { .. } => {
            None
        }
    }
}

// ── Upgrade control ─────────────────────────────────────────────────────────

fn assess_upgrade_control(c: &Fact<Controller>, m: &Methodology) -> Assessment {
    let controller = c.value();
    let rating = rate_controller(controller, m);
    let mut reason = describe_in(controller, m);
    if let Controller::Multisig {
        config_authority: Some(ca),
        ..
    } = controller
    {
        if rate_controller(ca, m) == rating && rating == Rating::High {
            reason = format!("config authority is {}; {reason}", describe_in(ca, m));
        }
    }
    if let Controller::Unrecognized { .. } = controller {
        reason.push_str(" (Celebtir does not yet support this controller type)");
    }
    Assessment::new(rating, reason, c.evidence().to_vec())
}

// ── Upgrade delay ───────────────────────────────────────────────────────────

pub(crate) fn rate_delay(secs: u64, m: &Methodology) -> Rating {
    if secs >= m.upgrade_delay.low_min_secs {
        Rating::Low
    } else if secs >= m.upgrade_delay.medium_min_secs {
        Rating::Medium
    } else {
        Rating::High
    }
}

fn assess_upgrade_delay(c: &Fact<Controller>, m: &Methodology) -> Assessment {
    let controller = c.value();
    let evidence = c.evidence().to_vec();
    match controller {
        Controller::ProgramControlled { upgrade, .. } if m.at_least(0, 2) => assess_upgrade_delay(
            &Fact::new(*upgrade.clone(), evidence).expect("controller has evidence"),
            m,
        ),
        Controller::Immutable => {
            Assessment::new(Rating::Low, "immutable: the code cannot change", evidence)
        }
        Controller::CannotAct { .. } | Controller::Unavailable { .. } | Controller::Pda { .. } => {
            Assessment::new(
                Rating::Unknown,
                format!("delay unknown: {}", describe_in(controller, m)),
                evidence,
            )
        }
        _ => {
            let secs = effective_delay(controller).unwrap_or(0);
            let reason = match controller {
                Controller::Multisig {
                    time_lock_secs,
                    config_authority: Some(ca),
                    ..
                } if secs < u64::from(*time_lock_secs) => format!(
                    "time lock {} can be bypassed by the config authority ({}), so the effective delay is {}",
                    duration(u64::from(*time_lock_secs)),
                    describe_in(ca, m),
                    duration(secs)
                ),
                Controller::Multisig {
                    kind: MultisigKind::SquadsV3,
                    ..
                } if m.at_least(0, 3) => "no time lock (Squads v3 has no time-lock field)".into(),
                Controller::Multisig { .. } if m.at_least(0, 3) && secs == 0 => {
                    "no time lock".into()
                }
                Controller::SingleKey { .. } => "a single key upgrades with no delay".into(),
                Controller::Unrecognized { .. } => {
                    "no delay can be read for an unrecognized controller".into()
                }
                _ => format!("time lock {}", duration(secs)),
            };
            Assessment::new(rate_delay(secs, m), reason, evidence)
        }
    }
}

// ── Admin powers ────────────────────────────────────────────────────────────

fn power_name(p: AdminPower) -> &'static str {
    match p {
        AdminPower::Parameters => "change parameters",
        AdminPower::Pause => "pause",
        AdminPower::Funds => "move funds",
    }
}

pub(crate) fn rate_admin(a: &AdminFact, m: &Methodology) -> (Rating, String) {
    let c = a.controller.value();
    // v0.3 (H6): name the account, so fields of different accounts differ.
    let field = if m.at_least(0, 3) {
        format!("`{}` of {}", a.field, a.account)
    } else {
        format!("`{}`", a.field)
    };
    let what = format!("{field} can {}", power_name(a.power));
    match c {
        Controller::Immutable => (Rating::Low, format!("{field} holds no key")),
        _ => match rate_controller(c, m) {
            Rating::Unknown => (
                Rating::Unknown,
                format!("{what}; holder {}", describe_in(c, m)),
            ),
            Rating::High => (
                Rating::High,
                format!("{what}, held by {}", describe_in(c, m)),
            ),
            _ => {
                // ADR 003: the delay comes from the admin's controller; only
                // parameter changes count as bounded.
                let delay = effective_delay(c).unwrap_or(0);
                if a.power == AdminPower::Parameters
                    && delay >= m.admin_powers.low_min_time_lock_secs
                {
                    (
                        Rating::Low,
                        format!("{what} behind a {} time lock", duration(delay)),
                    )
                } else {
                    // ADR 011: any power held by a multisig meeting the
                    // Medium bar, including moving funds, is Medium.
                    (
                        Rating::Medium,
                        format!("{what}, held by {}", describe_in(c, m)),
                    )
                }
            }
        },
    }
}

fn assess_admin_powers(
    admin: &BTreeMap<String, AdminFact>,
    review: Option<&ReviewedNone>,
    m: &Methodology,
) -> Assessment {
    if admin.is_empty() && m.at_least(0, 3) {
        // F3: an empty list is a gap, not a finding, unless a sourced
        // review says so. Neither cites unrelated reads as evidence.
        return match review {
            Some(r) => Assessment::new(
                Rating::Low,
                format!(
                    "registry review on {} found no admin accounts: {}",
                    r.reviewed_on, r.note
                ),
                Vec::new(),
            )
            .with_sources(vec![r.source.clone()]),
            None => Assessment::new(
                Rating::Unknown,
                "admin accounts not yet read: the registry lists none and records no review that found none",
                Vec::new(),
            ),
        };
    }
    if admin.is_empty() {
        // protocol.rs cites the core programs' reads as the evidence.
        return Assessment::new(
            Rating::Low,
            "the registry lists no admin accounts; the evidence is the core programs' upgrade reads",
            Vec::new(),
        );
    }
    let rated: Vec<(Rating, String, &AdminFact)> = admin
        .values()
        .map(|a| {
            let (r, why) = rate_admin(a, m);
            (r, why, a)
        })
        .collect();
    let rating = weakest(rated.iter().map(|(r, ..)| *r)).unwrap_or(Rating::Unknown);
    let reason = rated
        .iter()
        .filter(|(r, ..)| *r == rating)
        .map(|(_, why, _)| why.as_str())
        .collect::<Vec<_>>()
        .join("; ");
    let evidence = rated
        .iter()
        .flat_map(|(.., a)| a.controller.evidence().iter().cloned())
        .collect();
    Assessment::new(rating, reason, evidence)
}

// ── Code transparency ───────────────────────────────────────────────────────

fn assess_code_transparency(t: &Fact<CodeTransparency>) -> Assessment {
    let (rating, reason) = match t.value() {
        CodeTransparency::Verified {
            repo,
            commit,
            warning,
            ..
        } => {
            let mut reason = format!("verified build of {repo} at {commit}");
            if let Some(w) = warning {
                reason.push_str(&format!(" (warning: {w})"));
            }
            (Rating::Low, reason)
        }
        CodeTransparency::PublicSourceUnverified { repo, .. } => (
            Rating::Medium,
            format!("source public at {repo}, but the deployed build is not verified"),
        ),
        CodeTransparency::NoPublicSource => (Rating::High, "no public source".into()),
        CodeTransparency::Unavailable { reason } => (
            Rating::Unknown,
            format!("verification status unavailable ({reason})"),
        ),
    };
    Assessment::new(rating, reason, t.evidence().to_vec())
}

// ── Oracle dependency ───────────────────────────────────────────────────────

pub(crate) fn established<'a>(source: &'a OracleSource, m: &Methodology) -> Option<&'a str> {
    match source {
        OracleSource::Network { name }
            if m.oracle_dependency
                .established_networks
                .iter()
                .any(|n| n == name) =>
        {
            Some(name)
        }
        _ => None,
    }
}

pub(crate) fn source_name(source: &OracleSource) -> String {
    match source {
        OracleSource::Network { name } => name.clone(),
        OracleSource::ProtocolRun => "a protocol-run feed".into(),
        OracleSource::Other { owner_program } => {
            format!("an unclassified feed owned by {}", short(owner_program))
        }
        OracleSource::Unavailable { reason } => format!("an unreadable feed ({reason})"),
    }
}

pub(crate) fn rate_market(
    market: &str,
    feeds: &[&OracleFact],
    m: &Methodology,
) -> (Rating, String) {
    if let Some(r) = guarded_primary(market, feeds, m) {
        return r;
    }
    if let Some(f) = feeds
        .iter()
        .find(|f| matches!(f.source.value(), OracleSource::Unavailable { .. }))
    {
        return (
            Rating::Unknown,
            format!("{market}: {}", source_name(f.source.value())),
        );
    }
    let has_primary = feeds.iter().any(|f| f.role == OracleRole::Primary);
    let (primaries, fallbacks): (Vec<&&OracleFact>, Vec<&&OracleFact>) = feeds
        .iter()
        .partition(|f| f.role == OracleRole::Primary || !has_primary);

    if let Some(f) = primaries
        .iter()
        .find(|f| established(f.source.value(), m).is_none())
    {
        let mut reason = format!(
            "{market}: prices come from {}",
            source_name(f.source.value())
        );
        if m.at_least(0, 3) {
            reason.push_str(&aggregated(f));
        }
        return (Rating::High, reason);
    }
    let primary = source_name(primaries[0].source.value());
    match fallbacks
        .iter()
        .find_map(|f| established(f.source.value(), m))
    {
        Some(fallback) => (
            Rating::Low,
            format!("{market}: {primary} with {fallback} as fallback"),
        ),
        None => (
            Rating::Medium,
            format!("{market}: single oracle network ({primary}), no fallback"),
        ),
    }
}

/// v0.3 (D3): a primary outside the established networks, bounded by
/// fallback feeds from enough distinct established networks, rates Low.
fn guarded_primary(
    market: &str,
    feeds: &[&OracleFact],
    m: &Methodology,
) -> Option<(Rating, String)> {
    if !m.at_least(0, 3) {
        return None;
    }
    let min = usize::from(m.oracle_dependency.guarded_primary_min_networks?);
    let primaries: Vec<&OracleFact> = feeds
        .iter()
        .copied()
        .filter(|f| f.role == OracleRole::Primary)
        .collect();
    let unestablished: Vec<&OracleFact> = primaries
        .iter()
        .copied()
        .filter(|f| established(f.source.value(), m).is_none())
        .collect();
    if unestablished.is_empty()
        || unestablished
            .iter()
            .any(|f| matches!(f.source.value(), OracleSource::Unavailable { .. }))
    {
        return None;
    }
    let guards: std::collections::BTreeSet<&str> = feeds
        .iter()
        .filter(|f| f.role == OracleRole::Fallback && f.bounds_primary)
        .filter_map(|f| established(f.source.value(), m))
        .collect();
    let mut primary: Vec<String> = unestablished
        .iter()
        .map(|f| match &f.provider {
            Some(p) => format!("{p} ({})", source_name(f.source.value())),
            None => source_name(f.source.value()),
        })
        .collect();
    primary.dedup();
    let declared: std::collections::BTreeSet<&str> = unestablished
        .iter()
        .flat_map(|f| f.guarded_by.iter().map(String::as_str))
        .collect();
    let missing: Vec<&str> = declared.difference(&guards).copied().collect();
    if !declared.is_empty() && !missing.is_empty() {
        // A documented guard that cannot be read is a gap, not a finding.
        return Some((
            Rating::Unknown,
            format!(
                "{market}: primary {} is documented as bounded by {}, but no registered on-chain feed shows the {} guard",
                primary.join(", "),
                declared.iter().copied().collect::<Vec<_>>().join(" and "),
                missing.join(" and ")
            ),
        ));
    }
    if guards.len() < min {
        return None;
    }
    let guards: Vec<&str> = guards.into_iter().collect();
    Some((
        Rating::Low,
        format!(
            "{market}: primary {} is not an established network, but its price must agree with {}, which are also the fallback",
            primary.join(", "),
            guards.join(" and ")
        ),
    ))
}

/// v0.3 (F4): why a protocol-run aggregator is protocol-run: the protocol
/// itself picks and combines the listed underlying sources.
fn aggregated(f: &OracleFact) -> String {
    use celebtir_core::snapshot::UnderlyingFeeds;
    let mut types: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    let mut markets = 0;
    for u in f.underlying.iter().chain(f.underlying_markets.values()) {
        if let UnderlyingFeeds::Listed { feeds } = u.value() {
            markets += 1;
            types.extend(feeds.iter().map(|x| x.source_type.as_str()));
        }
    }
    if types.is_empty() {
        return String::new();
    }
    format!(
        ", which the protocol operates to select and combine {} across {markets} market{}",
        types.into_iter().collect::<Vec<_>>().join(", "),
        if markets == 1 { "" } else { "s" }
    )
}

fn assess_oracle_dependency(
    oracles: &BTreeMap<Address, OracleFact>,
    review: Option<&ReviewedNone>,
    m: &Methodology,
) -> Assessment {
    if oracles.is_empty() && m.at_least(0, 3) {
        return match review {
            Some(r) => Assessment::new(
                Rating::NotApplicable,
                format!(
                    "registry review on {} found no price feeds: {}",
                    r.reviewed_on, r.note
                ),
                Vec::new(),
            )
            .with_sources(vec![r.source.clone()]),
            None => Assessment::new(
                Rating::Unknown,
                "price feeds not yet listed: the registry records none and no review that found none",
                Vec::new(),
            ),
        };
    }
    if oracles.is_empty() {
        return Assessment::new(
            Rating::NotApplicable,
            "the registry lists no price feeds; the evidence is the core programs' upgrade reads",
            Vec::new(),
        );
    }
    let mut markets: BTreeMap<&str, Vec<&OracleFact>> = BTreeMap::new();
    for o in oracles.values() {
        markets.entry(o.market.as_str()).or_default().push(o);
    }
    let rated: Vec<(Rating, String)> = markets
        .iter()
        .map(|(market, feeds)| rate_market(market, feeds, m))
        .collect();
    let rating = weakest(rated.iter().map(|(r, _)| *r)).unwrap_or(Rating::Unknown);
    let reason = rated
        .iter()
        .filter(|(r, _)| *r == rating)
        .map(|(_, why)| why.as_str())
        .collect::<Vec<_>>()
        .join("; ");
    let evidence = oracles
        .values()
        .flat_map(|o| o.source.evidence().iter().cloned())
        .collect();
    Assessment::new(rating, reason, evidence)
}

// ── Dependencies ────────────────────────────────────────────────────────────

fn assess_dependencies(d: &Fact<Dependencies>, m: &Methodology) -> Assessment {
    let mut evidence = d.evidence().to_vec();
    match d.value() {
        Dependencies::Unavailable { reason } => Assessment::new(
            Rating::Unknown,
            format!("dependencies unavailable ({reason})"),
            evidence,
        ),
        Dependencies::Sampled { callees, .. } if callees.is_empty() => Assessment::new(
            Rating::Low,
            "calls no other programs with user funds",
            evidence,
        ),
        Dependencies::Sampled { callees, .. } => {
            let rated: Vec<(Rating, &Address, &Controller)> = callees
                .iter()
                .map(|(addr, dep)| {
                    let c = dep.upgrade_control.value();
                    (rate_controller(c, m), addr, c)
                })
                .collect();
            let rating = weakest(rated.iter().map(|(r, ..)| *r)).unwrap_or(Rating::Unknown);
            let reason = if rating == Rating::Low {
                match (callees.len(), m.at_least(0, 3)) {
                    (1, true) => "the 1 callee is rated Low on upgrade control".to_string(),
                    (n, _) => format!("all {n} callees rated Low on upgrade control"),
                }
            } else {
                rated
                    .iter()
                    .filter(|(r, ..)| *r == rating)
                    .map(|(_, a, c)| {
                        if m.at_least(0, 3) {
                            format!("callee {a} is {}", describe_in(c, m))
                        } else {
                            format!("callee {} is {}", short(a), describe(c))
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("; ")
            };
            evidence.extend(
                callees
                    .values()
                    .flat_map(|dep| dep.upgrade_control.evidence().iter().cloned()),
            );
            Assessment::new(rating, reason, evidence)
        }
    }
}

// ── Public factor functions: rating, reason, evidence and next step (SCO-5) ─

const LOWEST: &str = "none: LOW is the lowest risk level";

fn generic_next(rating: Rating) -> Option<String> {
    match rating {
        Rating::Low => Some(LOWEST.into()),
        Rating::Unknown => Some(
            "rated once Celebtir can read the missing data; until then the Stage is capped at 0"
                .into(),
        ),
        Rating::NotApplicable => Some("not applicable: nothing to improve".into()),
        Rating::Medium | Rating::High => None,
    }
}

fn multisig_bar(voters: u16, pct: u16) -> String {
    format!("a multisig of {voters}+ voters at a {pct}%+ threshold")
}

fn single_key_config_authority(c: &Controller, m: &Methodology) -> bool {
    matches!(c, Controller::Multisig { config_authority: Some(ca), .. } if rate_controller(ca, m) == Rating::High)
}

pub fn upgrade_control(c: &Fact<Controller>, m: &Methodology) -> Assessment {
    let a = assess_upgrade_control(c, m);
    let uc = &m.upgrade_control;
    let low = multisig_bar(uc.low_min_voters, uc.low_min_threshold_pct);
    let medium = multisig_bar(uc.medium_min_voters, uc.medium_min_threshold_pct);
    let next = generic_next(a.rating).unwrap_or_else(|| match (a.rating, c.value()) {
        (Rating::High, ctrl) if single_key_config_authority(ctrl, m) => {
            "remove the config authority or hand it to a multisig, so the multisig's own threshold counts".into()
        }
        (Rating::High, Controller::Unrecognized { .. }) => format!(
            "Celebtir support for this controller type, or {medium} → MEDIUM"
        ),
        (Rating::High, _) => format!("{medium} → MEDIUM; {low} → LOW"),
        _ => format!("{low} → LOW, or make the program immutable"),
    });
    a.with_next(next)
}

pub fn upgrade_delay(c: &Fact<Controller>, m: &Methodology) -> Assessment {
    let a = assess_upgrade_delay(c, m);
    let medium = duration(m.upgrade_delay.medium_min_secs);
    let low = duration(m.upgrade_delay.low_min_secs);
    let bypassed = matches!(c.value(), Controller::Multisig { time_lock_secs, config_authority: Some(ca), .. }
        if effective_delay(ca).is_some_and(|d| d < u64::from(*time_lock_secs)));
    let next = generic_next(a.rating).unwrap_or_else(|| match a.rating {
        Rating::High if bypassed => {
            "remove the config authority, so upgrades must wait for the time lock".into()
        }
        Rating::High => format!("a time lock of {medium}+ → MEDIUM; {low}+ → LOW"),
        _ => format!("a time lock of {low}+ → LOW"),
    });
    a.with_next(next)
}

pub fn admin_powers(admin: &BTreeMap<String, AdminFact>, m: &Methodology) -> Assessment {
    admin_powers_reviewed(admin, None, m)
}

/// Admin powers with the registry's reviewed-none statement, if any (v0.3).
pub fn admin_powers_reviewed(
    admin: &BTreeMap<String, AdminFact>,
    review: Option<&ReviewedNone>,
    m: &Methodology,
) -> Assessment {
    let a = assess_admin_powers(admin, review, m);
    if a.rating == Rating::Unknown && admin.is_empty() {
        return a.with_next(
            "read the protocol's admin accounts, or record a sourced review that it has none",
        );
    }
    let uc = &m.upgrade_control;
    let next = generic_next(a.rating).unwrap_or_else(|| match a.rating {
        Rating::High => format!(
            "hold every admin key in {} → MEDIUM",
            multisig_bar(uc.medium_min_voters, uc.medium_min_threshold_pct)
        ),
        _ => format!(
            "limit admin keys to parameters behind a {}+ time lock, or renounce them → LOW",
            duration(m.admin_powers.low_min_time_lock_secs)
        ),
    });
    a.with_next(next)
}

pub fn code_transparency(t: &Fact<CodeTransparency>) -> Assessment {
    let a = assess_code_transparency(t);
    let next = generic_next(a.rating).unwrap_or_else(|| match a.rating {
        Rating::High => "publish the program's source → MEDIUM".into(),
        _ => "verify the deployed build against public source (OtterSec) → LOW".into(),
    });
    a.with_next(next)
}

pub fn oracle_dependency(oracles: &BTreeMap<Address, OracleFact>, m: &Methodology) -> Assessment {
    oracle_dependency_reviewed(oracles, None, m)
}

/// Oracle dependency with the registry's reviewed-none statement (v0.3).
pub fn oracle_dependency_reviewed(
    oracles: &BTreeMap<Address, OracleFact>,
    review: Option<&ReviewedNone>,
    m: &Methodology,
) -> Assessment {
    let a = assess_oracle_dependency(oracles, review, m);
    if a.rating == Rating::Unknown && oracles.is_empty() {
        return a.with_next(
            "verify and register the protocol's price feeds, or record a sourced review that it uses none",
        );
    }
    let next = generic_next(a.rating).unwrap_or_else(|| match a.rating {
        Rating::High => format!(
            "take prices from an established oracle network ({}) → MEDIUM",
            m.oracle_dependency.established_networks.join(", ")
        ),
        _ => "add an established fallback feed for every market → LOW".into(),
    });
    a.with_next(next)
}

pub fn dependencies(d: &Fact<Dependencies>, m: &Methodology) -> Assessment {
    let mut a = assess_dependencies(d, m);
    if let Dependencies::Sampled { stale: true, .. } = d.value() {
        a.reason.push_str(
            " (list carried over from an earlier sample: no transactions in the latest window)",
        );
    }
    if let Dependencies::Sampled { declared: true, .. } = d.value() {
        a.reason
            .push_str(" (declared in the registry, not traced from transactions)");
    }
    let uc = &m.upgrade_control;
    let next = generic_next(a.rating).unwrap_or_else(|| match a.rating {
        Rating::High => format!(
            "callees rated High need {} → MEDIUM",
            multisig_bar(uc.medium_min_voters, uc.medium_min_threshold_pct)
        ),
        _ => format!(
            "callees rated Medium need {}, or to be immutable → LOW",
            multisig_bar(uc.low_min_voters, uc.low_min_threshold_pct)
        ),
    });
    a.with_next(next)
}

/// A short controller label for tables: "Squads 3 of 5 · no delay".
pub fn short_describe(c: &Controller) -> String {
    match c {
        Controller::Immutable => "Immutable".into(),
        Controller::ProgramControlled { upgrade, .. } => {
            format!("Program controlled: {}", short_describe(upgrade))
        }
        Controller::SingleKey { .. } => "Single key".into(),
        Controller::Multisig {
            kind,
            threshold,
            voters,
            time_lock_secs,
            config_authority,
            ..
        } => {
            let name = match kind {
                MultisigKind::SquadsV4 | MultisigKind::SquadsV3 => "Squads",
                MultisigKind::Safe => "Safe",
            };
            let tail = match config_authority {
                Some(ca) if matches!(**ca, Controller::SingleKey { .. }) => {
                    "config key".to_string()
                }
                _ if *time_lock_secs == 0 => "no delay".to_string(),
                _ => format!("{} delay", duration(u64::from(*time_lock_secs))),
            };
            format!("{name} {threshold} of {voters} · {tail}")
        }
        Controller::Timelock { min_delay_secs, .. } => {
            format!("Timelock · {} delay", duration(*min_delay_secs))
        }
        Controller::CannotAct { .. } => "Multisig that cannot act".into(),
        Controller::Unrecognized { .. } => "Unrecognized".into(),
        Controller::Pda { .. } => "Program-derived, controller unresolved".into(),
        Controller::Unavailable { .. } => "Unreadable".into(),
    }
}
