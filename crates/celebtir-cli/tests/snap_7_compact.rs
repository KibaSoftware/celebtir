//! SNAP-7: snapshots older than 180 days are compacted to weekly, keeping
//! every day's events.json.

use std::path::Path;
use std::process::Command;

fn day(root: &Path, date: &str) {
    let d = root.join("snapshots").join(date);
    std::fs::create_dir_all(&d).unwrap();
    for f in ["facts.json", "scores.json", "run.json", "events.json"] {
        std::fs::write(d.join(f), "{}\n").unwrap();
    }
}

#[test]
fn snap_7_old_days_compact_to_one_per_week_keeping_events() {
    let root = tempfile::tempdir().unwrap();
    // 2026-03-02 is a Monday; 14 days = two full weeks, all older than 180 days
    // before 2026-10-17. Plus two recent days.
    let old: Vec<String> = (2..=15).map(|d| format!("2026-03-{d:02}")).collect();
    for d in &old {
        day(root.path(), d);
    }
    day(root.path(), "2026-10-16");
    day(root.path(), "2026-10-17");

    let o = Command::new(env!("CARGO_BIN_EXE_celebtir"))
        .args([
            "compact",
            "--data",
            root.path().to_str().unwrap(),
            "--as-of",
            "2026-10-17",
        ])
        .output()
        .unwrap();
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );

    let full = |d: &str| {
        root.path()
            .join("snapshots")
            .join(d)
            .join("facts.json")
            .exists()
    };
    let events = |d: &str| {
        root.path()
            .join("snapshots")
            .join(d)
            .join("events.json")
            .exists()
    };
    // The newest day of each week keeps its full snapshot (Sundays 03-08, 03-15).
    let kept: Vec<&String> = old.iter().filter(|d| full(d)).collect();
    assert_eq!(kept, vec!["2026-03-08", "2026-03-15"]);
    assert!(
        old.iter().all(|d| events(d)),
        "every day's events.json is kept"
    );
    assert!(
        full("2026-10-16") && full("2026-10-17"),
        "recent days are untouched"
    );
    for d in old.iter().filter(|d| !full(d)) {
        let dir = root.path().join("snapshots").join(d);
        assert!(!dir.join("scores.json").exists() && !dir.join("run.json").exists());
    }

    // Running it again changes nothing.
    let again = Command::new(env!("CARGO_BIN_EXE_celebtir"))
        .args([
            "compact",
            "--data",
            root.path().to_str().unwrap(),
            "--as-of",
            "2026-10-17",
        ])
        .output()
        .unwrap();
    assert_eq!(again.status.code(), Some(0));
    assert_eq!(old.iter().filter(|d| full(d)).count(), 2);
}
