//! Bakes the commit into `--version` (CLI-5).
//!
//! A crate installed from crates.io carries `.cargo_vcs_info.json`, written by
//! `cargo package` with the commit it was packaged from; a checkout asks git.

use std::path::Path;
use std::process::Command;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let commit = packaged_commit(Path::new(&manifest_dir))
        .or_else(git_commit)
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=CELEBTIR_COMMIT={commit}");
}

/// `{"git": {"sha1": "<40 hex>"}, …}`; no JSON parser needed for one field.
fn packaged_commit(dir: &Path) -> Option<String> {
    let path = dir.join(".cargo_vcs_info.json");
    println!("cargo:rerun-if-changed={}", path.display());
    let text = std::fs::read_to_string(path).ok()?;
    let at = text.find("\"sha1\"")?;
    let rest = &text[at + "\"sha1\"".len()..];
    let start = rest.find('"')? + 1;
    let end = start + rest[start..].find('"')?;
    Some(rest[start..end].chars().take(12).collect())
}

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn git_commit() -> Option<String> {
    let commit = git(&["rev-parse", "--short=12", "HEAD"])?;
    // Rebuild when HEAD moves: a new commit on the branch or a checkout.
    for path in ["HEAD", "packed-refs"] {
        if let Some(p) = git(&["rev-parse", "--path-format=absolute", "--git-path", path]) {
            println!("cargo:rerun-if-changed={p}");
        }
    }
    if let Some(branch) = git(&["symbolic-ref", "-q", "HEAD"])
        && let Some(p) = git(&["rev-parse", "--path-format=absolute", "--git-path", &branch])
    {
        println!("cargo:rerun-if-changed={p}");
    }
    Some(commit)
}
