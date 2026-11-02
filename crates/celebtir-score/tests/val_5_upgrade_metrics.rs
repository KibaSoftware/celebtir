//! VAL-5: upgrade metrics per protocol: count, last upgrade, average
//! interval, upgrades in the last 30 days.

use celebtir_core::snapshot::{ProgramHistory, UpgradeEvent, UpgradeKind};
use celebtir_score::upgrades::{UpgradeMetrics, upgrade_metrics};

fn ev(date: Option<&str>, slot: u64, kind: UpgradeKind) -> UpgradeEvent {
    UpgradeEvent {
        signature: format!("s{slot}"),
        slot,
        date: date.map(str::to_string),
        kind,
        new_authority: None,
    }
}

/// Kamino Lend's 5 newest upgrades as recorded in the golden run.
fn kamino() -> ProgramHistory {
    ProgramHistory {
        events: [
            "2026-05-12",
            "2026-05-14",
            "2026-05-28",
            "2026-08-12",
            "2026-08-20",
        ]
        .iter()
        .enumerate()
        .map(|(i, d)| ev(Some(d), 400 + i as u64, UpgradeKind::Upgrade))
        .collect(),
        scanned: 5,
        total: 90,
        complete: false,
    }
}

#[test]
fn val_5_values_match_a_hand_count() {
    // Hand count: 5 upgrades; last 2026-08-20; gaps 2, 14, 76, 8 days → 25.0;
    // none in the 30 days up to 2026-10-17.
    let m = upgrade_metrics(&[&kamino()], "2026-10-17");
    assert_eq!(
        m,
        UpgradeMetrics {
            upgrades: 5,
            last_upgrade_date: Some("2026-08-20".into()),
            last_upgrade_slot: Some(404),
            average_interval_days: Some(25.0),
            upgrades_last_30_days: 0,
            complete: false,
        }
    );
    let earlier = upgrade_metrics(&[&kamino()], "2026-09-01");
    assert_eq!(
        earlier.upgrades_last_30_days, 2,
        "2026-08-12 and 2026-08-20"
    );
}

#[test]
fn val_5_combines_core_programs_and_counts_only_upgrades() {
    let a = ProgramHistory {
        events: vec![
            ev(Some("2026-01-01"), 1, UpgradeKind::Deploy),
            ev(Some("2026-01-11"), 2, UpgradeKind::Upgrade),
        ],
        scanned: 2,
        total: 2,
        complete: true,
    };
    let b = ProgramHistory {
        events: vec![
            ev(Some("2026-01-21"), 3, UpgradeKind::Upgrade),
            ev(Some("2026-01-22"), 4, UpgradeKind::SetAuthority),
        ],
        scanned: 2,
        total: 2,
        complete: true,
    };
    let m = upgrade_metrics(&[&a, &b], "2026-01-25");
    assert_eq!(m.upgrades, 2);
    assert_eq!(m.average_interval_days, Some(10.0));
    assert_eq!(m.upgrades_last_30_days, 2);
    assert!(m.complete);
}

#[test]
fn val_5_upgrades_without_dates_count_but_do_not_enter_intervals() {
    let h = ProgramHistory {
        events: vec![
            ev(None, 10, UpgradeKind::Upgrade),
            ev(Some("2026-03-01"), 20, UpgradeKind::Upgrade),
        ],
        scanned: 2,
        total: 2,
        complete: true,
    };
    let m = upgrade_metrics(&[&h], "2026-03-10");
    assert_eq!(m.upgrades, 2);
    assert_eq!(
        m.average_interval_days, None,
        "one dated upgrade has no interval"
    );
    assert_eq!(m.last_upgrade_slot, Some(20));
}

#[test]
fn val_5_no_upgrades_and_no_history() {
    let m = upgrade_metrics(&[], "2026-03-10");
    assert_eq!(m.upgrades, 0);
    assert_eq!(m.last_upgrade_date, None);
    assert!(!m.complete, "no history read is not a complete history");
}

#[test]
fn val_5_dates_round_trip() {
    for d in ["1970-01-01", "2000-02-29", "2026-10-17", "2024-12-31"] {
        let n = celebtir_core::date::days(d).unwrap();
        assert_eq!(celebtir_core::date::ymd(n * 86_400), d);
    }
    assert_eq!(celebtir_core::date::days("2026-02-30"), None);
    assert_eq!(celebtir_core::date::days("nope"), None);
}
