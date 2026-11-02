//! TOK-3: every mint read in one batched call; owner program, supply,
//! decimals, mint authority and freeze authority recorded.

mod common;

use celebtir_core::Address;
use celebtir_core::snapshot::TokenProgram;
use collect_solana::fixtures::load;
use collect_solana::mint::{decode_mint, read_mints};
use collect_solana::reads::SolanaRpc;
use common::*;

fn a(s: &str) -> Address {
    Address::new(s)
}

#[test]
fn tok_3_usdc_reports_both_authorities() {
    let m = decode_mint(&load("mint-usdc").read()).unwrap();
    assert_eq!(m.program, TokenProgram::SplToken);
    assert_eq!(m.decimals, 6);
    assert_eq!(m.supply, 7_966_579_498_112_302);
    assert_eq!(
        m.mint_authority,
        Some(a("BJE5MMbqXjVwjAF7oxwPYXnTXDyspzZyt4vwenNw5ruG"))
    );
    assert_eq!(
        m.freeze_authority,
        Some(a("7dGbd2QZcCKcTndnHcTL8q7SMVXAkp688NTQYwrRCrar"))
    );
    assert!(m.extension_data.is_empty());
}

#[test]
fn tok_3_jitosol_has_no_freeze_authority() {
    let m = decode_mint(&load("mint-jitosol").read()).unwrap();
    assert_eq!(m.decimals, 9);
    assert_eq!(
        m.mint_authority,
        Some(a("6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS"))
    );
    assert_eq!(m.freeze_authority, None);
}

#[test]
fn tok_3_token_2022_mint_keeps_its_extension_bytes() {
    let m = decode_mint(&load("mint-pyusd").read()).unwrap();
    assert_eq!(m.program, TokenProgram::Token2022);
    assert_eq!(
        m.mint_authority,
        Some(a("AxJkHRY3puoofyj7Pn512M1SGBgWroXMkEdNPQypkzbu"))
    );
    assert_eq!(
        m.freeze_authority,
        Some(a("2apBGMsS6ti9RyF5TwQTDswXBWskiJP2LD4cUEDqYJjk"))
    );
    assert_eq!(
        m.extension_data.len(),
        866 - 166,
        "TLV data after the account-type byte"
    );
}

#[test]
fn tok_3_rejects_non_mints() {
    let mut not_token = load("mint-usdc").read();
    not_token.account.as_mut().unwrap().owner = a("11111111111111111111111111111111");
    assert!(
        decode_mint(&not_token)
            .unwrap_err()
            .0
            .contains("not a token program")
    );

    let mut short = load("mint-usdc").read();
    short.account.as_mut().unwrap().data.truncate(60);
    assert!(decode_mint(&short).is_err());

    let mut uninit = load("mint-usdc").read();
    uninit.account.as_mut().unwrap().data[45] = 0;
    assert!(decode_mint(&uninit).unwrap_err().0.contains("initialized"));

    let mut bad_tag = load("mint-usdc").read();
    bad_tag.account.as_mut().unwrap().data[0] = 9;
    assert!(decode_mint(&bad_tag).is_err());
}

#[tokio::test]
async fn tok_3_all_mints_are_read_in_one_batched_call() {
    let accounts = ["mint-usdc", "mint-jitosol", "mint-pyusd"]
        .iter()
        .map(|n| {
            let f = load(n);
            (
                f.address.to_string(),
                f.owner.clone().unwrap().to_string(),
                f.data(),
            )
        })
        .collect();
    let rpc = SolanaRpc::new(account_server(accounts));
    let mints = [
        load("mint-usdc").address,
        load("mint-jitosol").address,
        load("mint-pyusd").address,
        a("Missing1111111111111111111111111111111111111"),
    ];
    let reads = read_mints(&rpc, &mints).await.unwrap();
    assert_eq!(reads.len(), 4);
    assert!(reads[..3].iter().all(Result::is_ok));
    assert!(reads[3].as_ref().unwrap_err().0.contains("does not exist"));
    assert_eq!(rpc.client().report().calls["getMultipleAccounts"], 1);
    assert_eq!(reads[0].as_ref().unwrap().slot, 458_100_000);
}
