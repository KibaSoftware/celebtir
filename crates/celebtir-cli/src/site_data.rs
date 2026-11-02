//! `celebtir build-site-data`: derives what the site reads from the dated
//! snapshots: `data/latest/` (the newest snapshot) and
//! `data/history/<protocol>.json` (compact daily series for charts).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use celebtir_core::snapshot::{Activity, FactsFile, ProgramRole};
use celebtir_core::to_canonical_json;
use celebtir_score::upgrades::{UpgradeMetrics, upgrade_metrics};
use celebtir_score::{Factor, Rating, ScoresFile, Stage};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{CliError, Format};

/// Version of the `history/<protocol>.json` format.
pub const HISTORY_SCHEMA_VERSION: &str = "0.2.0";

/// The site URL feeds link to (H8); `CELEBTIR_SITE_URL` overrides it.
pub const DEFAULT_SITE_URL: &str = "https://celebtir.com";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct HistoryDay {
    pub date: String,
    pub tvl_usd: Option<u64>,
    pub fees_usd: Option<u64>,
    pub volume_usd: Option<u64>,
    /// Under the shipped coverage policy (F5).
    pub assessment: crate::coverage::Assessment,
    /// None for an unassessed protocol.
    pub stage: Option<Stage>,
    /// None for an unassessed protocol.
    pub ratings: Option<BTreeMap<Factor, Rating>>,
    /// Charts mark the boundary where this changes (edge case 18).
    pub methodology_version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtocolHistoryFile {
    pub schema_version: String,
    pub protocol: String,
    /// Oldest first; one entry per snapshot the protocol appears in.
    pub days: Vec<HistoryDay>,
    /// Upgrade metrics as of the build date (VAL-5).
    pub upgrades: UpgradeMetrics,
    pub review: ReviewStatus,
}

/// `history/archived.json`: protocols that left coverage (ADR 017).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchivedSiteFile {
    pub schema_version: String,
    /// Sorted by ID.
    pub protocols: Vec<ArchivedSiteEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchivedSiteEntry {
    pub id: String,
    pub name: String,
    pub archived_on: String,
    pub reason: String,
    pub source: String,
    /// The last snapshot day it was rated; None if it never was.
    pub last_known: Option<HistoryDay>,
}

/// REG-7: registry entries reviewed more than 90 days ago show a note.
pub const REVIEW_DUE_DAYS: i64 = 90;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReviewStatus {
    pub last_reviewed: Option<String>,
    /// True when the review is older than 90 days, or has no date.
    pub due: bool,
    /// True while no human has reviewed the entry: the registry names a
    /// draft reviewer or none (H2).
    #[serde(default)]
    pub awaiting_human_review: bool,
}

fn err(msg: String) -> CliError {
    CliError::Validation(msg)
}

/// One day of a token's supply and price (TOK-9).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TokenDay {
    pub date: String,
    pub supply: u64,
    pub decimals: u8,
    pub price_usd_micros: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenHistoryFile {
    pub schema_version: String,
    pub mint: String,
    pub symbol: String,
    pub days: Vec<TokenDay>,
}

/// One day of the summary charts (VAL-8).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SummaryDay {
    pub date: String,
    /// Assessed protocols that day (F5).
    pub protocols: u64,
    /// Protocols listed but not rated that day.
    #[serde(default)]
    pub unassessed: u64,
    /// TVL of the assessed protocols with a known TVL that day.
    pub total_tvl_usd: u64,
    pub value_by_control: celebtir_score::value::ValueRow,
    /// Assessed protocols per Stage; every Stage present.
    pub stages: BTreeMap<Stage, u64>,
    pub methodology_version: String,
    /// The same figures per chain filter (`solana`, `hyperliquid`), so the
    /// site's charts follow the chain picker (HUI-1). Older days without it
    /// were Solana only.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub by_chain: BTreeMap<String, ChainDay>,
}

/// One chain's share of a summary day (HUI-1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChainDay {
    pub protocols: u64,
    pub total_tvl_usd: u64,
    pub assessment: crate::coverage::Assessment,
    /// None when the chain is unassessed.
    pub value_by_control: Option<celebtir_score::value::ValueRow>,
    /// None when the chain is unassessed.
    pub stages: Option<BTreeMap<Stage, u64>>,
}

