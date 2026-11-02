//! `celebtir collect` at the binary level, without network: an empty registry
//! needs no RPC calls. The full pipeline runs offline in 3.15 (CLI-4).

use std::process::Command;
use std::time::{Duration, UNIX_EPOCH};

fn celebtir(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_celebtir"))
        .args(args)
        .env_remove("CELEBTIR_RPC_URL")
        .output()
        .unwrap()
}

fn empty_registry() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("registry/solana")).unwrap();
    dir
}

#[test]
fn cli_1_collect_without_rpc_url_warns_and_still_runs() {
    let dir = empty_registry();
    let p = |s: &str| dir.path().join(s).to_string_lossy().into_owned();
    let o = celebtir(&[
        "collect",
        "--registry",
        &p("registry"),
        "--out",
        &p("out"),
        "--cache",
        &p("cache.json"),
    ]);
    let stderr = String::from_utf8_lossy(&o.stderr);
    assert_eq!(o.status.code(), Some(0), "{stderr}");
    assert!(stderr.contains("CELEBTIR_RPC_URL is not set"), "{stderr}");

    let run: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.path().join("out/run.json")).unwrap())
            .unwrap();
    assert_eq!(run["rpc_host"], "api.mainnet-beta.solana.com");
    assert_eq!(run["budget"], 5000);
    let facts: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.path().join("out/facts.json")).unwrap())
            .unwrap();
    assert_eq!(facts["protocols"], serde_json::json!({}));

    let schema = celebtir_core::schema::run_schema();
    let v = jsonschema::validator_for(&schema).unwrap();
    assert!(v.is_valid(&run), "run.json matches its schema");
}

#[test]
fn cli_2_collect_with_an_invalid_registry_exits_1() {
    let dir = empty_registry();
    std::fs::write(dir.path().join("registry/solana/x.yaml"), "id: [broken").unwrap();
    let p = |s: &str| dir.path().join(s).to_string_lossy().into_owned();
    let o = celebtir(&[
        "collect",
        "--registry",
        &p("registry"),
        "--out",
        &p("out"),
        "--cache",
        &p("cache.json"),
    ]);
    assert_eq!(o.status.code(), Some(1));
    assert!(
        !dir.path().join("out/facts.json").exists(),
        "nothing is written for a bad registry"
    );
}

#[test]
fn collect_unknown_protocol_exits_1() {
    let dir = empty_registry();
    let p = |s: &str| dir.path().join(s).to_string_lossy().into_owned();
    let o = celebtir(&[
        "collect",
        "--registry",
        &p("registry"),
        "--protocol",
        "nope",
        "--out",
        &p("out"),
        "--cache",
        &p("c.json"),
    ]);
    assert_eq!(o.status.code(), Some(1));
}

#[test]
fn run_timestamps_are_rfc3339_utc() {
    use celebtir::collect::rfc3339;
    assert_eq!(rfc3339(UNIX_EPOCH), "1970-01-01T00:00:00Z");
    assert_eq!(
        rfc3339(UNIX_EPOCH + Duration::from_secs(951_782_400)),
        "2000-02-29T00:00:00Z"
    );
    assert_eq!(
        rfc3339(UNIX_EPOCH + Duration::from_secs(1_704_067_199)),
        "2023-12-31T23:59:59Z"
    );
    assert_eq!(
        rfc3339(UNIX_EPOCH + Duration::from_secs(1_792_195_200)),
        "2026-10-17T00:00:00Z"
    );
}
