//! REG-6: registry changes merge only 24 hours after the last commit; the
//! check stays pending until then.

use std::path::Path;
use std::process::Command;

const T0: i64 = 1_792_195_200; // 2026-10-17T00:00:00Z

fn git(dir: &Path, args: &[&str], date: i64) {
    let o = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_DATE", format!("@{date} +0000"))
        .env("GIT_COMMITTER_DATE", format!("@{date} +0000"))
        .output()
        .unwrap();
    assert!(
        o.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&o.stderr)
    );
}

fn check(dir: &Path, now: i64) -> String {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/registry-cooling.sh");
    let o = Command::new("bash")
        .arg(script)
        .arg("base")
        .arg(now.to_string())
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    String::from_utf8_lossy(&o.stdout).trim().to_string()
}

fn repo() -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    git(d.path(), &["init", "-q", "-b", "main"], T0);
    git(d.path(), &["config", "user.email", "t@example.com"], T0);
    git(d.path(), &["config", "user.name", "t"], T0);
    std::fs::create_dir_all(d.path().join("registry/solana")).unwrap();
    std::fs::write(d.path().join("registry/solana/a.yaml"), "id: a\n").unwrap();
    std::fs::write(d.path().join("README.md"), "x\n").unwrap();
    git(d.path(), &["add", "-A"], T0);
    git(d.path(), &["commit", "-qm", "base"], T0);
    git(d.path(), &["tag", "base"], T0);
    d
}

#[test]
fn reg_6_registry_change_is_pending_until_24_hours_after_the_last_commit() {
    let d = repo();
    std::fs::write(d.path().join("registry/solana/a.yaml"), "id: a\nname: A\n").unwrap();
    git(d.path(), &["commit", "-qam", "edit"], T0 + 3_600);
    assert_eq!(check(d.path(), T0 + 3_600 + 60), "pending 24");
    assert_eq!(check(d.path(), T0 + 3_600 + 23 * 3_600), "pending 1");
    assert_eq!(check(d.path(), T0 + 3_600 + 24 * 3_600), "success");
}

#[test]
fn reg_6_a_new_commit_restarts_the_cooling_period() {
    let d = repo();
    std::fs::write(d.path().join("registry/solana/a.yaml"), "id: a\nname: A\n").unwrap();
    git(d.path(), &["commit", "-qam", "edit"], T0);
    std::fs::write(d.path().join("registry/solana/a.yaml"), "id: a\nname: B\n").unwrap();
    git(d.path(), &["commit", "-qam", "fix"], T0 + 20 * 3_600);
    assert!(check(d.path(), T0 + 25 * 3_600).starts_with("pending"));
}

#[test]
fn reg_6_changes_outside_the_registry_are_not_held() {
    let d = repo();
    std::fs::write(d.path().join("README.md"), "y\n").unwrap();
    git(d.path(), &["commit", "-qam", "docs"], T0);
    assert_eq!(check(d.path(), T0 + 60), "not-applicable");
}
