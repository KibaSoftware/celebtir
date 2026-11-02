//! REG-2 (on-chain half): every registry program ID resolves to an
//! executable account on mainnet. A closed or typo'd ID fails.

mod common;

use celebtir_core::Address;
use collect_solana::fixtures::load;
use collect_solana::reads::SolanaRpc;
use collect_solana::registry_check::check_programs;
use common::*;

#[tokio::test]
async fn reg_2_closed_or_non_executable_program_ids_fail() {
    let program = load("program-kamino-lend");
    let mut not_exec = load("programdata-kamino-lend");
    not_exec.executable = false;
    let rpc = SolanaRpc::new(client({
        let p = program.clone();
        let d = not_exec.clone();
        move |body: &serde_json::Value| {
            let n = body["params"][0].as_array().unwrap().len();
            assert_eq!(
                body["params"][1]["dataSlice"]["length"], 0,
                "existence check reads no data"
            );
            let mut values = vec![serde_json::Value::Null; n];
            for (i, a) in body["params"][0].as_array().unwrap().iter().enumerate() {
                for f in [&p, &d] {
                    if a.as_str() == Some(f.address.as_str()) {
                        values[i] = serde_json::json!({"data": ["", "base64"], "owner": f.owner, "lamports": 1, "executable": f.executable});
                    }
                }
            }
            serde_json::json!({"context": {"slot": 1}, "value": values})
        }
    }));
    let ids = [
        program.address.clone(),
        not_exec.address.clone(),
        Address::new("C1osed1111111111111111111111111111111111111"),
    ];
    let problems = check_programs(&rpc, &ids).await.unwrap();
    assert_eq!(problems.len(), 2, "{problems:#?}");
    assert!(
        problems
            .iter()
            .any(|(a, why)| *a == ids[1] && why.contains("not executable"))
    );
    assert!(
        problems
            .iter()
            .any(|(a, why)| *a == ids[2] && why.contains("does not exist"))
    );
    assert_eq!(rpc.client().report().calls["getMultipleAccounts"], 1);
}
