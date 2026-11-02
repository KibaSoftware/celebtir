//! SCO-1 (no I/O, no clock) and SCO-2 (every threshold lives in the
//! methodology file).

use std::path::{Path, PathBuf};

use celebtir_score::{Methodology, MethodologyError, SHIPPED_METHODOLOGY};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The scorer embeds its own copy of the methodology so the crate packages
/// for crates.io (CLI-5); it must be the published file, byte for byte.
#[test]
fn sco_2_embedded_methodology_is_the_published_file() {
    let published =
        std::fs::read_to_string(crate_dir().join("../../methodology/v0.3.toml")).unwrap();
    assert_eq!(SHIPPED_METHODOLOGY, published);
    // Every bundled replay copy matches its published file too (SNAP-4).
    for v in ["0.1", "0.2", "0.3"] {
        let file =
            std::fs::read_to_string(crate_dir().join(format!("../../methodology/v{v}.toml")))
                .unwrap();
        let bundled =
            std::fs::read_to_string(crate_dir().join(format!("methodology/v{v}.toml"))).unwrap();
        assert_eq!(file, bundled, "v{v}");
        assert_eq!(
            Methodology::bundled(v).unwrap(),
            Methodology::from_toml(&file).unwrap()
        );
    }
}

#[test]
fn sco_2_shipped_methodology_parses_and_is_consistent() {
    let m = Methodology::shipped();
    assert_eq!(m.version, "0.3");
    assert!(m.at_least(0, 3) && m.at_least(0, 2) && !m.at_least(0, 4));
    assert_eq!(m.value_classes.as_ref().unwrap().delayed_min_secs, 86_400);
    m.check().unwrap();
    assert_eq!(m.upgrade_delay.low_min_secs, 5 * 24 * 3600);
    assert_eq!(m.upgrade_control.low_min_voters, 5);
    assert_eq!(m.upgrade_control.low_min_threshold_pct, 60);
}

#[test]
fn sco_2_unknown_key_is_rejected() {
    let text = SHIPPED_METHODOLOGY.replace("[upgrade_delay]", "[upgrade_delay]\nsurprise = 1");
    assert!(matches!(
        Methodology::from_toml(&text),
        Err(MethodologyError::Parse(_))
    ));
}

#[test]
fn sco_2_missing_threshold_is_rejected() {
    let text: String = SHIPPED_METHODOLOGY
        .lines()
        .filter(|l| !l.starts_with("medium_min_secs"))
        .map(|l| format!("{l}\n"))
        .collect();
    assert!(matches!(
        Methodology::from_toml(&text),
        Err(MethodologyError::Parse(_))
    ));
}

#[test]
fn sco_2_low_must_be_stricter_than_medium() {
    let text = SHIPPED_METHODOLOGY.replace("low_min_secs = 432000", "low_min_secs = 3600");
    let m = Methodology::from_toml(&text).unwrap();
    assert!(matches!(m.check(), Err(MethodologyError::Inconsistent(_))));
}

#[test]
fn sco_2_percentages_above_100_are_rejected() {
    let text =
        SHIPPED_METHODOLOGY.replace("low_min_threshold_pct = 60", "low_min_threshold_pct = 160");
    let m = Methodology::from_toml(&text).unwrap();
    assert!(matches!(m.check(), Err(MethodologyError::Inconsistent(_))));
}

/// SCO-1: the scorer must build where network and filesystem are denied, so
/// its source may not touch either, nor the clock.
#[test]
fn sco_1_scorer_source_has_no_io_or_clock() {
    const FORBIDDEN: &[&str] = &[
        "std::fs",
        "std::net",
        "std::env",
        "std::process",
        "SystemTime",
        "Instant::now",
        "std::io::stdin",
        "File::open",
        "read_to_string(",
    ];
    let mut offenders = Vec::new();
    for file in rust_files(&crate_dir().join("src")) {
        let text = std::fs::read_to_string(&file).unwrap();
        for f in FORBIDDEN {
            if text.contains(f) {
                offenders.push(format!("{}: {f}", file.display()));
            }
        }
    }
    assert!(offenders.is_empty(), "{offenders:#?}");
}

#[test]
fn sco_1_scorer_dependencies_are_pure() {
    const ALLOWED: &[&str] = &["celebtir-core", "schemars", "serde", "thiserror", "toml"];
    let manifest = std::fs::read_to_string(crate_dir().join("Cargo.toml")).unwrap();
    let deps = manifest
        .split("[dependencies]")
        .nth(1)
        .unwrap()
        .split("\n[")
        .next()
        .unwrap();
    for line in deps
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        let name = line.split(['=', '.']).next().unwrap().trim();
        assert!(
            ALLOWED.contains(&name),
            "scorer depends on `{name}`, which is not allowed (SCO-1)"
        );
    }
}

fn rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let p = entry.path();
        if p.is_dir() {
            out.extend(rust_files(&p));
        } else if p.extension().is_some_and(|e| e == "rs") {
            out.push(p);
        }
    }
    out
}
