//! Change detection: every difference between the last good snapshot and
//! today's becomes a typed, permanent event that says whether the chain, the
//! registry or the methodology changed (CHG-1, CHG-2). Pure: no I/O.

use std::collections::{BTreeMap, BTreeSet};

use celebtir_core::snapshot::{
    CodeTransparency, Dependencies, FactsFile, ProtocolFacts, UpgradeKind,
};
use celebtir_core::{Address, Controller, Evidence};
use celebtir_score::{Factor, Rating, ScoresFile, Stage};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// Version of the `events.json` format.
pub const EVENTS_SCHEMA_VERSION: &str = "0.2.0";

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    ControllerChanged,
    MadeImmutable,
    ProgramUpgraded,
    /// A ProgramData `SetAuthority` transaction (F6).
    UpgradeAuthoritySet,
    /// A ProgramData authority removal: the program became immutable (F6).
    UpgradeAuthorityRemoved,
    MultisigChanged,
    AdminKeyChanged,
    VerificationChanged,
    OracleChanged,
    DependencyAdded,
    DependencyRemoved,
    TokenControlChanged,
    RatingChanged,
    StageChanged,
    RegistryEdit,
    MethodologyChange,
    /// A HIP-3 DEX appeared, disappeared, or changed its deployer, oracle
    /// control, fee recipient or caps (HIP-6).
    Hip3Changed,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Critical,
    Warning,
    Positive,
    Info,
}

/// What changed: the chain, the registry, or the methodology (CHG-2).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    OnChain,
    RegistryEdit,
    Methodology,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Event {
    /// Stable: `<date>:<protocol>:<kind>:<subject>` (CHG-4).
    pub id: String,
    /// The snapshot day the change was detected.
    pub date: String,
    /// Days since the snapshot compared against: 1 normally, more after missed
    /// runs, when the change happened at some point within that window (CHG-5).
    pub detected_within_days: u32,
    pub protocol: Option<String>,
    /// The program, mint, account or factor the event is about.
    pub subject: String,
    pub kind: EventKind,
    pub severity: Severity,
    pub source: Source,
    pub summary: String,
    pub before: Value,
    pub after: Value,
    pub evidence: Vec<Evidence>,
    /// The transaction behind the change, when known (CHG-1).
    pub signature: Option<String>,
    /// For a correction: the id of the event it corrects (CHG-4). Events are
    /// never edited or deleted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub corrects: Option<String>,
    /// For a transaction on chain: the UTC date of its block (F6). `date` is
    /// the day Celebtir recorded it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub occurred_on: Option<String>,
    /// The transaction's slot, when the event is a transaction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slot: Option<u64>,
    /// Recorded from ProgramData history that predates the comparison: the
    /// change is real, but older than the window it was recorded in (F6).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub backfill: bool,
}

/// `data/snapshots/<date>/events.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct EventsFile {
    pub schema_version: String,
    pub date: String,
    /// The snapshot compared against.
    pub previous_date: String,
    pub events: Vec<Event>,
}

pub struct Snapshot<'a> {
    pub date: &'a str,
    pub facts: &'a FactsFile,
    pub scores: &'a ScoresFile,
}

struct Builder<'a> {
    date: &'a str,
    window_days: u32,
    events: Vec<Event>,
}

impl Builder<'_> {
    #[allow(clippy::too_many_arguments)]
    fn push(
        &mut self,
        protocol: Option<&str>,
        subject: &str,
        kind: EventKind,
        severity: Severity,
        source: Source,
        summary: String,
        before: Value,
        after: Value,
        evidence: Vec<Evidence>,
        signature: Option<String>,
    ) {
        let kind_name = serde_json::to_value(kind).ok();
        let kind_name = kind_name
            .as_ref()
            .and_then(Value::as_str)
            .unwrap_or("event");
        self.events.push(Event {
            id: format!(
                "{}:{}:{kind_name}:{subject}",
                self.date,
                protocol.unwrap_or("-")
            ),
            date: self.date.to_string(),
            detected_within_days: self.window_days,
            protocol: protocol.map(str::to_string),
            subject: subject.to_string(),
            kind,
            severity,
            source,
            summary: if self.window_days > 1 {
                format!("{summary} (detected within {} days)", self.window_days)
            } else {
                summary
            },
            before,
            after,
            evidence,
            signature,
            corrects: None,
            occurred_on: None,
            slot: None,
            backfill: false,
        });
    }
}

