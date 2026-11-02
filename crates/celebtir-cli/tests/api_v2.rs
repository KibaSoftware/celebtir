//! API-1, API-2, API-5: `build-site-data` publishes the static API under
//! `data/api/v2/`, every file linked to and valid against its JSON Schema,
//! with a licence field, and the v2 schemas only ever grow. F5 (launch
//! audit): rows carry `chain` and `assessment`; unassessed protocols publish
//! no Stage or ratings. H2: a draft registry reviewer is not published.

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use celebtir_core::snapshot::{Metric, ProtocolMetrics};
use celebtir_core::to_canonical_json;
use common::*;
use serde_json::Value;

fn celebtir(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_celebtir"))
        .args(args)
        .output()
        .unwrap()
}

fn day(root: &Path, date: &str) {
    let dir = root.join("snapshots").join(date);
    std::fs::create_dir_all(&dir).unwrap();
    let mut f = sample();
    // One protocol on a chain v0.3 does not rate, reviewed only by a draft.
    let first = f.protocols.keys().next().unwrap().clone();
    let p = f.protocols.get_mut(&first).unwrap();
    p.chain = celebtir_core::Chain::HyperEvm;
    p.reviewed_by = Some("celebtir-loop-draft".into());
    for id in f.protocols.keys().cloned().collect::<Vec<_>>() {
        f.metrics.insert(
            id,
            ProtocolMetrics {
                defillama_slug: Some("x".into()),
                tvl: Metric::Usd { value: 1_000 },
                asset_groups: Default::default(),
                fees: Default::default(),
                volume: Default::default(),
            },
        );
    }
    std::fs::write(dir.join("facts.json"), to_canonical_json(&f).unwrap()).unwrap();
    let scored = celebtir(&["score", "--facts", dir.join("facts.json").to_str().unwrap()]);
    assert_eq!(scored.status.code(), Some(0));
    std::fs::write(dir.join("run.json"), "{}\n").unwrap();
}

fn built() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    day(root.path(), "2026-10-16");
    day(root.path(), "2026-10-17");
    let o = celebtir(&[
        "build-site-data",
        "--data",
        root.path().to_str().unwrap(),
        "--as-of",
        "2026-10-17",
    ]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    root
}

fn read(p: &Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

/// Every API data file under `dir`, schemas excluded.
fn data_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().unwrap() != "schemas" {
                out.extend(data_files(&p));
            }
        } else if p.extension().is_some_and(|x| x == "json") {
            out.push(p);
        }
    }
    out.sort();
    out
}

#[test]
fn api_1_summary_protocols_tokens_and_events_are_published() {
    let root = built();
    let v1 = root.path().join("api/v2");
    for f in ["summary.json", "events.json"] {
        assert!(v1.join(f).is_file(), "{f}");
    }
    let summary = read(&v1.join("summary.json"));
    let ids: Vec<&str> = summary["protocols"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["id"].as_str().unwrap())
        .collect();
    assert!(!ids.is_empty());
    for id in &ids {
        let p = read(&v1.join("protocols").join(format!("{id}.json")));
        assert_eq!(p["id"], *id);
        assert!(p["facts"]["programs"].is_object());
        if p["assessment"] == "assessed" {
            assert!(p["score"]["factors"].is_object());
        }
    }
    let first = &summary["protocols"][0];
    assert_eq!(
        first["links"]["api"],
        format!("/data/v2/protocols/{}.json", first["id"].as_str().unwrap())
    );
    assert_eq!(
        first["links"]["badge"],
        format!("/badges/{}.svg", first["id"].as_str().unwrap())
    );
    let tokens: Vec<_> = std::fs::read_dir(v1.join("tokens")).unwrap().collect();
    assert!(!tokens.is_empty());
}

#[test]
fn api_1_every_file_links_its_schema_and_validates_against_it() {
    let root = built();
    let v1 = root.path().join("api/v2");
    let files = data_files(&v1);
    assert!(files.len() >= 4);
    for f in files {
        let doc = read(&f);
        let link = doc["$schema"].as_str().expect("$schema link");
        let name = link
            .strip_prefix("/data/v2/schemas/")
            .unwrap_or_else(|| panic!("{}: {link}", f.display()));
        let schema = read(&v1.join("schemas").join(name));
        let validator = jsonschema::validator_for(&schema).unwrap();
        let errors: Vec<String> = validator.iter_errors(&doc).map(|e| e.to_string()).collect();
        assert!(errors.is_empty(), "{}: {errors:?}", f.display());
    }
}

#[test]
fn api_5_every_file_states_its_licence_and_attribution() {
    let root = built();
    for f in data_files(&root.path().join("api/v2")) {
        let doc = read(&f);
        assert_eq!(doc["licence"]["name"], "CC BY 4.0", "{}", f.display());
        assert_eq!(
            doc["licence"]["url"],
            "https://creativecommons.org/licenses/by/4.0/"
        );
        assert!(
            doc["licence"]["attribution"]
                .as_str()
                .unwrap()
                .contains("Celebtir")
        );
    }
}

#[test]
fn api_2_files_carry_version_and_an_empty_deprecation_note() {
    let root = built();
    for f in data_files(&root.path().join("api/v2")) {
        let doc = read(&f);
        assert_eq!(doc["api_version"], "v2", "{}", f.display());
        assert!(doc["deprecation"].is_null(), "{}", f.display());
        assert_eq!(doc["snapshot_date"], "2026-10-17");
    }
}

