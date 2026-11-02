//! TOK-1 (`registry/tokens.yaml`) and TOK-2 (protocols list the tokens they use).

mod common;

use celebtir_registry::validate;
use common::{Registry, VALID};

const USDC: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
const WETH: &str = "7vfCXTUXx5WJV5JADk17DUJ4ksgau7utNKj4b963voxs";

const TOKENS: &str = r#"
tokens:
  - mint: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
    symbol: USDC
    type: stablecoin
    source: https://developers.circle.com/stablecoins/usdc-contract-addresses
    issuer:
      name: Circle
      source: https://www.circle.com/usdc
    provenance:
      kind: native
      source: https://developers.circle.com/stablecoins/usdc-contract-addresses
  - mint: 7vfCXTUXx5WJV5JADk17DUJ4ksgau7utNKj4b963voxs
    symbol: WETH
    type: bridged
    source: https://portalbridge.com/
    issuer:
      name: Wormhole
      source: https://wormhole.com/
    provenance:
      kind: bridged
      bridge: wormhole
      backing_chain: ethereum
      backing_address: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
      source: https://portalbridge.com/
"#;

fn with_tokens(uses: &[(&str, &str)]) -> String {
    let mut block = String::from("tokens:\n");
    for (mint, role) in uses {
        block.push_str(&format!("  - mint: {mint}\n    roles: [{role}]\n"));
    }
    VALID.replace("review:\n", &format!("{block}review:\n"))
}

fn errors(r: &Registry) -> Vec<String> {
    validate(r.path())
        .diagnostics
        .iter()
        .map(|d| d.to_string())
        .collect()
}

fn registry_with_tokens(tokens: &str, uses: &[(&str, &str)]) -> Registry {
    let r = Registry::valid();
    r.write("tokens.yaml", tokens);
    r.protocol("kamino-lend", &with_tokens(uses));
    r
}

#[test]
fn tok_1_valid_tokens_file_passes() {
    let r = registry_with_tokens(TOKENS, &[(USDC, "collateral"), (WETH, "borrowable")]);
    let report = validate(r.path());
    assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
    assert_eq!(report.tokens.len(), 2);
}

#[test]
fn tok_1_bridged_token_without_bridge_and_backing_address_fails() {
    let tokens = TOKENS.replace("      bridge: wormhole\n", "").replace(
        "      backing_address: \"0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2\"\n",
        "",
    );
    let r = registry_with_tokens(&tokens, &[(USDC, "collateral"), (WETH, "borrowable")]);
    let errs = errors(&r);
    assert!(
        errs.iter()
            .any(|e| e.starts_with("tokens.yaml") && e.contains("/tokens/1/provenance")),
        "{errs:#?}"
    );
}

#[test]
fn tok_1_every_claim_needs_a_source() {
    let tokens = TOKENS.replacen("      source: https://www.circle.com/usdc\n", "", 1);
    let r = registry_with_tokens(&tokens, &[(USDC, "collateral"), (WETH, "borrowable")]);
    let errs = errors(&r);
    assert!(
        errs.iter()
            .any(|e| e.contains("tokens.yaml") && e.contains("/tokens/0/issuer/source")),
        "{errs:#?}"
    );
}

#[test]
fn tok_1_duplicate_mint_fails() {
    let dup = format!(
        "{TOKENS}{}",
        &TOKENS[TOKENS.find("  - mint: EPj").unwrap()..TOKENS.find("  - mint: 7vf").unwrap()]
    );
    let r = registry_with_tokens(&dup, &[(USDC, "collateral"), (WETH, "borrowable")]);
    let errs = errors(&r);
    assert!(
        errs.iter()
            .any(|e| e.contains("tokens.yaml") && e.contains("duplicate")),
        "{errs:#?}"
    );
}

#[test]
fn tok_2_token_used_by_no_protocol_fails_as_out_of_scope() {
    let r = registry_with_tokens(TOKENS, &[(USDC, "collateral")]);
    let errs = errors(&r);
    assert_eq!(errs.len(), 1, "{errs:#?}");
    assert!(
        errs[0].contains("tokens.yaml")
            && errs[0].contains("/tokens/1")
            && errs[0].contains("out of scope")
    );
}

#[test]
fn tok_2_protocol_using_an_unregistered_token_fails() {
    let r = registry_with_tokens(
        TOKENS,
        &[
            (USDC, "collateral"),
            (WETH, "borrowable"),
            ("So11111111111111111111111111111111111111112", "deposit"),
        ],
    );
    let errs = errors(&r);
    assert!(
        errs.iter()
            .any(|e| e.contains("kamino-lend.yaml") && e.contains("/tokens/2/mint")),
        "{errs:#?}"
    );
}

#[test]
fn tok_2_unknown_role_fails() {
    let r = registry_with_tokens(TOKENS, &[(USDC, "collateral"), (WETH, "airdrop")]);
    let errs = errors(&r);
    assert!(
        errs.iter()
            .any(|e| e.contains("kamino-lend.yaml") && e.contains("/tokens/1/roles")),
        "{errs:#?}"
    );
}

#[test]
fn tok_1_committed_schema_matches_the_code() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../schemas/registry-tokens.schema.json");
    let generated = celebtir_core::to_canonical_json(&celebtir_registry::tokens_schema()).unwrap();
    if std::env::var_os("CELEBTIR_UPDATE_SCHEMAS").is_some() {
        std::fs::write(&path, &generated).unwrap();
    }
    let committed = std::fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(
        committed, generated,
        "stale; run with CELEBTIR_UPDATE_SCHEMAS=1"
    );
}