/// Who a controller is, ignoring parameters: the key or account address.
fn identity(c: &Controller) -> Option<String> {
    match c {
        Controller::Immutable => Some("immutable".into()),
        Controller::SingleKey { key }
        | Controller::Unrecognized { key, .. }
        | Controller::Pda { key, .. }
        | Controller::ProgramControlled { key, .. } => Some(key.to_string()),
        Controller::Multisig { address, .. }
        | Controller::Timelock { address, .. }
        | Controller::CannotAct { address, .. } => Some(address.to_string()),
        // A failed read is not a change on chain.
        Controller::Unavailable { .. } => None,
    }
}

fn to_json<T: Serialize>(v: &T) -> Value {
    serde_json::to_value(v).unwrap_or(Value::Null)
}

/// `(protocol, signature)` pairs earlier event files already reported.
pub type Reported = BTreeSet<(String, String)>;

/// The changes between two snapshots. Upgrade-history transactions already
/// in `old` are not repeated; see [`diff_with`] for the catch-up form.
pub fn diff(old: &Snapshot<'_>, new: &Snapshot<'_>) -> Vec<Event> {
    diff_inner(old, new, None)
}

/// [`diff`], also writing every ProgramData transaction in `new`'s history
/// that no earlier event file reported (F6). Run on every new snapshot, it
/// backfills history recorded before upgrade events existed exactly once.
pub fn diff_with(old: &Snapshot<'_>, new: &Snapshot<'_>, reported: &Reported) -> Vec<Event> {
    diff_inner(old, new, Some(reported))
}

/// The first snapshot ever: every ProgramData transaction in its history,
/// flagged as backfill (F6).
pub fn initial_events(new: &Snapshot<'_>) -> Vec<Event> {
    let mut b = Builder {
        date: new.date,
        window_days: 1,
        events: Vec::new(),
    };
    let empty = FactsFile::new(BTreeMap::new());
    let reported = Reported::new();
    for (id, p) in &new.facts.protocols {
        for program in p.programs.keys() {
            upgrade_events(&mut b, id, program, &empty, new.facts, Some(&reported), "");
        }
    }
    b.events.sort_by(|x, y| x.id.cmp(&y.id));
    b.events
}

fn diff_inner(old: &Snapshot<'_>, new: &Snapshot<'_>, reported: Option<&Reported>) -> Vec<Event> {
    let mut b = Builder {
        date: new.date,
        window_days: window_days(old.date, new.date),
        events: Vec::new(),
    };
    let methodology_changed = old.scores.methodology_version != new.scores.methodology_version;
    if methodology_changed {
        b.push(
            None,
            "methodology",
            EventKind::MethodologyChange,
            Severity::Info,
            Source::Methodology,
            format!(
                "methodology {} replaced {}; every protocol was re-scored",
                new.scores.methodology_version, old.scores.methodology_version
            ),
            json!(old.scores.methodology_version),
            json!(new.scores.methodology_version),
            Vec::new(),
            None,
        );
    }

    let ids: BTreeSet<&String> = old
        .facts
        .protocols
        .keys()
        .chain(new.facts.protocols.keys())
        .collect();
    for id in ids {
        match (old.facts.protocols.get(id), new.facts.protocols.get(id)) {
            (None, Some(_)) => b.push(
                Some(id),
                id,
                EventKind::RegistryEdit,
                Severity::Info,
                Source::RegistryEdit,
                format!("{id} added to coverage"),
                Value::Null,
                json!("covered"),
                Vec::new(),
                None,
            ),
            // A protocol missing today may simply have failed to collect;
            // removals from coverage are recorded by the registry, not here.
            (Some(_), None) => {}
            (Some(o), Some(n)) => {
                protocol_events(&mut b, id, o, n, old.facts, new.facts, reported, old.date)
            }
            (None, None) => {}
        }
        // A protocol new to coverage still reports its recorded history.
        if let (None, Some(n), Some(r)) = (
            old.facts.protocols.get(id),
            new.facts.protocols.get(id),
            reported,
        ) {
            let empty = FactsFile::new(BTreeMap::new());
            for program in n.programs.keys() {
                upgrade_events(&mut b, id, program, &empty, new.facts, Some(r), old.date);
            }
        }
        rating_events(&mut b, id, old, new, methodology_changed);
    }
    token_events(&mut b, old.facts, new.facts);
    hip3_events(&mut b, old.facts, new.facts);
    b.events.sort_by(|x, y| x.id.cmp(&y.id));
    b.events
}

