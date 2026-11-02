//! COL-13, F1: OtterSec verified-build status per program: verified flag,
//! both hashes, repo and commit stored. The public source comes from the
//! program's `programs.yaml` entry, never a protocol's GitHub org; a verified
//! repo that differs from it is flagged.

use std::collections::VecDeque;
use std::sync::Mutex;

use celebtir_core::snapshot::CodeTransparency;
use celebtir_core::{Address, Chain, Evidence};
use celebtir_registry::{ProgramSource, SourceCode};
use collect_solana::rpc::TransportError;
use collect_solana::verify::{HttpGet, OTTERSEC_STATUS_URL, read_verification};
use serde_json::Value;

const KLEND: &str = "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD";
const DRIFT: &str = "dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH";
const RAYDIUM: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
const MEMO: &str = "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr";

/// Serves the captured OtterSec responses; can fail the first N requests.
struct Fixtures {
    failures: Mutex<VecDeque<TransportError>>,
    urls: Mutex<Vec<String>>,
}

impl Fixtures {
    fn new(failures: Vec<TransportError>) -> Self {
        Self {
            failures: Mutex::new(failures.into()),
            urls: Mutex::new(Vec::new()),
        }
    }
}

impl HttpGet for Fixtures {
    async fn get_json(&self, url: &str) -> Result<Value, TransportError> {
        self.urls.lock().unwrap().push(url.to_string());
        if let Some(e) = self.failures.lock().unwrap().pop_front() {
            return Err(e);
        }
        let program = url.rsplit('/').next().unwrap();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/http/ottersec")
            .join(format!("{program}.json"));
        let text = std::fs::read_to_string(path).map_err(|_| TransportError::Status(404))?;
        serde_json::from_str(&text).map_err(|e| TransportError::Decode(e.to_string()))
    }
}

fn ev(program: &str) -> Evidence {
    Evidence::new(
        Chain::Solana,
        Address::new(program),
        458_009_913,
        "verified_build",
    )
}

fn public(repo: &str, commit: Option<&str>) -> ProgramSource {
    ProgramSource {
        id: Address::new(KLEND),
        name: "test".into(),
        source_code: SourceCode::Public {
            repo: repo.parse().unwrap(),
            commit: commit.map(str::to_string),
            path: None,
            evidence: "https://example.org/declare_id".parse().unwrap(),
        },
    }
}

fn none_found() -> ProgramSource {
    ProgramSource {
        id: Address::new(KLEND),
        name: "test".into(),
        source_code: SourceCode::NoneFound {
            evidence: "https://example.org/programs".parse().unwrap(),
        },
    }
}

async fn status(http: &Fixtures, program: &str, source: Option<ProgramSource>) -> CodeTransparency {
    read_verification(http, &Address::new(program), source.as_ref(), ev(program))
        .await
        .value()
        .clone()
}

#[tokio::test]
async fn col_13_verified_program_stores_flag_hashes_repo_and_commit() {
    let http = Fixtures::new(vec![]);
    let t = status(
        &http,
        KLEND,
        Some(public("https://github.com/Kamino-Finance/klend", None)),
    )
    .await;
    match t {
        CodeTransparency::Verified {
            repo,
            commit,
            on_chain_hash,
            executable_hash,
            warning,
        } => {
            assert_eq!(repo, "https://github.com/Kamino-Finance/klend");
            assert_eq!(commit, "a08760976f51a3a58c4a0c6ea27b4a0e565bca79");
            assert_eq!(on_chain_hash, executable_hash);
            assert_eq!(on_chain_hash.len(), 64);
            assert_eq!(warning, None);
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(
        http.urls.lock().unwrap()[0],
        format!("{OTTERSEC_STATUS_URL}{KLEND}")
    );
}

#[tokio::test]
async fn col_13_verified_repo_other_than_the_registry_source_is_flagged() {
    let http = Fixtures::new(vec![]);
    let t = status(
        &http,
        KLEND,
        Some(public("https://github.com/someone-else/klend", None)),
    )
    .await;
    match t {
        CodeTransparency::Verified {
            warning: Some(w), ..
        } => assert!(w.contains("someone-else"), "{w}"),
        other => panic!("{other:?}"),
    }
    let t = status(&http, KLEND, Some(none_found())).await;
    assert!(
        matches!(
            t,
            CodeTransparency::Verified {
                warning: Some(_),
                ..
            }
        ),
        "{t:?}"
    );
}

#[tokio::test]
async fn col_13_repo_match_ignores_case_and_trailing_slash() {
    let http = Fixtures::new(vec![]);
    let t = status(
        &http,
        KLEND,
        Some(public("https://GitHub.com/kamino-finance/KLEND/", None)),
    )
    .await;
    assert!(
        matches!(t, CodeTransparency::Verified { warning: None, .. }),
        "{t:?}"
    );
}

#[tokio::test]
async fn col_13_unverified_program_uses_its_registry_source() {
    let http = Fixtures::new(vec![]);
    let commit = "63705a2dd9e8647fcfba9bfe1c411b049c209e9c";
    for program in [DRIFT, RAYDIUM] {
        let t = status(
            &http,
            program,
            Some(public("https://github.com/org/repo", Some(commit))),
        )
        .await;
        match t {
            CodeTransparency::PublicSourceUnverified {
                repo,
                commit: c,
                on_chain_hash,
            } => {
                assert_eq!(repo, "https://github.com/org/repo");
                assert_eq!(c.as_deref(), Some(commit));
                assert_eq!(on_chain_hash.unwrap().len(), 64);
            }
            other => panic!("{program}: {other:?}"),
        }
    }
}

#[tokio::test]
async fn col_13_no_protocol_org_fallback_for_unverified_programs() {
    let http = Fixtures::new(vec![]);
    let t = status(&http, MEMO, Some(none_found())).await;
    assert_eq!(t, CodeTransparency::NoPublicSource);
    let t = status(&http, MEMO, None).await;
    assert!(matches!(t, CodeTransparency::Unavailable { .. }), "{t:?}");
}

#[tokio::test(start_paused = true)]
async fn col_13_transient_errors_are_retried_then_unavailable() {
    let http = Fixtures::new(vec![TransportError::Status(503)]);
    let t = status(&http, KLEND, None).await;
    assert!(
        matches!(t, CodeTransparency::Verified { .. }),
        "one 503 then success: {t:?}"
    );

    let http = Fixtures::new(vec![TransportError::Timeout; 5]);
    let t = status(&http, KLEND, None).await;
    assert!(matches!(t, CodeTransparency::Unavailable { .. }), "{t:?}");
}

#[tokio::test]
async fn col_13_malformed_response_is_unavailable() {
    struct Garbage;
    impl HttpGet for Garbage {
        async fn get_json(&self, _: &str) -> Result<Value, TransportError> {
            Ok(serde_json::json!({"unexpected": true}))
        }
    }
    let fact = read_verification(&Garbage, &Address::new(KLEND), None, ev(KLEND)).await;
    assert!(matches!(fact.value(), CodeTransparency::Unavailable { .. }));
    assert_eq!(fact.evidence()[0], ev(KLEND));
}
