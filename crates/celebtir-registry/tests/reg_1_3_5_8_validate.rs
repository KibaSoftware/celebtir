//! REG-1 (schema), REG-3 (sources), REG-5 (core program), REG-8 (vendored IDLs).

mod common;

use celebtir_registry::validate;
use common::{Registry, VALID};

fn errors(r: &Registry) -> Vec<String> {
    validate(r.path())
        .diagnostics
        .iter()
        .map(|d| d.to_string())
        .collect()
}

fn assert_error(r: &Registry, needles: &[&str]) {
    let errs = errors(r);
    assert!(
        errs.iter().any(|e| needles.iter().all(|n| e.contains(n))),
        "expected an error containing {needles:?}, got {errs:#?}"
    );
}

#[test]
fn reg_1_valid_registry_passes() {
    let r = Registry::valid();
    let report = validate(r.path());
    assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
    assert_eq!(report.protocols.len(), 1);
    assert_eq!(report.protocols[0].id, "kamino-lend");
}

#[test]
fn reg_1_missing_source_names_file_and_field() {
    let r = Registry::valid();
    r.protocol(
        "kamino-lend",
        &VALID.replacen(
            "    source: https://github.com/Kamino-Finance/klend\n",
            "",
            1,
        ),
    );
    assert_error(&r, &["kamino-lend.yaml", "/programs/0/source"]);
}

#[test]
fn reg_1_unknown_category_names_file_and_field() {
    let r = Registry::valid();
    r.protocol(
        "kamino-lend",
        &VALID.replace("category: lending", "category: casino"),
    );
    assert_error(&r, &["kamino-lend.yaml", "/category"]);
}

#[test]
fn reg_1_unknown_field_is_rejected() {
    let r = Registry::valid();
    r.protocol(
        "kamino-lend",
        &VALID.replace("name: Kamino Lend", "name: Kamino Lend\ntvl: 5"),
    );
    assert_error(&r, &["kamino-lend.yaml", "tvl"]);
}

#[test]
fn reg_1_file_name_must_match_id() {
    let r = Registry::valid();
    r.remove("solana/kamino-lend.yaml");
    r.protocol("kamino", VALID);
    assert_error(&r, &["kamino.yaml", "/id"]);
}

#[test]
fn reg_1_id_must_be_kebab_case() {
    let r = Registry::valid();
    r.remove("solana/kamino-lend.yaml");
    r.protocol(
        "Kamino_Lend",
        &VALID.replace("id: kamino-lend", "id: Kamino_Lend"),
    );
    assert_error(&r, &["Kamino_Lend.yaml", "/id"]);
}

#[test]
fn reg_1_invalid_yaml_names_file() {
    let r = Registry::valid();
    r.protocol("kamino-lend", "id: [unclosed");
    assert_error(&r, &["kamino-lend.yaml"]);
}

#[test]
fn reg_1_committed_schema_matches_the_code() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../schemas/registry-protocol.schema.json");
    let generated =
        celebtir_core::to_canonical_json(&celebtir_registry::protocol_schema()).unwrap();
    if std::env::var_os("CELEBTIR_UPDATE_SCHEMAS").is_some() {
        std::fs::write(&path, &generated).unwrap();
    }
    let committed = std::fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(
        committed, generated,
        "stale; run with CELEBTIR_UPDATE_SCHEMAS=1"
    );
}

#[test]
fn reg_3_every_entry_kind_requires_a_source() {
    let cases = [
        (
            "    vault_index: 0\n    source: https://docs.kamino.finance/\n",
            "    vault_index: 0\n",
            "/multisigs/0/source",
        ),
        (
            "    power: parameters\n    source: https://github.com/Kamino-Finance/klend\n",
            "    power: parameters\n",
            "/admin_accounts/0/source",
        ),
        (
            "    role: primary\n    source: https://docs.kamino.finance/\n",
            "    role: primary\n",
            "/oracles/0/source",
        ),
    ];
    for (from, to, field) in cases {
        let r = Registry::valid();
        let yaml = VALID.replace(from, to);
        assert_ne!(yaml, VALID, "fixture edit for {field} did not apply");
        r.protocol("kamino-lend", &yaml);
        assert_error(&r, &["kamino-lend.yaml", field]);
    }
}

#[test]
fn reg_3_source_must_be_a_url() {
    let r = Registry::valid();
    r.protocol(
        "kamino-lend",
        &VALID.replacen(
            "source: https://github.com/Kamino-Finance/klend",
            "source: the docs",
            1,
        ),
    );
    assert_error(&r, &["kamino-lend.yaml", "/programs/0/source"]);
}

