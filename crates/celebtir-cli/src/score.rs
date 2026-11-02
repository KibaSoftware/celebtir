//! `celebtir score` and `celebtir explain`: the only file I/O around the pure
//! scorer.

use std::fmt::Write as _;
use std::path::Path;

use celebtir_core::snapshot::FactsFile;
use celebtir_core::to_canonical_json;
use celebtir_score::{Factor, Methodology, ProtocolScore, Rating, ScoresFile, Stage, factors};

use crate::{CliError, Format};

fn load_methodology(path: Option<&Path>) -> Result<Methodology, CliError> {
    let Some(path) = path else {
        return Ok(Methodology::shipped());
    };
    let text = std::fs::read_to_string(path)
        .map_err(|e| CliError::Validation(format!("cannot read {}: {e}", path.display())))?;
    let m = Methodology::from_toml(&text).map_err(|e| CliError::Validation(e.to_string()))?;
    m.check().map_err(|e| CliError::Validation(e.to_string()))?;
    Ok(m)
}

fn load_facts(path: &Path) -> Result<FactsFile, CliError> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| CliError::Validation(format!("cannot read {}: {e}", path.display())))?;
    serde_json::from_str(&text).map_err(|e| {
        CliError::Validation(format!("{} is not a valid facts file: {e}", path.display()))
    })
}

fn json(value: &impl serde::Serialize) -> Result<String, CliError> {
    to_canonical_json(value).map_err(|e| CliError::Validation(e.to_string()))
}

pub fn score(
    facts_path: &Path,
    out: Option<&Path>,
    methodology: Option<&Path>,
    format: Format,
) -> Result<(), CliError> {
    let m = load_methodology(methodology)?;
    let facts = load_facts(facts_path)?;
    let scores = celebtir_score::score(&facts, &m);
    let text = json(&scores)?;

    let out = out.map(Path::to_path_buf).unwrap_or_else(|| {
        facts_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("scores.json")
    });
    std::fs::write(&out, &text)
        .map_err(|e| CliError::Validation(format!("cannot write {}: {e}", out.display())))?;

    match format {
        Format::Json => print!("{text}"),
        Format::Text => {
            for (id, p) in &scores.protocols {
                let pills: Vec<String> = Factor::ALL
                    .iter()
                    .map(|f| p.factors[f].rating.label().to_string())
                    .collect();
                println!("{id:<24} {:<8} {}", p.stage.stage.label(), pills.join(" "));
            }
            println!(
                "scored {} protocols and {} tokens with methodology {} → {}",
                scores.protocols.len(),
                scores.tokens.len(),
                scores.methodology_version,
                out.display()
            );
        }
    }
    Ok(())
}

pub fn explain(
    id: &str,
    facts_path: &Path,
    methodology: Option<&Path>,
    format: Format,
) -> Result<(), CliError> {
    let m = load_methodology(methodology)?;
    let mut facts = load_facts(facts_path)?;
    if !facts.protocols.contains_key(id) {
        let known: Vec<&str> = facts.protocols.keys().map(String::as_str).collect();
        return Err(CliError::Validation(format!(
            "no protocol `{id}` in {}; known: {}",
            facts_path.display(),
            known.join(", ")
        )));
    }
    facts.protocols.retain(|k, _| k == id);
    let scores = celebtir_score::score(&facts, &m);
    match format {
        // A scores file holding only this protocol, so it validates against
        // the published scores schema (CLI-3).
        Format::Json => print!("{}", json(&scores)?),
        Format::Text => print!("{}", render(id, &scores.protocols[id], &scores, &facts)),
    }
    Ok(())
}