/// The chain filter a protocol belongs to: HyperEVM and HyperCore are both
/// Hyperliquid.
pub fn chain_filter(chain: celebtir_core::Chain) -> &'static str {
    match chain {
        celebtir_core::Chain::Solana => "solana",
        celebtir_core::Chain::HyperEvm | celebtir_core::Chain::HyperCore => "hyperliquid",
    }
}

fn empty_stages() -> BTreeMap<Stage, u64> {
    [Stage::Unrated, Stage::Stage0, Stage::Stage1, Stage::Stage2]
        .into_iter()
        .map(|s| (s, 0))
        .collect()
}

/// HUI-1: each chain's protocols and TVL for one day; TVL by control and
/// Stages only over assessed protocols, and absent for an unassessed chain.
pub fn by_chain(facts: &FactsFile, scores: &ScoresFile) -> BTreeMap<String, ChainDay> {
    let policy = crate::coverage::policy();
    ["solana", "hyperliquid"]
        .into_iter()
        .map(|chain| {
            let mut scoped = facts.clone();
            scoped
                .protocols
                .retain(|_, p| chain_filter(p.chain) == chain);
            let value = crate::coverage::assessed_value(&scoped, scores, &policy).all;
            let mut stages = empty_stages();
            let mut assessed = 0;
            let mut tvl = 0;
            for (id, p) in &scoped.protocols {
                tvl += facts.metrics.get(id).and_then(|m| m.tvl.usd()).unwrap_or(0);
                if let Some(s) = scores
                    .protocols
                    .get(id)
                    .filter(|_| crate::coverage::is_assessed(id, p, scores, &policy))
                {
                    assessed += 1;
                    *stages.entry(s.stage.stage).or_default() += 1;
                }
            }
            let assessment = if assessed > 0 {
                crate::coverage::Assessment::Assessed
            } else {
                crate::coverage::Assessment::Unassessed
            };
            let rated = assessment == crate::coverage::Assessment::Assessed;
            (
                chain.to_string(),
                ChainDay {
                    protocols: scoped.protocols.len() as u64,
                    total_tvl_usd: if rated { value.total } else { tvl },
                    assessment,
                    value_by_control: rated.then_some(value),
                    stages: rated.then_some(stages),
                },
            )
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SummaryFile {
    pub schema_version: String,
    pub days: Vec<SummaryDay>,
}

/// Snapshot folders (named `YYYY-MM-DD`) holding facts.json and scores.json.
pub fn snapshot_dirs(data: &Path) -> Vec<(String, PathBuf)> {
    let mut out: Vec<(String, PathBuf)> = std::fs::read_dir(data.join("snapshots"))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_str()?.to_string();
            let path = e.path();
            (celebtir_core::date::days(&name).is_some()
                && path.join("facts.json").is_file()
                && path.join("scores.json").is_file())
            .then_some((name, path))
        })
        .collect();
    out.sort();
    out
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, CliError> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| err(format!("cannot read {}: {e}", path.display())))?;
    serde_json::from_str(&text).map_err(|e| err(format!("{}: {e}", path.display())))
}

fn latest_usd(a: &Activity) -> Option<u64> {
    match a {
        Activity::Usd { latest, .. } => Some(*latest),
        Activity::Unavailable { .. } => None,
    }
}

