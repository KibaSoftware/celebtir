//! HREG-1, HREG-2, HREG-3: HyperEVM registry files under `hyperliquid/`
//! and the EVM sections of `controllers.yaml` and `oracles.yaml` (ADR 024).

mod common;

use celebtir_registry::{ProxyKind, evm, validate};
use common::Registry;

// EIP-55 test vectors.
const A: &str = "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed";
const B: &str = "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359";
const C: &str = "0xdbF03B407c01E7cD3CBea99509d93f8DDDC8C6FB";
const D: &str = "0xD1220A0cf47c7B9Be7A2E6BA89F429762e7b9aDb";

fn evm_protocol() -> String {
    format!(
        r#"
id: evm-lend
name: EVM Lend
chain: hyper_evm
category: lending
website: https://example.org
programs:
  - id: {A}
    name: Pool proxy
    role: core
    proxy: eip1967
    source: https://example.org/contracts
safes:
  - address: {B}
    source: https://example.org/safe
timelocks:
  - address: {C}
    proposers: [{B}]
    source: https://example.org/timelock
review:
  reviewed_by: andreikiba
  last_reviewed: 2026-10-21
"#
    )
}

fn registry(yaml: &str) -> Registry {
    let r = Registry::valid();
    std::fs::create_dir_all(r.path().join("hyperliquid")).unwrap();
    r.write("hyperliquid/evm-lend.yaml", yaml);
    r
}

fn errors(r: &Registry) -> Vec<String> {
    validate(r.path())
        .diagnostics
        .iter()
        .map(|d| d.to_string())
        .collect()
}

#[test]
fn hreg_1_eip55_matches_the_spec_vectors() {
    for a in [A, B, C, D] {
        assert_eq!(evm::eip55(&a.to_ascii_lowercase()).as_deref(), Some(a));
        assert!(evm::is_checksummed(a));
    }
    assert_eq!(evm::eip55("0x123"), None);
    assert_eq!(evm::eip55(&A.replace("0x", "")), None);
}

#[test]
fn hreg_1_valid_hyperevm_file_passes_and_is_kept_apart() {
    let r = registry(&evm_protocol());
    let report = validate(r.path());
    assert!(report.is_ok(), "{:#?}", errors(&r));
    assert_eq!(report.protocols.len(), 1, "Solana list unchanged");
    assert_eq!(report.hyperevm_protocols.len(), 1);
    let p = &report.hyperevm_protocols[0];
    assert_eq!(p.programs[0].proxy, Some(ProxyKind::Eip1967));
    assert_eq!(p.timelocks[0].proposers[0].as_str(), B);
}

#[test]
fn hreg_1_lower_case_address_fails_with_file_and_field() {
    let r = registry(&evm_protocol().replace(A, &A.to_ascii_lowercase()));
    let errs = errors(&r);
    assert!(
        errs.iter().any(|e| e.contains("hyperliquid/evm-lend.yaml")
            && e.contains("/programs/0/id")
            && e.contains("EIP-55")),
        "{errs:#?}"
    );
}

#[test]
fn hreg_1_bad_checksum_fails() {
    // Flip the case of one letter.
    let bad = A.replacen("aAeb", "aaeb", 1);
    let r = registry(&evm_protocol().replace(A, &bad));
    assert!(
        errors(&r).iter().any(|e| e.contains("/programs/0/id")),
        "{:#?}",
        errors(&r)
    );
}

#[test]
fn hreg_1_contract_without_proxy_kind_fails() {
    let r = registry(&evm_protocol().replace("    proxy: eip1967\n", ""));
    assert!(
        errors(&r).iter().any(|e| e.contains("/programs/0/proxy")),
        "{:#?}",
        errors(&r)
    );
}

#[test]
fn hreg_1_wrong_chain_in_hyperliquid_dir_fails() {
    let r = registry(&evm_protocol().replace("chain: hyper_evm", "chain: solana"));
    assert!(
        errors(&r).iter().any(|e| e.contains("/chain")),
        "{:#?}",
        errors(&r)
    );
}

#[test]
fn hreg_2_timelock_without_proposers_fails() {
    let r = registry(&evm_protocol().replace(&format!("proposers: [{B}]"), "proposers: []"));
    let errs = errors(&r);
    assert!(
        errs.iter().any(|e| e.contains("/timelocks/0/proposers")),
        "{errs:#?}"
    );
}

#[test]
fn hreg_2_safe_needs_no_member_list() {
    // The valid file lists a Safe with only address and source.
    let r = registry(&evm_protocol());
    assert!(validate(r.path()).is_ok(), "{:#?}", errors(&r));
}

fn known(controllers: &str, oracles: &str) -> Registry {
    let r = Registry::valid();
    r.write("controllers.yaml", controllers);
    r.write("oracles.yaml", oracles);
    r
}

#[test]
fn hreg_3_evm_sections_validate() {
    let r = known(
        &format!(
            "programs: []\nevm:\n  - address: {A}\n    kind: safe-singleton\n    name: safe-1.3.0\n    source: https://github.com/safe-global/safe-deployments\n"
        ),
        &format!(
            "programs: []\nevm:\n  - address: {B}\n    class: network\n    name: pyth\n    source: https://docs.pyth.network/\n"
        ),
    );
    assert!(validate(r.path()).is_ok(), "{:#?}", errors(&r));
}

#[test]
fn hreg_3_duplicate_address_across_sections_fails() {
    let r = known(
        &format!(
            "programs: []\nevm:\n  - address: {A}\n    kind: timelock\n    name: oz-timelock\n    source: https://docs.openzeppelin.com/\n"
        ),
        &format!(
            "programs: []\nevm:\n  - address: {A}\n    class: network\n    name: pyth\n    source: https://docs.pyth.network/\n"
        ),
    );
    let errs = errors(&r);
    assert!(
        errs.iter()
            .any(|e| e.contains("oracles.yaml") && e.contains("already listed")),
        "{errs:#?}"
    );
}