/// `382114880` → `382,114,880`.
fn thousands(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

const INDENT: &str = "                          ";

fn render(id: &str, p: &ProtocolScore, scores: &ScoresFile, facts: &FactsFile) -> String {
    let max_slot = p
        .factors
        .values()
        .flat_map(|f| f.evidence.iter().map(|e| e.slot))
        .max()
        .unwrap_or(0);
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{id}  ·  {}  ·  slot {}  ·  methodology {}\n",
        p.stage.stage.label(),
        thousands(max_slot),
        scores.methodology_version
    );
    for f in Factor::ALL {
        let s = &p.factors[&f];
        let _ = writeln!(out, "{:<18}{:<8}{}", f.title(), s.rating.label(), s.reason);
        let mut seen = Vec::new();
        for e in &s.evidence {
            let key = (e.address.clone(), e.field.clone());
            if seen.contains(&key) {
                continue;
            }
            seen.push(key);
            let _ = writeln!(
                out,
                "{INDENT}evidence: {} {} @ {}",
                e.field,
                e.address,
                thousands(e.slot)
            );
        }
        if s.rating != Rating::Low {
            let _ = writeln!(out, "{INDENT}next: {}", s.next_step);
        }
    }

    if !p.risks.is_empty() {
        let _ = writeln!(out, "\nRisks");
        for r in &p.risks {
            let _ = writeln!(out, "  [{:?}] {}", r.severity, r.text);
        }
    }

    let protocol_facts = &facts.protocols[id];
    let periphery: Vec<String> = p
        .programs
        .iter()
        .filter(|(a, _)| {
            protocol_facts
                .programs
                .get(*a)
                .is_some_and(|f| f.role == celebtir_core::snapshot::ProgramRole::Periphery)
        })
        .map(|(a, s)| {
            format!(
                "{a} ({}: {})",
                s.upgrade_control.rating.label(),
                factors::describe(protocol_facts.programs[a].upgrade_control.value())
            )
        })
        .collect();
    if !periphery.is_empty() {
        let _ = writeln!(
            out,
            "\nPeriphery programs (shown, not counted): {}",
            periphery.join("; ")
        );
    }

    for (mint, t) in &p.tokens {
        if let Some(w) = &t.warning {
            let _ = writeln!(out, "\nWarning: {w} ({mint})");
        }
    }

    match p.stage.next {
        Some(next) if !p.stage.missing.is_empty() => {
            let _ = writeln!(out, "\nMissing for {}:", next.label());
            for r in &p.stage.missing {
                let _ = writeln!(out, "  - {}: {}", r.description, r.detail);
            }
        }
        _ if p.stage.stage == Stage::Stage2 => {
            let _ = writeln!(out, "\nStage 2: every requirement met.");
        }
        _ => {}
    }
    out
}

fn load_scores(path: &Path) -> Result<ScoresFile, CliError> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| CliError::Validation(format!("cannot read {}: {e}", path.display())))?;
    serde_json::from_str(&text).map_err(|e| {
        CliError::Validation(format!(
            "{} is not a valid scores file: {e}",
            path.display()
        ))
    })
}

fn folder_date(dir: &Path) -> Result<String, CliError> {
    let name = dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_string();
    celebtir_core::date::days(&name)
        .map(|_| name.clone())
        .ok_or_else(|| {
            CliError::Validation(format!(
                "{} is not a snapshot folder named YYYY-MM-DD",
                dir.display()
            ))
        })
}