/// HIP-6. Only two listed days are compared: a day the info API failed is
/// not a change on chain.
fn hip3_events(b: &mut Builder<'_>, old: &FactsFile, new: &FactsFile) {
    use celebtir_core::snapshot::Hip3Snapshot;
    let (Some(Hip3Snapshot::Listed { dexs: o }), Some(Hip3Snapshot::Listed { dexs: n })) =
        (&old.hip3, &new.hip3)
    else {
        return;
    };
    let names: BTreeSet<&String> = o.keys().chain(n.keys()).collect();
    for name in names {
        let subject = |field: &str| format!("hip3:{name}{field}");
        match (o.get(name), n.get(name)) {
            (None, Some(d)) => b.push(
                None,
                &subject(""),
                EventKind::Hip3Changed,
                Severity::Info,
                Source::OnChain,
                format!(
                    "HIP-3 DEX {name} is now listed, deployed by {}",
                    d.deployer.value()
                ),
                Value::Null,
                json!(d.deployer.value()),
                d.deployer.evidence().to_vec(),
                None,
            ),
            (Some(d), None) => b.push(
                None,
                &subject(""),
                EventKind::Hip3Changed,
                Severity::Info,
                Source::OnChain,
                format!("HIP-3 DEX {name} is no longer listed by perpDexs"),
                json!(d.deployer.value()),
                Value::Null,
                d.deployer.evidence().to_vec(),
                None,
            ),
            (Some(a), Some(z)) => {
                let fields: [(&str, Value, Value, Vec<Evidence>, Severity); 5] = [
                    (
                        ".deployer",
                        to_json(a.deployer.value()),
                        to_json(z.deployer.value()),
                        z.deployer.evidence().to_vec(),
                        Severity::Warning,
                    ),
                    (
                        ".oracle_updater",
                        to_json(a.oracle_updater.value()),
                        to_json(z.oracle_updater.value()),
                        z.oracle_updater.evidence().to_vec(),
                        Severity::Warning,
                    ),
                    (
                        ".sub_deployers",
                        to_json(a.sub_deployers.value()),
                        to_json(z.sub_deployers.value()),
                        z.sub_deployers.evidence().to_vec(),
                        Severity::Warning,
                    ),
                    (
                        ".fee_recipient",
                        to_json(a.fee_recipient.value()),
                        to_json(z.fee_recipient.value()),
                        z.fee_recipient.evidence().to_vec(),
                        Severity::Info,
                    ),
                    (
                        ".oi_caps",
                        to_json(a.oi_caps.value()),
                        to_json(z.oi_caps.value()),
                        z.oi_caps.evidence().to_vec(),
                        Severity::Info,
                    ),
                ];
                for (field, before, after, evidence, severity) in fields {
                    if before != after {
                        b.push(
                            None,
                            &subject(field),
                            EventKind::Hip3Changed,
                            severity,
                            Source::OnChain,
                            format!(
                                "HIP-3 DEX {name}: {} changed",
                                field.trim_start_matches('.').replace('_', " ")
                            ),
                            before,
                            after,
                            evidence,
                            None,
                        );
                    }
                }
            }
            (None, None) => {}
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn protocol_events(
    b: &mut Builder<'_>,
    id: &str,
    o: &ProtocolFacts,
    n: &ProtocolFacts,
    old_facts: &FactsFile,
    new_facts: &FactsFile,
    reported: Option<&Reported>,
    old_date: &str,
) {
    // Programs added to or removed from the registry entry (CHG-2).
    for program in o.programs.keys().filter(|p| !n.programs.contains_key(*p)) {
        b.push(
            Some(id),
            program.as_str(),
            EventKind::RegistryEdit,
            Severity::Info,
            Source::RegistryEdit,
            format!("program {program} removed from the registry entry"),
            json!(program),
            Value::Null,
            Vec::new(),
            None,
        );
    }
    for program in n.programs.keys().filter(|p| !o.programs.contains_key(*p)) {
        b.push(
            Some(id),
            program.as_str(),
            EventKind::RegistryEdit,
            Severity::Info,
            Source::RegistryEdit,
            format!("program {program} added to the registry entry"),
            Value::Null,
            json!(program),
            Vec::new(),
            None,
        );
    }

    for (program, nf) in &n.programs {
        let Some(of) = o.programs.get(program) else {
            continue;
        };
        controller_events(
            b,
            id,
            program.as_str(),
            "upgrade authority",
            of.upgrade_control.value(),
            nf.upgrade_control.value(),
            nf.upgrade_control.evidence().to_vec(),
        );
        upgrade_events(b, id, program, old_facts, new_facts, reported, old_date);
    }

    for (key, na) in &n.admin {
        match o.admin.get(key) {
            None => b.push(
                Some(id),
                key,
                EventKind::RegistryEdit,
                Severity::Info,
                Source::RegistryEdit,
                format!("admin field {key} added to the registry entry"),
                Value::Null,
                json!(key),
                Vec::new(),
                None,
            ),
            Some(oa) => {
                let (before, after) = (oa.controller.value(), na.controller.value());
                if let (Some(x), Some(y)) = (identity(before), identity(after)) {
                    if x != y {
                        b.push(
                            Some(id),
                            key,
                            EventKind::AdminKeyChanged,
                            Severity::Warning,
                            Source::OnChain,
                            format!("admin key `{}` changed from {x} to {y}", na.field),
                            to_json(before),
                            to_json(after),
                            na.controller.evidence().to_vec(),
                            None,
                        );
                    }
                }
            }
        }
    }

    for (account, no) in &n.oracles {
        match o.oracles.get(account) {
            None => b.push(
                Some(id),
                account.as_str(),
                EventKind::RegistryEdit,
                Severity::Info,
                Source::RegistryEdit,
                format!("oracle {account} added for {}", no.market),
                Value::Null,
                to_json(no.source.value()),
                no.source.evidence().to_vec(),
                None,
            ),
            Some(oo) if oo.source.value() != no.source.value() => {
                let unreadable = |s: &celebtir_core::snapshot::OracleSource| {
                    matches!(s, celebtir_core::snapshot::OracleSource::Unavailable { .. })
                };
                if !unreadable(oo.source.value()) && !unreadable(no.source.value()) {
                    b.push(
                        Some(id),
                        account.as_str(),
                        EventKind::OracleChanged,
                        Severity::Warning,
                        Source::OnChain,
                        format!("price source for {} changed", no.market),
                        to_json(oo.source.value()),
                        to_json(no.source.value()),
                        no.source.evidence().to_vec(),
                        None,
                    );
                }
            }
            Some(_) => {}
        }
    }
}

fn controller_events(
    b: &mut Builder<'_>,
    id: &str,
    subject: &str,
    what: &str,
    before: &Controller,
    after: &Controller,
    evidence: Vec<Evidence>,
) {
    let (Some(x), Some(y)) = (identity(before), identity(after)) else {
        return;
    };
    if x != y {
        let (kind, severity, summary) = if *after == Controller::Immutable {
            (
                EventKind::MadeImmutable,
                Severity::Positive,
                format!("{what} of {subject} removed: the program is now immutable"),
            )
        } else {
            (
                EventKind::ControllerChanged,
                Severity::Critical,
                format!("{what} of {subject} changed from {x} to {y}"),
            )
        };
        b.push(
            Some(id),
            subject,
            kind,
            severity,
            Source::OnChain,
            summary,
            to_json(before),
            to_json(after),
            evidence,
            None,
        );
        return;
    }
    // Same multisig, different parameters.
    if let (
        Controller::Multisig {
            threshold: t0,
            voters: v0,
            time_lock_secs: l0,
            config_authority: c0,
            ..
        },
        Controller::Multisig {
            threshold: t1,
            voters: v1,
            time_lock_secs: l1,
            config_authority: c1,
            ..
        },
    ) = (before, after)
    {
        if (t0, v0, l0, c0) != (t1, v1, l1, c1) {
            let severity = if c1.is_some() {
                Severity::Critical
            } else {
                Severity::Warning
            };
            b.push(
                Some(id),
                subject,
                EventKind::MultisigChanged,
                severity,
                Source::OnChain,
                format!(
                    "multisig {x} changed: {t0} of {v0} → {t1} of {v1}, time lock {l0} s → {l1} s{}",
                    if c1.is_some() { ", config authority set" } else { "" }
                ),
                to_json(before),
                to_json(after),
                evidence,
                None,
            );
        }
    }
}

/// ProgramData transactions (upgrades and authority changes) as events,
/// each dated by its block and linked to its transaction (CHG-1, F6).
///
/// Without `reported`, only transactions new since `old` are written. With
/// it, every transaction no earlier event file reported is written once;
/// those already in `old`'s history, older than `old_date`, or in a history
/// that first appears are flagged as backfill.
fn upgrade_events(
    b: &mut Builder<'_>,
    id: &str,
    program: &Address,
    old_facts: &FactsFile,
    new_facts: &FactsFile,
    reported: Option<&Reported>,
    old_date: &str,
) {
    let seen: BTreeSet<&str> = old_facts
        .program_history
        .get(program)
        .map(|h| h.events.iter().map(|e| e.signature.as_str()).collect())
        .unwrap_or_default();
    // A cold cache or bounded scan can omit transactions already present in
    // the preceding immutable snapshot. Catch-up must retain that evidence;
    // otherwise the first post-fix run silently loses its historical events.
    let histories = new_facts.program_history.get(program).into_iter().chain(
        old_facts
            .program_history
            .get(program)
            .filter(|_| reported.is_some()),
    );
    let mut emitted = BTreeSet::new();
    let first_history = !old_facts.program_history.contains_key(program);
    for e in histories.flat_map(|h| &h.events) {
        let (kind, severity, verb) = match e.kind {
            UpgradeKind::Upgrade => (EventKind::ProgramUpgraded, Severity::Warning, "upgraded"),
            UpgradeKind::SetAuthority => (
                EventKind::UpgradeAuthoritySet,
                Severity::Critical,
                "upgrade authority set",
            ),
            UpgradeKind::RemoveAuthority => (
                EventKind::UpgradeAuthorityRemoved,
                Severity::Positive,
                "upgrade authority removed: now immutable",
            ),
            _ => continue,
        };
        if !emitted.insert((&e.signature, e.slot, format!("{kind:?}"))) {
            continue;
        }
        let already = match reported {
            Some(r) => r.contains(&(id.to_string(), e.signature.clone())),
            // Plain diff: what the old snapshot held was reported before,
            // and a first history is left to the catch-up form.
            None => seen.contains(e.signature.as_str()) || first_history,
        };
        if already {
            continue;
        }
        let backfill = first_history
            || seen.contains(e.signature.as_str())
            || e.date.as_deref().is_some_and(|d| d < old_date);
        let detected = match (
            e.date.as_deref().and_then(celebtir_core::date::days),
            celebtir_core::date::days(b.date),
        ) {
            (Some(on), Some(today)) if today >= on => u32::try_from(today - on).unwrap_or(u32::MAX),
            _ => b.window_days,
        };
        let when = match &e.date {
            Some(d) => format!("on {d} (slot {})", e.slot),
            None => format!("at slot {}", e.slot),
        };
        let mut summary = format!("program {program} {verb} {when}");
        if let Some(a) = &e.new_authority {
            summary.push_str(&format!("; new authority {a}"));
        }
        if backfill {
            summary.push_str("; recorded from ProgramData history");
        }
        let kind_name = serde_json::to_value(kind)
            .ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_default();
        b.events.push(Event {
            id: format!("{}:{id}:{kind_name}:{program}@{}", b.date, e.signature),
            date: b.date.to_string(),
            detected_within_days: detected,
            protocol: Some(id.to_string()),
            subject: format!("{program}@{}", e.slot),
            kind,
            severity,
            source: Source::OnChain,
            summary,
            before: Value::Null,
            after: json!({"slot": e.slot, "date": e.date, "new_authority": e.new_authority}),
            evidence: vec![Evidence::transaction(
                celebtir_core::Chain::Solana,
                program.clone(),
                e.slot,
                &e.signature,
                format!("ProgramData transaction ({kind_name})"),
            )],
            signature: Some(e.signature.clone()),
            corrects: None,
            occurred_on: e.date.clone(),
            slot: Some(e.slot),
            backfill,
        });
    }
}

fn rating_events(
    b: &mut Builder<'_>,
    id: &str,
    old: &Snapshot<'_>,
    new: &Snapshot<'_>,
    methodology_changed: bool,
) {
    let (Some(os), Some(ns)) = (old.scores.protocols.get(id), new.scores.protocols.get(id)) else {
        return;
    };
    // Attribute the change: methodology first, then whatever else changed
    // today. Backfilled history is not today's change.
    let today: BTreeMap<Source, usize> = b
        .events
        .iter()
        .filter(|e| e.protocol.as_deref() == Some(id) && !e.backfill)
        .fold(BTreeMap::new(), |mut m, e| {
            *m.entry(e.source).or_default() += 1;
            m
        });
    let source = if methodology_changed {
        Source::Methodology
    } else if today.contains_key(&Source::OnChain) || !today.contains_key(&Source::RegistryEdit) {
        Source::OnChain
    } else {
        Source::RegistryEdit
    };
    let direction = |a: Rating, z: Rating| match (a.level(), z.level()) {
        (Some(x), Some(y)) if y < x => Severity::Positive,
        (Some(x), Some(y)) if y > x => Severity::Warning,
        _ => Severity::Info,
    };
    for f in Factor::ALL {
        let (a, z) = (os.factors[&f].rating, ns.factors[&f].rating);
        if a != z {
            b.push(
                Some(id),
                to_json(&f).as_str().unwrap_or_default(),
                EventKind::RatingChanged,
                direction(a, z),
                source,
                format!("{}: {} → {}", f.title(), a.label(), z.label()),
                to_json(&a),
                to_json(&z),
                ns.factors[&f].evidence.clone(),
                None,
            );
        }
    }
    let (a, z) = (os.stage.stage, ns.stage.stage);
    if a != z {
        let severity = match (a, z) {
            (x, y) if y > x && x != Stage::Unrated => Severity::Positive,
            (x, y) if y < x && y != Stage::Unrated => Severity::Warning,
            _ => Severity::Info,
        };
        b.push(
            Some(id),
            "stage",
            EventKind::StageChanged,
            severity,
            source,
            format!("{} → {}", a.label(), z.label()),
            to_json(&a),
            to_json(&z),
            Vec::new(),
            None,
        );
    }
}

/// The published JSON Schema for `events.json`.
pub fn events_schema() -> serde_json::Value {
    serde_json::to_value(schemars::schema_for!(EventsFile)).expect("schemas serialize")
}

/// Consecutive runs a change must hold before it becomes an event (CHG-3).
pub const DEPENDENCY_RUNS: usize = 3;
pub const VERIFICATION_RUNS: usize = 2;

/// `Some(new value)` when the newest `runs` values agree and differ from the
/// value just before them: the day a change is confirmed, exactly once.
fn confirmed(values: &[Option<bool>], runs: usize) -> Option<bool> {
    if values.len() < runs + 1 {
        return None;
    }
    let tail = &values[values.len() - runs..];
    let now = tail[0]?;
    if tail.iter().any(|v| *v != Some(now)) {
        return None;
    }
    (values[values.len() - runs - 1]? != now).then_some(now)
}

/// Events that must hold for several runs before they count: dependencies
/// (3) and verified builds (2). `window` is oldest first; the last snapshot
/// is today.
pub fn confirmed_events(window: &[Snapshot<'_>]) -> Vec<Event> {
    let Some(today) = window.last() else {
        return Vec::new();
    };
    let previous = window.len().checked_sub(2).map(|i| window[i].date);
    let mut b = Builder {
        date: today.date,
        window_days: previous.map_or(1, |p| window_days(p, today.date)),
        events: Vec::new(),
    };
    for (id, p) in &today.facts.protocols {
        // Dependency membership per day; unreadable or stale samples are unknown.
        let sets: Vec<Option<BTreeSet<&Address>>> = window
            .iter()
            .map(
                |s| match s.facts.protocols.get(id).map(|p| p.dependencies.value()) {
                    Some(Dependencies::Sampled {
                        callees,
                        stale: false,
                        ..
                    }) => Some(callees.keys().collect()),
                    _ => None,
                },
            )
            .collect();
        let candidates: BTreeSet<&Address> = sets.iter().flatten().flatten().copied().collect();
        for callee in candidates {
            let seen: Vec<Option<bool>> = sets
                .iter()
                .map(|s| s.as_ref().map(|s| s.contains(callee)))
                .collect();
            if let Some(added) = confirmed(&seen, DEPENDENCY_RUNS) {
                let (kind, verb) = if added {
                    (EventKind::DependencyAdded, "now calls")
                } else {
                    (EventKind::DependencyRemoved, "no longer calls")
                };
                b.push(
                    Some(id),
                    callee.as_str(),
                    kind,
                    Severity::Info,
                    Source::OnChain,
                    format!("{id} {verb} {callee} (seen in {DEPENDENCY_RUNS} consecutive samples)"),
                    json!(!added),
                    json!(added),
                    p.dependencies.evidence().to_vec(),
                    None,
                );
            }
        }

        for (program, f) in &p.programs {
            let verified: Vec<Option<bool>> = window
                .iter()
                .map(|s| {
                    let t = s.facts.protocols.get(id)?.programs.get(program)?;
                    match t.code_transparency.value() {
                        CodeTransparency::Verified { .. } => Some(true),
                        CodeTransparency::Unavailable { .. } => None,
                        _ => Some(false),
                    }
                })
                .collect();
            if let Some(now) = confirmed(&verified, VERIFICATION_RUNS) {
                b.push(
                    Some(id),
                    program.as_str(),
                    EventKind::VerificationChanged,
                    if now {
                        Severity::Positive
                    } else {
                        Severity::Warning
                    },
                    Source::OnChain,
                    format!(
                        "verified build of {program} {}",
                        if now { "gained" } else { "lost" }
                    ),
                    json!(!now),
                    json!(now),
                    f.code_transparency.evidence().to_vec(),
                    None,
                );
            }
        }
    }
    b.events.sort_by(|x, y| x.id.cmp(&y.id));
    b.events
}

/// Whole days between two `YYYY-MM-DD` dates, at least 1.
fn window_days(old: &str, new: &str) -> u32 {
    match (
        celebtir_core::date::days(old),
        celebtir_core::date::days(new),
    ) {
        (Some(a), Some(b)) if b > a => u32::try_from(b - a).unwrap_or(u32::MAX),
        _ => 1,
    }
}

/// TOK-8: changes to a mint's authorities and extensions.
fn token_events(b: &mut Builder<'_>, old: &FactsFile, new: &FactsFile) {
    for (mint, nt) in &new.tokens {
        let Some(ot) = old.tokens.get(mint) else {
            continue;
        };
        let (oe, ne) = (ot.extensions.value(), nt.extensions.value());
        let hook_controller = |e: &celebtir_core::snapshot::TokenExtensions| {
            e.transfer_hook.as_ref().map(|h| h.authority.clone())
        };
        let fields: [(&str, Option<Controller>, Option<Controller>); 8] = [
            (
                "mint_authority",
                Some(ot.mint_authority.value().clone()),
                Some(nt.mint_authority.value().clone()),
            ),
            (
                "freeze_authority",
                Some(ot.freeze_authority.value().clone()),
                Some(nt.freeze_authority.value().clone()),
            ),
            (
                "permanent_delegate",
                oe.permanent_delegate.clone(),
                ne.permanent_delegate.clone(),
            ),
            (
                "transfer_hook_authority",
                hook_controller(oe),
                hook_controller(ne),
            ),
            (
                "transfer_fee_authority",
                oe.transfer_fee_authority.clone(),
                ne.transfer_fee_authority.clone(),
            ),
            (
                "pause_authority",
                oe.pause_authority.clone(),
                ne.pause_authority.clone(),
            ),
            (
                "mint_close_authority",
                oe.mint_close_authority.clone(),
                ne.mint_close_authority.clone(),
            ),
            (
                "metadata_update_authority",
                oe.metadata_update_authority.clone(),
                ne.metadata_update_authority.clone(),
            ),
        ];
        let renounced = |c: &Option<Controller>| matches!(c, None | Some(Controller::Immutable));
        for (field, before, after) in fields {
            let id_of = |c: &Option<Controller>| match c {
                None => Some("none".to_string()),
                Some(c) => identity(c),
            };
            let (Some(x), Some(y)) = (id_of(&before), id_of(&after)) else {
                continue;
            };
            if x == y {
                continue;
            }
            let severity =
                if field == "permanent_delegate" && renounced(&before) && !renounced(&after) {
                    Severity::Critical
                } else if renounced(&after) {
                    Severity::Positive
                } else {
                    Severity::Warning
                };
            b.push(
                None,
                &format!("{mint}:{field}"),
                EventKind::TokenControlChanged,
                severity,
                Source::OnChain,
                format!("{} {field} changed from {x} to {y}", nt.symbol),
                to_json(&before),
                to_json(&after),
                nt.extensions.evidence().to_vec(),
                None,
            );
        }

        if oe.default_frozen != ne.default_frozen {
            b.push(
                None,
                &format!("{mint}:default_frozen"),
                EventKind::TokenControlChanged,
                if ne.default_frozen {
                    Severity::Warning
                } else {
                    Severity::Positive
                },
                Source::OnChain,
                format!(
                    "{}: new accounts {} frozen",
                    nt.symbol,
                    if ne.default_frozen {
                        "now start"
                    } else {
                        "no longer start"
                    }
                ),
                json!(oe.default_frozen),
                json!(ne.default_frozen),
                nt.extensions.evidence().to_vec(),
                None,
            );
        }
        let list = |e: &celebtir_core::snapshot::TokenExtensions| {
            let mut v: Vec<String> = e.other.clone();
            v.extend(e.unknown.iter().map(|t| format!("type {t}")));
            v.sort();
            v
        };
        let (ol, nl) = (list(oe), list(ne));
        if ol != nl {
            let added: Vec<&String> = nl.iter().filter(|x| !ol.contains(x)).collect();
            b.push(
                None,
                &format!("{mint}:extensions"),
                EventKind::TokenControlChanged,
                Severity::Warning,
                Source::OnChain,
                if added.is_empty() {
                    format!("{}: extensions removed", nt.symbol)
                } else {
                    let names: Vec<&str> = added.iter().map(|s| s.as_str()).collect();
                    format!("{}: extensions enabled: {}", nt.symbol, names.join(", "))
                },
                json!(ol),
                json!(nl),
                nt.extensions.evidence().to_vec(),
                None,
            );
        }
    }
}
