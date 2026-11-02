//! The public static API under `data/api/v2/` (API-1 to API-5): the same data
//! the site reads, one file per protocol and token, a summary and the event
//! log. Every file links its JSON Schema, states its licence (CC BY 4.0) and
//! carries a deprecation slot for the next version.
//!
//! v2 (launch audit, owner decision D7) replaced v1 before launch: every
//! protocol row names its `chain` and `assessment`, and unassessed protocols
//! publish facts but no Stage, ratings or value class (F5).

use std::collections::BTreeMap;
use std::path::Path;

use celebtir_core::snapshot::{
    FactsFile, ProgramHistory, ProtocolFacts, ProtocolMetrics, TokenFacts, TokenMetrics, TokenRole,
};
use celebtir_core::to_canonical_json;
use celebtir_core::{Address, Chain};
use celebtir_score::upgrades::UpgradeMetrics;
use celebtir_score::value::{ControlClass, ValueByControl};
use celebtir_score::{Factor, ProtocolScore, Rating, ScoresFile, Stage, TokenScore};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::CliError;
use crate::coverage::{self, Assessment};
use crate::site_data::{HistoryDay, ProtocolHistoryFile, ReviewStatus, TokenDay, TokenHistoryFile};

/// The API path version. Breaking schema changes go to a new one (API-2).
pub const API_VERSION: &str = "v2";
/// Version of the v2 file formats; additive changes bump the minor.
pub const API_SCHEMA_VERSION: &str = "2.0.0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Licence {
    pub name: String,
    pub url: String,
    /// How to credit the data when reusing it.
    pub attribution: String,
}

pub fn licence() -> Licence {
    Licence {
        name: "CC BY 4.0".into(),
        url: "https://creativecommons.org/licenses/by/4.0/".into(),
        attribution: "Data: Celebtir (CC BY 4.0)".into(),
    }
}

/// Deprecation metadata for future public API migrations (API-2).
/// The pre-launch v1 replacement is the explicit D7 exception.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Deprecation {
    /// Path of the replacing version, e.g. `/data/v2/`.
    pub successor: String,
    /// `YYYY-MM-DD`: the last day this version is published.
    pub sunset_date: String,
}

/// Fields every API file starts with.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Meta {
    /// Root-relative path of this file's JSON Schema.
    #[serde(rename = "$schema")]
    pub schema: String,
    pub api_version: String,
    pub schema_version: String,
    /// The snapshot these data come from (`YYYY-MM-DD`).
    pub snapshot_date: String,
    pub methodology_version: String,
    pub licence: Licence,
    pub deprecation: Option<Deprecation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Links {
    pub page: String,
    pub markdown: String,
    pub api: String,
    pub badge: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SummaryProtocol {
    pub id: String,
    pub name: Option<String>,
    pub category: Option<String>,
    pub chain: Chain,
    /// `unassessed`: listed with its facts, but not rated (F5).
    pub assessment: Assessment,
    /// None for unassessed protocols.
    pub stage: Option<Stage>,
    /// None for unassessed protocols.
    pub ratings: Option<BTreeMap<Factor, Rating>>,
    /// None for unassessed protocols.
    pub control_class: Option<ControlClass>,
    /// The weakest core program's controller in a few words: a fact, shown
    /// for every protocol.
    pub controller_summary: String,
    pub tvl_usd: Option<u64>,
    pub links: Links,
}

/// `/data/v2/summary.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SummaryApi {
    #[serde(flatten)]
    pub meta: Meta,
    /// Chains whose protocols are rated.
    pub assessed_chains: Vec<Chain>,
    /// Sorted by ID.
    pub protocols: Vec<SummaryProtocol>,
    /// Assessed protocols only.
    pub value_by_control: ValueByControl,
}

