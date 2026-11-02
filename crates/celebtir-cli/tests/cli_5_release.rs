//! CLI-5: `--version` prints the commit, and every published crate can be
//! packaged for crates.io on its own.

mod common;

use std::path::Path;
use std::process::Command;

use common::repo;

fn git_head() -> Option<String> {
    let out = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo())
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8(out.stdout).unwrap().trim().to_string())
}

#[test]
fn cli_5_version_prints_the_commit() {
    let out = Command::new(env!("CARGO_BIN_EXE_celebtir"))
        .arg("--version")
        .output()
        .unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    let text = text.trim();
    let prefix = format!("celebtir {} (commit ", env!("CARGO_PKG_VERSION"));
    assert!(
        text.starts_with(&prefix) && text.ends_with(')'),
        "unexpected --version output: {text}"
    );
    let commit = &text[prefix.len()..text.len() - 1];
    assert!(
        commit.len() >= 12 && commit.chars().all(|c| c.is_ascii_hexdigit()),
        "not a commit hash: {commit}"
    );
    if let Some(head) = git_head() {
        assert!(head.starts_with(commit), "{commit} is not HEAD ({head})");
    }
}

/// The crates `cargo install celebtir` pulls in: the CLI and its path
/// dependencies.
fn published_crates() -> Vec<(String, toml::Table)> {
    let mut crates = Vec::new();
    for entry in std::fs::read_dir(repo().join("crates")).unwrap() {
        let dir = entry.unwrap().path();
        let manifest: toml::Table = std::fs::read_to_string(dir.join("Cargo.toml"))
            .unwrap()
            .parse()
            .unwrap();
        let package = manifest["package"].as_table().unwrap();
        if package.get("publish").and_then(|p| p.as_bool()) == Some(false) {
            continue;
        }
        crates.push((
            dir.file_name().unwrap().to_string_lossy().into_owned(),
            manifest,
        ));
    }
    crates.sort_by(|a, b| a.0.cmp(&b.0));
    crates
}

#[test]
fn cli_5_published_crates_have_crates_io_metadata() {
    let names: Vec<_> = published_crates().into_iter().map(|(d, _)| d).collect();
    assert!(names.contains(&"celebtir-cli".to_string()), "{names:?}");
    for (dir, manifest) in published_crates() {
        let package = manifest["package"].as_table().unwrap();
        for key in ["description", "license", "repository", "readme"] {
            assert!(package.contains_key(key), "crates/{dir}: no `{key}`");
        }
        let readme = package["readme"].as_str().unwrap();
        assert!(
            repo().join("crates").join(&dir).join(readme).is_file(),
            "crates/{dir}: readme {readme} is missing"
        );
    }
}

#[test]
fn cli_5_workspace_path_dependencies_carry_a_version() {
    let manifest: toml::Table = std::fs::read_to_string(repo().join("Cargo.toml"))
        .unwrap()
        .parse()
        .unwrap();
    let deps = manifest["workspace"]["dependencies"].as_table().unwrap();
    for (name, dep) in deps {
        if let Some(dep) = dep.as_table()
            && dep.contains_key("path")
        {
            assert!(
                dep.contains_key("version"),
                "workspace dependency {name} has a path but no version, so it cannot be published"
            );
        }
    }
}

/// `include_str!`/`include_bytes!` paths must stay inside the crate, or the
/// packaged crate does not build.
fn includes_outside(crate_dir: &Path, dir: &Path, found: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            includes_outside(crate_dir, &path, found);
            continue;
        }
        if path.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        for macro_name in ["include_str!(\"", "include_bytes!(\""] {
            for (at, _) in text.match_indices(macro_name) {
                let rest = &text[at + macro_name.len()..];
                let target = &rest[..rest.find('"').unwrap()];
                let resolved = path.parent().unwrap().join(target);
                let resolved = std::fs::canonicalize(&resolved).unwrap_or(resolved);
                let root = std::fs::canonicalize(crate_dir).unwrap();
                if !resolved.starts_with(&root) {
                    found.push(format!("{}: {target}", path.display()));
                }
            }
        }
    }
}

#[test]
fn cli_5_published_crates_embed_only_their_own_files() {
    for (dir, _) in published_crates() {
        let crate_dir = repo().join("crates").join(&dir);
        let mut found = Vec::new();
        includes_outside(&crate_dir, &crate_dir.join("src"), &mut found);
        assert!(
            found.is_empty(),
            "crates/{dir} embeds files outside the crate: {found:?}"
        );
    }
}
