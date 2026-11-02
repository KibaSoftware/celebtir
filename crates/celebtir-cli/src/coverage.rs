//! Which protocols the published outputs rate (F5). One policy, from the
//! shipped methodology's `[coverage]`, applied to every day: a protocol is
//! assessed when its chain is rated and its day has a published score.
//! Older days scored before a chain left coverage keep their files; their
//! values are withheld here, never rescored or reclassified.

use std::collections::BTreeMap;

use celebtir_core::Chain;
use celebtir_core::snapshot::{Activity, FactsFile, ProtocolFacts, ProtocolMetrics};
use celebtir_score::value::{ControlClass, ValueByControl, ValueRow};
use celebtir_score::{Methodology, ScoresFile};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Assessment {
    Assessed,
    Unassessed,
}

/// The coverage policy of the methodology this build ships.
pub fn policy() -> Methodology {
    Methodology::shipped()
}

pub fn assessment(id: &str, chain: Chain, scores: &ScoresFile, m: &Methodology) -> Assessment {
    if m.assessed(chain) && scores.protocols.contains_key(id) {
        Assessment::Assessed
    } else {
        Assessment::Unassessed
    }
}

pub fn is_assessed(id: &str, p: &ProtocolFacts, scores: &ScoresFile, m: &Methodology) -> bool {
    assessment(id, p.chain, scores, m) == Assessment::Assessed
}

/// TVL by control class over assessed protocols only, from each protocol's
/// published class; the day's own class set is kept.
pub fn assessed_value(facts: &FactsFile, scores: &ScoresFile, m: &Methodology) -> ValueByControl {
    let classes: Vec<ControlClass> = scores
        .value_by_control
        .all
        .classes
        .keys()
        .copied()
        .collect();
    let empty = || ValueRow::empty_of(&classes);
    let mut out = ValueByControl {
        all: empty(),
        ..ValueByControl::default()
    };
    for (id, p) in &facts.protocols {
        let Some(score) = scores
            .protocols
            .get(id)
            .filter(|_| is_assessed(id, p, scores, m))
        else {
            continue;
        };
        let Some(tvl) = facts.metrics.get(id).and_then(|x| x.tvl.usd()) else {
            out.excluded.push(id.clone());
            continue;
        };
        add(&mut out.all, score.control_class, tvl);
        let category = p.category.clone().unwrap_or_else(|| "uncategorized".into());
        add(
            out.by_category.entry(category).or_insert_with(empty),
            score.control_class,
            tvl,
        );
    }
    out
}

fn add(row: &mut ValueRow, class: ControlClass, usd: u64) {
    *row.classes.entry(class).or_default() += usd;
    row.total += usd;
}

/// The 7-day change re-derived with the methodology's gap rule (D5), so
/// days collected before the rule publish the same definition.
pub fn normalized_activity(a: &Activity) -> Activity {
    match a {
        Activity::Usd { days, latest, .. } => {
            let (change_7d_bp, gap_days) = celebtir_core::activity::change_7d(days);
            Activity::Usd {
                days: days.clone(),
                latest: *latest,
                change_7d_bp,
                gap_days,
            }
        }
        other => other.clone(),
    }
}

pub fn normalized_metrics(m: &ProtocolMetrics) -> ProtocolMetrics {
    let mut m = m.clone();
    m.fees = normalized_activity(&m.fees);
    m.volume = normalized_activity(&m.volume);
    m
}

/// H2: a registry reviewer that names a draft loop is not a reviewer.
pub fn is_draft_reviewer(name: &str) -> bool {
    name.to_ascii_lowercase().contains("draft")
}

/// Protocol facts as published: a draft reviewer is withheld (H2).
pub fn published_facts(p: &ProtocolFacts) -> ProtocolFacts {
    let mut p = p.clone();
    if p.reviewed_by.as_deref().is_some_and(is_draft_reviewer) {
        p.reviewed_by = None;
    }
    p
}

/// `history/coverage.json`: the assessment of every protocol on the newest
/// day, read by the site so no page applies its own rule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CoverageFile {
    pub schema_version: String,
    pub methodology_version: String,
    pub assessed_chains: Vec<Chain>,
    pub protocols: BTreeMap<String, CoverageEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CoverageEntry {
    pub chain: Chain,
    pub assessment: Assessment,
    /// Why an unassessed protocol is not rated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

pub fn coverage_file(facts: &FactsFile, scores: &ScoresFile, m: &Methodology) -> CoverageFile {
    CoverageFile {
        schema_version: "0.1.0".into(),
        methodology_version: m.version.clone(),
        assessed_chains: m
            .coverage
            .as_ref()
            .map(|c| c.assessed_chains.clone())
            .unwrap_or_default(),
        protocols: facts
            .protocols
            .iter()
            .map(|(id, p)| {
                let a = assessment(id, p.chain, scores, m);
                (
                    id.clone(),
                    CoverageEntry {
                        chain: p.chain,
                        assessment: a,
                        reason: (a == Assessment::Unassessed).then(|| {
                            scores
                                .unassessed
                                .get(id)
                                .map(|u| u.reason.clone())
                                .unwrap_or_else(|| {
                                    format!(
                                        "methodology v{} does not rate this chain's protocols; facts are listed unrated",
                                        m.version
                                    )
                                })
                        }),
                    },
                )
            })
            .collect(),
    }
}