/// `/data/v2/protocols/<id>.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ProtocolApi {
    #[serde(flatten)]
    pub meta: Meta,
    pub id: String,
    pub chain: Chain,
    pub assessment: Assessment,
    /// A registry reviewer naming a draft is withheld (H2).
    pub facts: ProtocolFacts,
    /// None for unassessed protocols.
    pub score: Option<ProtocolScore>,
    pub metrics: Option<ProtocolMetrics>,
    /// Upgrade history of this protocol's programs, keyed by program.
    pub program_history: BTreeMap<Address, ProgramHistory>,
    /// One entry per snapshot, oldest first.
    pub history: Vec<HistoryDay>,
    pub upgrades: UpgradeMetrics,
    pub review: ReviewStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TokenUser {
    pub protocol: String,
    pub roles: Vec<TokenRole>,
}

/// `/data/v2/tokens/<mint>.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TokenApi {
    #[serde(flatten)]
    pub meta: Meta,
    pub mint: String,
    pub facts: TokenFacts,
    pub score: Option<TokenScore>,
    pub price: Option<TokenMetrics>,
    /// One entry per snapshot, oldest first.
    pub supply_history: Vec<TokenDay>,
    /// Rated protocols using this token, sorted by protocol.
    pub used_by: Vec<TokenUser>,
}

/// `/data/v2/events.json`: the change log, newest first. Rating and Stage
/// events of unassessed protocols are not published.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct EventsApi {
    #[serde(flatten)]
    pub meta: Meta,
    pub events: Vec<celebtir_diff::Event>,
}

/// Every v2 schema by name: `/data/v2/schemas/<name>.schema.json`.
pub fn schemas() -> Vec<(&'static str, Value)> {
    let s = |v: schemars::Schema| serde_json::to_value(v).expect("schemas serialize");
    vec![
        ("summary", s(schemars::schema_for!(SummaryApi))),
        ("protocol", s(schemars::schema_for!(ProtocolApi))),
        ("token", s(schemars::schema_for!(TokenApi))),
        ("events", s(schemars::schema_for!(EventsApi))),
    ]
}

fn err(msg: String) -> CliError {
    CliError::Validation(msg)
}

fn put<T: Serialize>(path: &Path, value: &T) -> Result<(), CliError> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| err(e.to_string()))?;
    }
    let text = to_canonical_json(value).map_err(|e| err(e.to_string()))?;
    std::fs::write(path, text).map_err(|e| err(format!("cannot write {}: {e}", path.display())))
}

pub struct Inputs<'a> {
    pub snapshot_date: &'a str,
    pub facts: &'a FactsFile,
    pub scores: &'a ScoresFile,
    pub histories: &'a BTreeMap<String, ProtocolHistoryFile>,
    pub tokens: &'a BTreeMap<String, TokenHistoryFile>,
    /// Newest first.
    pub events: &'a [celebtir_diff::Event],
}

/// The weakest core program's controller, described, for any protocol.
fn controller_summary(p: &ProtocolFacts) -> String {
    p.programs
        .values()
        .filter(|f| f.role == celebtir_core::snapshot::ProgramRole::Core)
        .map(|f| f.upgrade_control.value())
        .min_by_key(|c| celebtir_score::value::control_class(c))
        .map_or_else(
            || "No core programs".to_string(),
            celebtir_score::factors::short_describe,
        )
}

