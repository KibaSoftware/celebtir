//! CLI-1 (RPC URL from the environment), CLI-2 (exit codes), CLI-3 (--format).

use std::process::{Command, Output};

use celebtir::{CliError, Exit, PUBLIC_RPC_URL, rpc_endpoint};
use clap::CommandFactory;

fn celebtir(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_celebtir"))
        .args(args)
        .env_remove("CELEBTIR_RPC_URL")
        .output()
        .unwrap()
}

fn registry(valid: bool) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("solana")).unwrap();
    let role = if valid { "core" } else { "periphery" };
    std::fs::write(
        dir.path().join("solana/demo.yaml"),
        format!(
            "id: demo\nname: Demo\nchain: solana\ncategory: dex\nwebsite: https://demo.example\n\
             programs:\n  - id: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA\n    name: Demo\n    role: {role}\n    source: https://demo.example/src\n\
             review:\n  reviewed_by: someone\n  last_reviewed: 2026-10-17\n"
        ),
    )
    .unwrap();
    dir
}

fn path(dir: &tempfile::TempDir) -> &str {
    dir.path().to_str().unwrap()
}

#[test]
fn cli_1_unset_rpc_url_falls_back_to_public_endpoint_with_warning() {
    let (rpc, warning) = rpc_endpoint(None);
    assert_eq!(rpc.url(), PUBLIC_RPC_URL);
    let warning = warning.expect("a warning when CELEBTIR_RPC_URL is unset");
    assert!(warning.contains("CELEBTIR_RPC_URL"), "{warning}");
}

#[test]
fn cli_1_rpc_url_comes_from_the_environment_without_warning() {
    let (rpc, warning) = rpc_endpoint(Some("https://rpc.example.com/?api-key=SECRET".into()));
    assert_eq!(rpc.url(), "https://rpc.example.com/?api-key=SECRET");
    assert!(warning.is_none());
    assert_eq!(rpc.to_string(), "rpc.example.com");
}

#[test]
fn cli_1_empty_rpc_url_counts_as_unset() {
    let (rpc, warning) = rpc_endpoint(Some("  ".into()));
    assert_eq!(rpc.url(), PUBLIC_RPC_URL);
    assert!(warning.is_some());
}

#[test]
fn cli_2_valid_registry_exits_0() {
    let dir = registry(true);
    let out = celebtir(&["validate", "--registry", path(&dir)]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn cli_2_validation_error_exits_1_and_names_the_file() {
    let dir = registry(false);
    let out = celebtir(&["validate", "--registry", path(&dir)]);
    assert_eq!(out.status.code(), Some(1));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains("solana/demo.yaml") && text.contains("/programs"),
        "{text}"
    );
}

#[test]
fn cli_2_missing_registry_dir_exits_1() {
    let out = celebtir(&["validate", "--registry", "does/not/exist"]);
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn cli_2_error_kinds_map_to_documented_exit_codes() {
    assert_eq!(Exit::Success as i32, 0);
    assert_eq!(CliError::Validation("x".into()).exit(), Exit::Validation);
    assert_eq!(Exit::Validation as i32, 1);
    assert_eq!(CliError::Partial("x".into()).exit(), Exit::Partial);
    assert_eq!(Exit::Partial as i32, 2);
    assert_eq!(
        CliError::BudgetExceeded("x".into()).exit(),
        Exit::BudgetExceeded
    );
    assert_eq!(Exit::BudgetExceeded as i32, 3);
}

#[test]
fn cli_3_every_command_accepts_format_json() {
    let cmd = celebtir::Cli::command();
    let mut count = 0;
    for sub in cmd.get_subcommands() {
        count += 1;
        let has = sub.get_arguments().any(|a| a.get_id() == "format")
            || cmd
                .get_arguments()
                .any(|a| a.get_id() == "format" && a.is_global_set());
        assert!(has, "`{}` has no --format", sub.get_name());
    }
    assert!(count > 0);
}

#[test]
fn cli_3_validate_json_output_parses_and_is_the_default_text_otherwise() {
    let dir = registry(false);
    let out = celebtir(&["validate", "--registry", path(&dir), "--format", "json"]);
    assert_eq!(out.status.code(), Some(1));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).expect("stdout is JSON");
    assert_eq!(json["ok"], false);
    assert_eq!(json["diagnostics"][0]["file"], "solana/demo.yaml");
    assert_eq!(json["diagnostics"][0]["pointer"], "/programs");

    let text = celebtir(&["validate", "--registry", path(&dir)]);
    assert!(serde_json::from_slice::<serde_json::Value>(&text.stdout).is_err());
}

#[test]
fn cli_3_unknown_format_is_rejected() {
    let dir = registry(true);
    let out = celebtir(&["validate", "--registry", path(&dir), "--format", "xml"]);
    // Usage errors use 1, not clap's default 2, which means "partial failure".
    assert_eq!(out.status.code(), Some(1));
    let help = celebtir(&["--help"]);
    assert_eq!(help.status.code(), Some(0));
}