#[test]
fn reg_5_protocol_with_only_periphery_programs_fails() {
    let r = Registry::valid();
    r.protocol(
        "kamino-lend",
        &VALID.replace("role: core", "role: periphery"),
    );
    assert_error(&r, &["kamino-lend.yaml", "/programs", "core"]);
}

#[test]
fn reg_8_missing_idl_file_fails() {
    let r = Registry::valid();
    r.remove("idl/kamino-lend.json");
    assert_error(
        &r,
        &[
            "kamino-lend.yaml",
            "/admin_accounts/0/idl",
            "does not exist",
        ],
    );
}

#[test]
fn reg_8_idl_without_recorded_origin_fails() {
    let r = Registry::valid();
    r.write("idl/origins.yaml", "{}\n");
    assert_error(&r, &["kamino-lend.yaml", "/admin_accounts/0/idl", "origin"]);
}

#[test]
fn reg_8_origin_needs_commit_or_slot() {
    let r = Registry::valid();
    r.write(
        "idl/origins.yaml",
        "kamino-lend.json:\n  source: https://github.com/Kamino-Finance/klend\n",
    );
    assert_error(&r, &["origins.yaml", "kamino-lend.json"]);
}

#[test]
fn reg_8_origin_with_slot_passes() {
    let r = Registry::valid();
    r.write(
        "idl/origins.yaml",
        "kamino-lend.json:\n  source: https://solscan.io/account/KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD\n  slot: 382114880\n",
    );
    assert!(errors(&r).is_empty(), "{:#?}", errors(&r));
}

#[test]
fn reg_8_idl_path_cannot_escape_the_idl_dir() {
    let r = Registry::valid();
    r.protocol(
        "kamino-lend",
        &VALID.replace("idl: idl/kamino-lend.json", "idl: ../secrets.json"),
    );
    assert_error(&r, &["kamino-lend.yaml", "/admin_accounts/0/idl"]);
}

#[test]
fn reg_1_reserved_ids_are_rejected() {
    let r = Registry::valid();
    r.remove("solana/kamino-lend.yaml");
    r.protocol("summary", &VALID.replace("id: kamino-lend", "id: summary"));
    assert_error(&r, &["summary.yaml", "/id", "reserved"]);
}

#[test]
fn reg_1_a_squads_v3_multisig_is_accepted() {
    let r = Registry::valid();
    r.protocol(
        "kamino-lend",
        &VALID.replacen("program: squads-v4", "program: squads-v3", 1),
    );
    assert!(errors(&r).is_empty(), "{:#?}", errors(&r));
}

#[test]
fn reg_1_an_unknown_multisig_program_is_rejected() {
    let r = Registry::valid();
    r.protocol(
        "kamino-lend",
        &VALID.replacen("program: squads-v4", "program: squads-v9", 1),
    );
    assert_error(&r, &["kamino-lend.yaml", "/multisigs/0/program"]);
}

/// Edge case 11: an oracle entry may say where its Scope chain lives; the IDL
/// it names follows the same rules as an admin account's (REG-8).
fn with_scope_chain(idl: &str) -> String {
    VALID.replacen(
        "    role: primary\n",
        &format!(
            "    role: primary\n    scope_chain:\n      account: d4A2prbA2whesmvHaL88BH6Ewn5N4bTSU2Ze8P6Bc4Q\n      idl: {idl}\n      feed_field: config.token_info.scope_configuration.price_feed\n      chain_field: config.token_info.scope_configuration.price_chain\n"
        ),
        1,
    )
}

#[test]
fn reg_8_an_oracle_scope_chain_with_a_vendored_idl_passes() {
    let r = Registry::valid();
    let yaml = with_scope_chain("idl/kamino-lend.json");
    assert!(
        yaml.contains("scope_chain"),
        "the fixture has a primary oracle"
    );
    r.protocol("kamino-lend", &yaml);
    assert!(errors(&r).is_empty(), "{:#?}", errors(&r));
}

#[test]
fn reg_8_an_oracle_scope_chain_idl_must_exist() {
    let r = Registry::valid();
    r.protocol("kamino-lend", &with_scope_chain("idl/missing.json"));
    assert_error(
        &r,
        &[
            "kamino-lend.yaml",
            "/oracles/0/scope_chain/idl",
            "does not exist",
        ],
    );
}