/// Writes `data/api/v2/` from scratch, schemas included; no older version
/// directory is kept (D7: v1 was replaced before launch).
pub fn write(data: &Path, i: &Inputs) -> Result<(), CliError> {
    let api = data.join("api");
    if api.exists() {
        std::fs::remove_dir_all(&api).map_err(|e| err(e.to_string()))?;
    }
    let root = api.join(API_VERSION);
    let policy = coverage::policy();
    let meta = |name: &str| Meta {
        schema: format!("/data/{API_VERSION}/schemas/{name}.schema.json"),
        api_version: API_VERSION.into(),
        schema_version: API_SCHEMA_VERSION.into(),
        snapshot_date: i.snapshot_date.into(),
        methodology_version: i.scores.methodology_version.clone(),
        licence: licence(),
        deprecation: None,
    };

    for (name, schema) in schemas() {
        put(
            &root.join("schemas").join(format!("{name}.schema.json")),
            &schema,
        )?;
    }

    let mut summary = Vec::new();
    let mut unassessed = std::collections::BTreeSet::new();
    for (id, facts) in &i.facts.protocols {
        let assessment = coverage::assessment(id, facts.chain, i.scores, &policy);
        let score = i
            .scores
            .protocols
            .get(id)
            .filter(|_| assessment == Assessment::Assessed);
        if score.is_none() {
            unassessed.insert(id.clone());
        }
        let metrics = i.facts.metrics.get(id).map(coverage::normalized_metrics);
        let history = i.histories.get(id);
        summary.push(SummaryProtocol {
            id: id.clone(),
            name: facts.name.clone(),
            category: facts.category.clone(),
            chain: facts.chain,
            assessment,
            stage: score.map(|s| s.stage.stage),
            ratings: score.map(|s| s.factors.iter().map(|(f, x)| (*f, x.rating)).collect()),
            control_class: score.map(|s| s.control_class),
            controller_summary: score.map_or_else(
                || controller_summary(facts),
                |s| s.controller_summary.clone(),
            ),
            tvl_usd: metrics.as_ref().and_then(|m| m.tvl.usd()),
            links: Links {
                page: format!("/protocols/{id}/"),
                markdown: format!("/protocols/{id}.md"),
                api: format!("/data/{API_VERSION}/protocols/{id}.json"),
                badge: format!("/badges/{id}.svg"),
            },
        });
        put(
            &root.join("protocols").join(format!("{id}.json")),
            &ProtocolApi {
                meta: meta("protocol"),
                id: id.clone(),
                chain: facts.chain,
                assessment,
                facts: coverage::published_facts(facts),
                score: score.cloned(),
                metrics,
                program_history: facts
                    .programs
                    .keys()
                    .filter_map(|a| {
                        i.facts
                            .program_history
                            .get(a)
                            .map(|h| (a.clone(), h.clone()))
                    })
                    .collect(),
                history: history.map(|h| h.days.clone()).unwrap_or_default(),
                upgrades: history.map(|h| h.upgrades.clone()).unwrap_or_else(|| {
                    celebtir_score::upgrades::upgrade_metrics(&[], i.snapshot_date)
                }),
                review: history.map(|h| h.review.clone()).unwrap_or(ReviewStatus {
                    last_reviewed: facts.last_reviewed.clone(),
                    due: facts.last_reviewed.is_none(),
                    awaiting_human_review: facts
                        .reviewed_by
                        .as_deref()
                        .is_none_or(coverage::is_draft_reviewer),
                }),
            },
        )?;
    }
    put(
        &root.join("summary.json"),
        &SummaryApi {
            meta: meta("summary"),
            assessed_chains: policy
                .coverage
                .as_ref()
                .map(|c| c.assessed_chains.clone())
                .unwrap_or_default(),
            protocols: summary,
            value_by_control: coverage::assessed_value(i.facts, i.scores, &policy),
        },
    )?;

    for (mint, facts) in &i.facts.tokens {
        put(
            &root.join("tokens").join(format!("{mint}.json")),
            &TokenApi {
                meta: meta("token"),
                mint: mint.to_string(),
                facts: facts.clone(),
                score: i.scores.tokens.get(mint).cloned(),
                price: i.facts.token_metrics.get(mint).cloned(),
                supply_history: i
                    .tokens
                    .get(mint.as_str())
                    .map(|t| t.days.clone())
                    .unwrap_or_default(),
                used_by: i
                    .facts
                    .protocols
                    .iter()
                    .filter_map(|(id, p)| {
                        p.tokens.get(mint).map(|roles| TokenUser {
                            protocol: id.clone(),
                            roles: roles.clone(),
                        })
                    })
                    .collect(),
            },
        )?;
    }

    put(
        &root.join("events.json"),
        &EventsApi {
            meta: meta("events"),
            events: i
                .events
                .iter()
                .filter(|e| {
                    !(matches!(
                        e.kind,
                        celebtir_diff::EventKind::RatingChanged
                            | celebtir_diff::EventKind::StageChanged
                    ) && e.protocol.as_ref().is_some_and(|p| unassessed.contains(p)))
                })
                .cloned()
                .collect(),
        },
    )?;
    Ok(())
}
