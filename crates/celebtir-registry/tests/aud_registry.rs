//! Launch-audit registry rules (CELEBTIR_FIX_INSTRUCTIONS F1, F3, F10):
//! source facts keyed by program ID with a repository rule, sourced
//! reviewed-none statements, and stake pools bound to their pool mint.

mod common;

use std::path::PathBuf;

use celebtir_registry::{check_commit, check_repo_url, validate};
use common::{Registry, VALID};

fn registry_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../registry")
}

fn messages(r: &Registry) -> Vec<String> {
    validate(r.dir.path())
        .diagnostics
        .iter()
        .map(|d| d.to_string())
        .collect()
}

const PROGRAMS: &str = r#"
programs:
  - id: KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD
    name: Lending
    source_code:
      kind: public
      repo: https://github.com/Kamino-Finance/klend
      commit: e921c648765fc2343c87614a00f4b39b640f0510
      evidence: https://verify.osec.io/status/KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD
  - id: FarmsPZpWu9i7Kky8tPN37rs2TpmMrAZrC7S7vJa91Hr
    name: Farms
    source_code:
      kind: none_found
      evidence: https://docs.kamino.finance/
"#;

#[test]
fn aud_1_repo_url_rejects_org_docs_explorer_idl_and_adapter_links() {
    for bad in [
        "https://github.com/jup-ag",
        "https://github.com/pump-fun/pump-public-docs/blob/main/docs/PUMP_SWAP_README.md",
        "https://solscan.io/account/PERPHjGBqRHArX4DySjwM6UJHiR3sWAatqfdBS2qQJu",
        "https://github.com/julianfssen/jupiter-perps-anchor-idl-parsing",
        "https://github.com/DefiLlama/DefiLlama-Adapters",
        "https://github.com/raydium-io/raydium-clmm/tree/119ed36bf9f33",
        "https://docs.raydium.io/reference/program-addresses",
        "https://github.com/someone/solana-idl-parser",
    ] {
        assert!(check_repo_url(bad).is_err(), "{bad} must be rejected");
    }
    for good in [
        "https://github.com/solana-program/stake-pool",
        "https://github.com/raydium-io/raydium-clmm",
        "https://gitlab.com/org/repo.git",
    ] {
        assert!(check_repo_url(good).is_ok(), "{good}");
    }
}

#[test]
fn aud_1_commit_must_be_a_full_hash() {
    assert!(check_commit("119ed36bf9f33").is_err());
    assert!(check_commit("119ed36bf9f33335ed8d671780502bc37d9f1605").is_ok());
    assert!(check_commit("119ED36BF9F33335ED8D671780502BC37D9F1605").is_err());
}

#[test]
fn aud_1_every_program_needs_one_source_entry() {
    let r = Registry::valid();
    r.write(
        "programs.yaml",
        "programs:\n  - id: KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD\n    name: Lending\n    source_code:\n      kind: none_found\n      evidence: https://docs.kamino.finance/\n",
    );
    let m = messages(&r);
    assert!(
        m.iter()
            .any(|d| d.contains("FarmsPZp") && d.contains("programs.yaml")),
        "{m:?}"
    );
    r.write("programs.yaml", PROGRAMS);
    assert!(messages(&r).is_empty(), "{:?}", messages(&r));
}

#[test]
fn aud_1_programs_file_rejects_org_pages_and_short_commits() {
    let r = Registry::valid();
    r.write(
        "programs.yaml",
        &PROGRAMS
            .replace(
                "https://github.com/Kamino-Finance/klend",
                "https://github.com/Kamino-Finance",
            )
            .replace("e921c648765fc2343c87614a00f4b39b640f0510", "e921c648765f"),
    );
    let m = messages(&r);
    assert!(m.iter().any(|d| d.contains("/source_code/repo")), "{m:?}");
    assert!(m.iter().any(|d| d.contains("/source_code/commit")), "{m:?}");
}

#[test]
fn aud_1_published_registry_has_a_source_for_every_program() {
    let report = validate(&registry_root());
    assert!(report.is_ok(), "{:?}", report.diagnostics);
    for p in &report.protocols {
        for program in &p.programs {
            assert!(
                report.programs.contains_key(program.id.as_str()),
                "{}: {} has no programs.yaml entry",
                p.id,
                program.id
            );
        }
    }
}

#[test]
fn aud_3_reviewed_none_cannot_contradict_the_entry() {
    let r = Registry::valid();
    let with = VALID.replace(
        "review:",
        "admin_reviewed_none:\n  source: https://example.org\n  reviewed_on: 2026-10-25\n  note: checked\noracles_reviewed_none:\n  source: https://example.org\n  reviewed_on: 2026-10-25\n  note: checked\nreview:",
    );
    r.protocol("kamino-lend", &with);
    let m = messages(&r);
    assert!(
        m.iter().any(|d| d.contains("/admin_reviewed_none")),
        "{m:?}"
    );
    assert!(
        m.iter()
            .any(|d| d.contains("/oracles_reviewed_none") && d.contains("lending")),
        "{m:?}"
    );
}

#[test]
fn aud_3_stake_pool_mint_must_be_a_protocol_token() {
    let r = Registry::valid();
    let with = VALID.replace(
        "review:",
        "stake_pools:\n  - account: Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb\n    pool_mint: J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn\n    source: https://example.org\nreview:",
    );
    r.protocol("kamino-lend", &with);
    let m = messages(&r);
    assert!(
        m.iter().any(|d| d.contains("/stake_pools/0/pool_mint")),
        "{m:?}"
    );
}

#[test]
fn aud_1_committed_programs_schema_matches_the_code() {
    let path = registry_root().join("../schemas/registry-programs.schema.json");
    let generated =
        celebtir_core::to_canonical_json(&celebtir_registry::programs_schema()).unwrap();
    if std::env::var_os("CELEBTIR_UPDATE_SCHEMAS").is_some() {
        std::fs::write(&path, &generated).unwrap();
    }
    let committed = std::fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(
        committed, generated,
        "stale; run with CELEBTIR_UPDATE_SCHEMAS=1"
    );
}