/// API-2: the published v2 schemas may gain fields but never lose one.
/// Regenerate additive changes with `CELEBTIR_UPDATE_SCHEMAS=1`; a breaking
/// change is refused even then: it belongs in a new /v3/.
#[test]
fn api_2_published_v2_schemas_only_grow() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../schemas/api/v2");
    for (name, schema) in celebtir::api::schemas() {
        let path = dir.join(format!("{name}.schema.json"));
        let generated = to_canonical_json(&schema).unwrap();
        if let Ok(text) = std::fs::read_to_string(&path) {
            let published: Value = serde_json::from_str(&text).unwrap();
            if let Err(e) = celebtir_core::schema::check_compatible(&published, &schema) {
                panic!(
                    "{name}: breaking change to the published v2 schema ({e}); publish it as /v3/ instead"
                );
            }
        }
        if std::env::var_os("CELEBTIR_UPDATE_SCHEMAS").is_some() {
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(&path, &generated).unwrap();
        }
        assert_eq!(
            std::fs::read_to_string(&path).unwrap_or_default(),
            generated,
            "{name}: stale; run with CELEBTIR_UPDATE_SCHEMAS=1"
        );
    }
}

#[test]
fn api_2_removing_a_published_field_is_caught() {
    let (_, summary) = celebtir::api::schemas()
        .into_iter()
        .find(|(n, _)| *n == "summary")
        .unwrap();
    let mut broken = summary.clone();
    broken["properties"]
        .as_object_mut()
        .unwrap()
        .remove("protocols");
    assert!(celebtir_core::schema::check_compatible(&summary, &broken).is_err());
    assert!(celebtir_core::schema::check_compatible(&summary, &summary).is_ok());
}

#[test]
fn aud_5_unassessed_rows_carry_chain_and_no_stage_or_ratings() {
    let root = built();
    let v2 = root.path().join("api/v2");
    let summary = read(&v2.join("summary.json"));
    assert_eq!(summary["assessed_chains"], serde_json::json!(["solana"]));
    let rows = summary["protocols"].as_array().unwrap();
    assert!(rows.iter().all(|r| r.get("chain").is_some()));
    let evm: Vec<&Value> = rows.iter().filter(|r| r["chain"] == "hyper_evm").collect();
    assert_eq!(evm.len(), 1);
    let r = evm[0];
    assert_eq!(r["assessment"], "unassessed");
    assert!(r["stage"].is_null() && r["ratings"].is_null() && r["control_class"].is_null());
    assert!(r["controller_summary"].is_string());
    let p = read(
        &v2.join("protocols")
            .join(format!("{}.json", r["id"].as_str().unwrap())),
    );
    assert!(p["score"].is_null());
    assert!(
        p["history"]
            .as_array()
            .unwrap()
            .iter()
            .all(|d| d["stage"].is_null())
    );
    for r in rows.iter().filter(|r| r["chain"] == "solana") {
        assert_eq!(r["assessment"], "assessed");
        assert!(r["stage"].is_string());
    }
    // No older API version survives the rebuild (D7).
    assert!(!root.path().join("api/v1").exists());
}

#[test]
fn aud_19_draft_reviewer_is_withheld() {
    let root = built();
    for f in std::fs::read_dir(root.path().join("api/v2/protocols"))
        .unwrap()
        .flatten()
    {
        let p = read(&f.path());
        let by = p["facts"]["reviewed_by"].as_str().unwrap_or_default();
        assert!(!by.contains("draft"), "{}", f.path().display());
        if p["chain"] == "hyper_evm" {
            assert!(p["facts"]["reviewed_by"].is_null());
            assert_eq!(p["review"]["awaiting_human_review"], true);
        }
    }
}

#[test]
fn aud_5_coverage_record_lists_every_protocol_with_its_assessment() {
    let root = built();
    let c = read(&root.path().join("history/coverage.json"));
    assert_eq!(c["assessed_chains"], serde_json::json!(["solana"]));
    let protocols = c["protocols"].as_object().unwrap();
    assert!(
        protocols
            .values()
            .any(|p| p["assessment"] == "unassessed" && p["reason"].is_string())
    );
    assert!(
        protocols
            .values()
            .any(|p| p["assessment"] == "assessed" && p["reason"].is_null())
    );
}

#[test]
fn aud_22_atom_feeds_link_themselves_and_the_change_log() {
    let root = built();
    let feed = std::fs::read_to_string(root.path().join("feeds/all.atom")).unwrap();
    assert!(feed.contains(r#"<link rel="self" type="application/atom+xml" href="https://celebtir.com/feeds/all.atom"/>"#), "{feed}");
    assert!(feed.contains(
        r#"<link rel="alternate" type="text/html" href="https://celebtir.com/changelog/"/>"#
    ));
    assert!(feed.contains("<author><name>Celebtir</name></author>"));
    for e in feed.split("<entry>").skip(1) {
        assert!(
            e.contains(
                r#"<link rel="alternate" type="text/html" href="https://celebtir.com/changelog/#"#
            ),
            "{e}"
        );
    }
}
