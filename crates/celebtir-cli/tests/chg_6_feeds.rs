//! CHG-6: an Atom feed of all events and one per protocol, listing the
//! latest 50 events.

use std::path::Path;
use std::process::Command;

use quick_xml::Reader;
use quick_xml::events::Event as Xml;

fn event(id: &str, date: &str, protocol: &str, summary: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id, "date": date, "detected_within_days": 1, "protocol": protocol, "subject": "x",
        "kind": "controller_changed", "severity": "critical", "source": "on_chain",
        "summary": summary, "before": null, "after": null, "evidence": [], "signature": null
    })
}

fn snapshot(root: &Path, date: &str, events: Vec<serde_json::Value>) {
    let d = root.join("snapshots").join(date);
    std::fs::create_dir_all(&d).unwrap();
    let facts = celebtir_core::snapshot::FactsFile::new(Default::default());
    std::fs::write(
        d.join("facts.json"),
        celebtir_core::to_canonical_json(&facts).unwrap(),
    )
    .unwrap();
    let scores = celebtir_score::score(&facts, &celebtir_score::Methodology::shipped());
    std::fs::write(
        d.join("scores.json"),
        celebtir_core::to_canonical_json(&scores).unwrap(),
    )
    .unwrap();
    let file = serde_json::json!({"schema_version": "0.1.0", "date": date, "previous_date": date, "events": events});
    std::fs::write(d.join("events.json"), serde_json::to_string(&file).unwrap()).unwrap();
}

/// Element names in document order, after checking the XML is well formed.
fn elements(xml: &str) -> Vec<String> {
    let mut r = Reader::from_str(xml);
    let mut out = Vec::new();
    loop {
        match r.read_event().expect("well-formed XML") {
            Xml::Start(e) | Xml::Empty(e) => out.push(e.name().as_ref().to_string()),
            Xml::Eof => break,
            _ => {}
        }
    }
    out
}

#[test]
fn chg_6_feeds_validate_and_list_the_latest_50_events() {
    let root = tempfile::tempdir().unwrap();
    for day in 0..30 {
        let date = celebtir_core::date::ymd(
            (celebtir_core::date::days("2026-09-18").unwrap() + day) * 86_400,
        );
        let events = (0..2)
            .map(|i| {
                event(
                    &format!("{date}:p{i}:controller_changed:x"),
                    &date,
                    &format!("p{i}"),
                    "authority <changed> & more",
                )
            })
            .collect();
        snapshot(root.path(), &date, events);
    }
    let o = Command::new(env!("CARGO_BIN_EXE_celebtir"))
        .args([
            "build-site-data",
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

    let all = std::fs::read_to_string(root.path().join("feeds/all.atom")).unwrap();
    let names = elements(&all);
    assert_eq!(names[0], "feed");
    for required in ["id", "title", "updated"] {
        assert!(
            names.iter().any(|n| n == required),
            "feed lacks <{required}>"
        );
    }
    assert_eq!(
        names.iter().filter(|n| *n == "entry").count(),
        50,
        "latest 50 of 60"
    );
    assert!(
        all.contains("authority &lt;changed&gt; &amp; more"),
        "text is escaped"
    );
    assert!(
        all.contains("<updated>2026-10-17T00:00:00Z</updated>"),
        "newest first"
    );

    let log: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.path().join("history/events.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        log["events"].as_array().unwrap().len(),
        60,
        "the change log keeps every event"
    );
    assert_eq!(log["events"][0]["date"], "2026-10-17");

    let p0 = std::fs::read_to_string(root.path().join("feeds/p0.atom")).unwrap();
    assert_eq!(elements(&p0).iter().filter(|n| *n == "entry").count(), 30);
}
