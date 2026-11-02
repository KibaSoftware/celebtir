//! COL-6 (program account → ProgramData header, never the bytecode) and
//! COL-7 (loader classification), against captured mainnet fixtures.

mod common;

use celebtir_core::Address;
use collect_solana::fixtures::load;
use collect_solana::program::{
    self, LOADER_V4, ProgramDataHeader, ProgramKind, decode_programdata,
};
use collect_solana::reads::{AccountRead, RawAccount, SolanaRpc};
use common::*;
use serde_json::{Value, json};

fn a(s: &str) -> Address {
    Address::new(s)
}

#[test]
fn col_6_program_account_points_to_its_programdata() {
    let kind = program::classify(&load("program-kamino-lend").read()).unwrap();
    assert_eq!(
        kind,
        ProgramKind::Upgradeable {
            programdata: a("9uSbGW1y9H5Av6H5TKxQ1wnFApSq2t3oEpfF2YfjDQGA")
        }
    );
}

#[test]
fn col_6_decodes_last_deploy_slot_and_authority() {
    let h = decode_programdata(&load("programdata-kamino-lend").data()).unwrap();
    assert_eq!(
        h,
        ProgramDataHeader {
            last_deploy_slot: 440_486_775,
            authority: Some(a("GzFgdRJXmawPhGeBsyRCDLx4jAKPsvbUqoqitzppkzkW"))
        }
    );
}

#[test]
fn col_6_decodes_a_removed_authority_as_none() {
    let h = decode_programdata(&load("programdata-spl-token").data()).unwrap();
    assert_eq!(h.authority, None);
    assert_eq!(h.last_deploy_slot, 419_472_000);
}

#[test]
fn col_6_truncated_or_mistagged_programdata_is_an_error_not_a_guess() {
    let good = load("programdata-kamino-lend").data();
    assert!(decode_programdata(&good[..20]).is_err());
    let mut wrong_tag = good.clone();
    wrong_tag[0] = 2;
    assert!(decode_programdata(&wrong_tag).is_err());
    let mut bad_option = good;
    bad_option[12] = 7;
    assert!(decode_programdata(&bad_option).is_err());
}

#[test]
fn col_7_loader_v1_and_v2_programs_are_immutable() {
    for name in ["program-memo-v1", "program-memo-v2"] {
        let kind = program::classify(&load(name).read()).unwrap();
        assert!(
            matches!(kind, ProgramKind::Immutable { .. }),
            "{name}: {kind:?}"
        );
    }
}

#[test]
fn col_7_native_and_loader_v4_are_unsupported_not_guessed() {
    let native = program::classify(&load("program-system").read()).unwrap();
    assert_eq!(
        native,
        ProgramKind::UnsupportedLoader {
            owner: a("NativeLoader1111111111111111111111111111111")
        }
    );

    // Synthetic: no well-known loader-v4 program on mainnet to capture.
    let v4 = AccountRead {
        address: a("Synthetic1oaderV4Program111111111111111111"),
        slot: 1,
        account: Some(RawAccount {
            owner: a(LOADER_V4),
            lamports: 1,
            executable: true,
            data: vec![0; 48],
        }),
    };
    assert_eq!(
        program::classify(&v4).unwrap(),
        ProgramKind::UnsupportedLoader {
            owner: a(LOADER_V4)
        }
    );
}

#[test]
fn col_7_missing_program_account_is_closed() {
    let read = AccountRead {
        address: a("x"),
        slot: 9,
        account: None,
    };
    assert_eq!(program::classify(&read).unwrap(), ProgramKind::Closed);
}

#[test]
fn col_7_each_program_records_its_loader_for_the_protocol_page() {
    let upgradeable = program::classify(&load("program-kamino-lend").read()).unwrap();
    assert_eq!(upgradeable.loader(), Some(a(program::UPGRADEABLE_LOADER)));
    let v2 = program::classify(&load("program-memo-v2").read()).unwrap();
    assert_eq!(v2.loader(), Some(a(program::LOADER_V2)));
    let native = program::classify(&load("program-system").read()).unwrap();
    assert_eq!(
        native.loader(),
        Some(a("NativeLoader1111111111111111111111111111111"))
    );
    assert_eq!(ProgramKind::Closed.loader(), None);
}

/// Serves captured fixtures by address, honouring dataSlice like a real node.
fn fixture_rpc() -> SolanaRpc<FnTransport> {
    use base64::Engine;
    let fixtures: Vec<_> = [
        "program-kamino-lend",
        "programdata-kamino-lend",
        "program-spl-token",
        "programdata-spl-token",
        "program-memo-v1",
    ]
    .iter()
    .map(|n| load(n))
    .collect();
    SolanaRpc::new(client(move |body: &Value| {
        let slice = &body["params"][1]["dataSlice"];
        let values: Vec<Value> = body["params"][0]
            .as_array()
            .unwrap()
            .iter()
            .map(|addr| {
                let f = fixtures
                    .iter()
                    .find(|f| f.address.as_str() == addr.as_str().unwrap())
                    .unwrap();
                let mut data = f.data();
                if let (Some(o), Some(l)) = (slice["offset"].as_u64(), slice["length"].as_u64()) {
                    data = data.into_iter().skip(o as usize).take(l as usize).collect();
                }
                json!({"data": [base64::engine::general_purpose::STANDARD.encode(data), "base64"],
                       "owner": f.owner, "lamports": f.lamports, "executable": f.executable})
            })
            .collect();
        json!({"context": {"slot": 458_009_999u64}, "value": values})
    }))
}

#[tokio::test]
async fn col_6_reading_program_control_never_downloads_bytecode() {
    let rpc = fixture_rpc();
    let programs = [
        a("KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD"),
        a("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
        a("Memo1UhkJRfHyvLMcVucJwxXeuD728EqVDDwQDxFMNo"),
    ];
    let reads = program::read_program_control(&rpc, &programs)
        .await
        .unwrap();
    assert_eq!(reads.len(), 3);
    assert_eq!(
        reads[0].header.as_ref().unwrap().authority,
        Some(a("GzFgdRJXmawPhGeBsyRCDLx4jAKPsvbUqoqitzppkzkW"))
    );
    assert_eq!(reads[1].header.as_ref().unwrap().authority, None);
    assert!(matches!(reads[2].kind, ProgramKind::Immutable { .. }));
    assert!(reads[2].header.is_none());
    assert_eq!(reads[0].programdata_slot, Some(458_009_999));

    let bodies = rpc.client().transport().bodies();
    assert_eq!(
        bodies.len(),
        2,
        "one batch of program accounts, one of ProgramData headers"
    );
    for b in bodies {
        let len = b["params"][1]["dataSlice"]["length"]
            .as_u64()
            .expect("every read is sliced");
        assert!(len <= 45, "{b}");
    }
}