pub fn run(data: &Path, as_of: &str, format: Format) -> Result<(), CliError> {
    let policy = crate::coverage::policy();
    let dirs = snapshot_dirs(data);
    let Some((latest_date, latest_dir)) = dirs.last().cloned() else {
        return Err(err(format!(
            "no snapshots with facts.json and scores.json under {}",
            data.join("snapshots").display()
        )));
    };

    // data/latest/: an exact copy of the newest snapshot.
    let latest = data.join("latest");
    if latest.exists() {
        std::fs::remove_dir_all(&latest)
            .map_err(|e| err(format!("cannot clear {}: {e}", latest.display())))?;
    }
    std::fs::create_dir_all(&latest).map_err(|e| err(e.to_string()))?;
    for entry in std::fs::read_dir(&latest_dir)
        .map_err(|e| err(e.to_string()))?
        .flatten()
    {
        if entry.path().is_file() {
            std::fs::copy(entry.path(), latest.join(entry.file_name()))
                .map_err(|e| err(e.to_string()))?;
        }
    }

    // data/history/<protocol>.json.
    let mut days: BTreeMap<String, Vec<HistoryDay>> = BTreeMap::new();
    let mut newest_facts: Option<FactsFile> = None;
    let mut newest_scores: Option<ScoresFile> = None;
    let mut summary = Vec::new();
    let mut tokens: BTreeMap<String, TokenHistoryFile> = BTreeMap::new();
    let mut observed_upgrades: BTreeMap<
        celebtir_core::Address,
        Vec<celebtir_core::snapshot::UpgradeEvent>,
    > = BTreeMap::new();
    for (date, dir) in &dirs {
        let facts: FactsFile = read_json(&dir.join("facts.json"))?;
        for (program, history) in &facts.program_history {
            let known = observed_upgrades.entry(program.clone()).or_default();
            for event in &history.events {
                if !known.iter().any(|e| {
                    e.signature == event.signature && e.slot == event.slot && e.kind == event.kind
                }) {
                    known.push(event.clone());
                }
            }
        }
        let scores: ScoresFile = read_json(&dir.join("scores.json"))?;
        let mut stages = empty_stages();
        let (mut assessed, mut unassessed) = (0u64, 0u64);
        for (id, p) in &facts.protocols {
            let m = facts.metrics.get(id);
            let a = crate::coverage::assessment(id, p.chain, &scores, &policy);
            let s = scores
                .protocols
                .get(id)
                .filter(|_| a == crate::coverage::Assessment::Assessed);
            match s {
                Some(s) => {
                    assessed += 1;
                    *stages.entry(s.stage.stage).or_default() += 1;
                }
                None => unassessed += 1,
            }
            days.entry(id.clone()).or_default().push(HistoryDay {
                date: date.clone(),
                tvl_usd: m.and_then(|m| m.tvl.usd()),
                fees_usd: m.and_then(|m| latest_usd(&m.fees)),
                volume_usd: m.and_then(|m| latest_usd(&m.volume)),
                assessment: a,
                stage: s.map(|s| s.stage.stage),
                ratings: s.map(|s| s.factors.iter().map(|(f, x)| (*f, x.rating)).collect()),
                methodology_version: scores.methodology_version.clone(),
            });
        }
        let value = crate::coverage::assessed_value(&facts, &scores, &policy);
        summary.push(SummaryDay {
            date: date.clone(),
            protocols: assessed,
            unassessed,
            total_tvl_usd: value.all.total,
            value_by_control: value.all.clone(),
            stages,
            methodology_version: scores.methodology_version.clone(),
            by_chain: by_chain(&facts, &scores),
        });
        for (mint, t) in &facts.tokens {
            let price = match facts.token_metrics.get(mint) {
                Some(celebtir_core::snapshot::TokenMetrics::Price { usd_micros }) => {
                    Some(*usd_micros)
                }
                _ => None,
            };
            tokens
                .entry(mint.to_string())
                .or_insert_with(|| TokenHistoryFile {
                    schema_version: HISTORY_SCHEMA_VERSION.to_string(),
                    mint: mint.to_string(),
                    symbol: t.symbol.clone(),
                    days: Vec::new(),
                })
                .days
                .push(TokenDay {
                    date: date.clone(),
                    supply: t.supply,
                    decimals: t.decimals,
                    price_usd_micros: price,
                });
        }
        newest_facts = Some(facts);
        newest_scores = Some(scores);
    }
    let newest = newest_facts.expect("at least one snapshot");
    let newest_scores = newest_scores.expect("at least one snapshot");

    let history_dir = data.join("history");
    std::fs::create_dir_all(&history_dir).map_err(|e| err(e.to_string()))?;
    // ADR 017: archived protocols with their last known day.
    let archived = ArchivedSiteFile {
        schema_version: HISTORY_SCHEMA_VERSION.to_string(),
        protocols: newest
            .archived
            .iter()
            .map(|(id, a)| ArchivedSiteEntry {
                id: id.clone(),
                name: a.name.clone(),
                archived_on: a.archived_on.clone(),
                reason: a.reason.clone(),
                source: a.source.clone(),
                last_known: days.get(id).and_then(|d| d.last().cloned()),
            })
            .collect(),
    };
    std::fs::write(
        history_dir.join("archived.json"),
        to_canonical_json(&archived).map_err(|e| err(e.to_string()))?,
    )
    .map_err(|e| err(e.to_string()))?;
    let mut history_files: BTreeMap<String, ProtocolHistoryFile> = BTreeMap::new();
    for (id, series) in days {
        let histories: Vec<_> = newest
            .protocols
            .get(&id)
            .map(|p| {
                p.programs
                    .iter()
                    .filter(|(_, f)| f.role == ProgramRole::Core)
                    .map(|(a, _)| {
                        let current = newest.program_history.get(a);
                        celebtir_core::snapshot::ProgramHistory {
                            events: observed_upgrades.get(a).cloned().unwrap_or_default(),
                            scanned: current.map_or(0, |h| h.scanned),
                            total: current.map_or(0, |h| h.total),
                            complete: current.is_some_and(|h| h.complete),
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();
        let file = ProtocolHistoryFile {
            schema_version: HISTORY_SCHEMA_VERSION.to_string(),
            protocol: id.clone(),
            days: series,
            upgrades: upgrade_metrics(&histories.iter().collect::<Vec<_>>(), as_of),
            review: review_status(
                newest
                    .protocols
                    .get(&id)
                    .and_then(|p| p.last_reviewed.clone()),
                newest
                    .protocols
                    .get(&id)
                    .and_then(|p| p.reviewed_by.as_deref()),
                as_of,
            ),
        };
        let text = to_canonical_json(&file).map_err(|e| err(e.to_string()))?;
        std::fs::write(history_dir.join(format!("{id}.json")), text)
            .map_err(|e| err(e.to_string()))?;
        history_files.insert(id, file);
    }

    let text = to_canonical_json(&SummaryFile {
        schema_version: HISTORY_SCHEMA_VERSION.to_string(),
        days: summary,
    })
    .map_err(|e| err(e.to_string()))?;
    std::fs::write(history_dir.join("summary.json"), text).map_err(|e| err(e.to_string()))?;

    // F5: one assessment record the site reads instead of its own rule.
    std::fs::write(
        history_dir.join("coverage.json"),
        to_canonical_json(&crate::coverage::coverage_file(
            &newest,
            &newest_scores,
            &policy,
        ))
        .map_err(|e| err(e.to_string()))?,
    )
    .map_err(|e| err(e.to_string()))?;

    let site_url = std::env::var("CELEBTIR_SITE_URL")
        .ok()
        .filter(|u| !u.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_SITE_URL.to_string());
    let events = write_feeds_at(data, &dirs, site_url.trim_end_matches('/'))?;

    let token_dir = history_dir.join("tokens");
    std::fs::create_dir_all(&token_dir).map_err(|e| err(e.to_string()))?;
    for (mint, file) in &tokens {
        let text = to_canonical_json(file).map_err(|e| err(e.to_string()))?;
        std::fs::write(token_dir.join(format!("{mint}.json")), text)
            .map_err(|e| err(e.to_string()))?;
    }

    // API-1: the public static API.
    crate::api::write(
        data,
        &crate::api::Inputs {
            snapshot_date: &latest_date,
            facts: &newest,
            scores: &newest_scores,
            histories: &history_files,
            tokens: &tokens,
            events: &events,
        },
    )?;

    match format {
        Format::Json => println!(
            "{{\"latest\":\"{latest_date}\",\"snapshots\":{}}}",
            dirs.len()
        ),
        Format::Text => println!(
            "site data from {} snapshots; latest {latest_date} → {}",
            dirs.len(),
            data.display()
        ),
    }
    Ok(())
}

/// Entries per Atom feed (CHG-6).
pub const FEED_ENTRIES: usize = 50;
/// Events kept in `history/events.json` for the change log.
pub const EVENT_LOG_MAX: usize = 2_000;

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// The change-log anchor of an event (H8): the page renders the same id.
pub fn event_anchor(id: &str) -> String {
    id.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

/// An Atom 1.0 feed of `events` (already newest first), with its own URL
/// and a link from every entry to the change log (H8).
fn atom(
    id: &str,
    title: &str,
    self_path: &str,
    site: &str,
    events: &[&celebtir_diff::Event],
) -> String {
    let updated = events
        .first()
        .map_or("1970-01-01".to_string(), |e| e.date.clone());
    let mut out = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<feed xmlns=\"http://www.w3.org/2005/Atom\">\n  <id>tag:celebtir,2026:{}</id>\n  <title>{}</title>\n  <link rel=\"self\" type=\"application/atom+xml\" href=\"{site}{self_path}\"/>\n  <link rel=\"alternate\" type=\"text/html\" href=\"{site}/changelog/\"/>\n  <author><name>Celebtir</name></author>\n  <updated>{updated}T00:00:00Z</updated>\n",
        xml_escape(id),
        xml_escape(title)
    );
    for e in events {
        let mut text = format!("{} ({:?}, {:?})", e.summary, e.severity, e.source);
        if let Some(sig) = &e.signature {
            text.push_str(&format!("; transaction {sig}"));
        }
        out.push_str(&format!(
            "  <entry>\n    <id>tag:celebtir,2026:{}</id>\n    <title>{}</title>\n    <link rel=\"alternate\" type=\"text/html\" href=\"{site}/changelog/#{}\"/>\n    <updated>{}T00:00:00Z</updated>\n    <category term=\"{}\"/>\n    <content type=\"text\">{}</content>\n  </entry>\n",
            xml_escape(&e.id),
            xml_escape(&e.summary),
            event_anchor(&e.id),
            e.date,
            xml_escape(&format!("{:?}", e.severity).to_lowercase()),
            xml_escape(&text),
        ));
    }
    out.push_str("</feed>\n");
    out
}

/// `feeds/all.atom` and `feeds/<protocol>.atom` from every events.json.
/// Writes the change log and Atom feeds; returns every event, newest first.
pub fn write_feeds(
    data: &Path,
    dirs: &[(String, PathBuf)],
) -> Result<Vec<celebtir_diff::Event>, CliError> {
    write_feeds_at(data, dirs, DEFAULT_SITE_URL)
}

/// [`write_feeds`] with the site URL the feeds link to.
pub fn write_feeds_at(
    data: &Path,
    dirs: &[(String, PathBuf)],
    site: &str,
) -> Result<Vec<celebtir_diff::Event>, CliError> {
    let mut events: Vec<celebtir_diff::Event> = Vec::new();
    for (_, dir) in dirs {
        let path = dir.join("events.json");
        if path.is_file() {
            let file: celebtir_diff::EventsFile = read_json(&path)?;
            events.extend(file.events);
        }
    }
    events.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| a.id.cmp(&b.id)));
    // The change log reads this: every event, newest first, capped.
    let recent = celebtir_diff::EventsFile {
        schema_version: celebtir_diff::EVENTS_SCHEMA_VERSION.to_string(),
        date: events.first().map(|e| e.date.clone()).unwrap_or_default(),
        previous_date: String::new(),
        events: events.iter().take(EVENT_LOG_MAX).cloned().collect(),
    };
    std::fs::write(
        data.join("history").join("events.json"),
        to_canonical_json(&recent).map_err(|e| err(e.to_string()))?,
    )
    .map_err(|e| err(e.to_string()))?;
    let feeds = data.join("feeds");
    std::fs::create_dir_all(&feeds).map_err(|e| err(e.to_string()))?;
    let all: Vec<&celebtir_diff::Event> = events.iter().take(FEED_ENTRIES).collect();
    std::fs::write(
        feeds.join("all.atom"),
        atom(
            "all",
            "Celebtir: all changes",
            "/feeds/all.atom",
            site,
            &all,
        ),
    )
    .map_err(|e| err(e.to_string()))?;
    let mut by_protocol: BTreeMap<&str, Vec<&celebtir_diff::Event>> = BTreeMap::new();
    for e in &events {
        if let Some(p) = &e.protocol {
            let list = by_protocol.entry(p).or_default();
            if list.len() < FEED_ENTRIES {
                list.push(e);
            }
        }
    }
    for (p, list) in by_protocol {
        std::fs::write(
            feeds.join(format!("{p}.atom")),
            atom(
                p,
                &format!("Celebtir: changes to {p}"),
                &format!("/feeds/{p}.atom"),
                site,
                &list,
            ),
        )
        .map_err(|e| err(e.to_string()))?;
    }
    Ok(events)
}

fn review_status(
    last_reviewed: Option<String>,
    reviewed_by: Option<&str>,
    as_of: &str,
) -> ReviewStatus {
    use celebtir_core::date::days;
    let due = match (last_reviewed.as_deref().and_then(days), days(as_of)) {
        (Some(r), Some(today)) => today - r > REVIEW_DUE_DAYS,
        _ => true,
    };
    ReviewStatus {
        last_reviewed,
        due,
        awaiting_human_review: reviewed_by.is_none_or(crate::coverage::is_draft_reviewer),
    }
}