pub fn diff(old: &Path, new: &Path, format: Format) -> Result<(), CliError> {
    let (old_date, new_date) = (folder_date(old)?, folder_date(new)?);
    let (of, nf) = (
        load_facts(&old.join("facts.json"))?,
        load_facts(&new.join("facts.json"))?,
    );
    let (os, ns) = (
        load_scores(&old.join("scores.json"))?,
        load_scores(&new.join("scores.json"))?,
    );
    let old_snap = celebtir_diff::Snapshot {
        date: &old_date,
        facts: &of,
        scores: &os,
    };
    let new_snap = celebtir_diff::Snapshot {
        date: &new_date,
        facts: &nf,
        scores: &ns,
    };
    let reported = reported_before(new, &new_date);
    let mut events = celebtir_diff::diff_with(&old_snap, &new_snap, &reported);

    // Noise control (CHG-3) looks back over earlier snapshot folders next to `old`.
    let earlier = earlier_snapshots(old, &old_date, celebtir_diff::DEPENDENCY_RUNS - 1);
    let mut window: Vec<celebtir_diff::Snapshot<'_>> = earlier
        .iter()
        .map(|(d, f, s)| celebtir_diff::Snapshot {
            date: d,
            facts: f,
            scores: s,
        })
        .collect();
    window.push(old_snap);
    window.push(new_snap);
    events.extend(celebtir_diff::confirmed_events(&window));
    events.sort_by(|a, b| a.id.cmp(&b.id));
    let file = celebtir_diff::EventsFile {
        schema_version: celebtir_diff::EVENTS_SCHEMA_VERSION.to_string(),
        date: new_date,
        previous_date: old_date,
        events,
    };
    let text = json(&file)?;
    let out = new.join("events.json");
    std::fs::write(&out, &text)
        .map_err(|e| CliError::Validation(format!("cannot write {}: {e}", out.display())))?;
    match format {
        Format::Json => print!("{text}"),
        Format::Text => {
            for e in &file.events {
                println!("{:?}  {:?}  {}", e.severity, e.source, e.summary);
            }
            println!("{} events → {}", file.events.len(), out.display());
        }
    }
    Ok(())
}

/// `(protocol, signature)` of every transaction event in event files of
/// snapshot folders dated before `before`, next to `new` (F6 catch-up).
fn reported_before(new: &Path, before: &str) -> celebtir_diff::Reported {
    let Some(parent) = new.parent() else {
        return Default::default();
    };
    let mut out = celebtir_diff::Reported::new();
    for e in std::fs::read_dir(parent).into_iter().flatten().flatten() {
        let Some(name) = e.file_name().to_str().map(str::to_string) else {
            continue;
        };
        if celebtir_core::date::days(&name).is_none() || name.as_str() >= before {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(e.path().join("events.json")) else {
            continue;
        };
        let Ok(file) = serde_json::from_str::<celebtir_diff::EventsFile>(&text) else {
            continue;
        };
        for ev in file.events {
            if let (Some(p), Some(s)) = (ev.protocol, ev.signature) {
                out.insert((p, s));
            }
        }
    }
    out
}

/// `diff --initial`: the first snapshot's events (F6).
pub fn diff_initial(new: &Path, format: Format) -> Result<(), CliError> {
    let date = folder_date(new)?;
    let facts = load_facts(&new.join("facts.json"))?;
    let scores = load_scores(&new.join("scores.json"))?;
    let snap = celebtir_diff::Snapshot {
        date: &date,
        facts: &facts,
        scores: &scores,
    };
    let file = celebtir_diff::EventsFile {
        schema_version: celebtir_diff::EVENTS_SCHEMA_VERSION.to_string(),
        date: date.clone(),
        previous_date: String::new(),
        events: celebtir_diff::initial_events(&snap),
    };
    let text = json(&file)?;
    let out = new.join("events.json");
    std::fs::write(&out, &text)
        .map_err(|e| CliError::Validation(format!("cannot write {}: {e}", out.display())))?;
    match format {
        Format::Json => print!("{text}"),
        Format::Text => println!("{} events → {}", file.events.len(), out.display()),
    }
    Ok(())
}

/// Up to `count` snapshot folders dated before `before`, next to `old`,
/// oldest first. Folders that fail to load are skipped.
fn earlier_snapshots(
    old: &Path,
    before: &str,
    count: usize,
) -> Vec<(String, FactsFile, ScoresFile)> {
    let Some(parent) = old.parent() else {
        return Vec::new();
    };
    let mut dates: Vec<String> = std::fs::read_dir(parent)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| e.file_name().to_str().map(str::to_string))
        .filter(|n| celebtir_core::date::days(n).is_some() && n.as_str() < before)
        .collect();
    dates.sort();
    let mut out: Vec<(String, FactsFile, ScoresFile)> = dates
        .into_iter()
        .rev()
        .take(count)
        .filter_map(|d| {
            let dir = parent.join(&d);
            Some((
                d,
                load_facts(&dir.join("facts.json")).ok()?,
                load_scores(&dir.join("scores.json")).ok()?,
            ))
        })
        .collect();
    out.reverse();
    out
}
