//! Verified builds (COL-13) from the OtterSec Verified Programs API:
//! `GET https://verify.osec.io/status/<program_id>`. Celebtir trusts OtterSec's
//! on-chain hash in iteration 1 (Decisions table); recomputing it is
//! iteration 2.

use std::future::Future;
use std::time::Duration;

use celebtir_core::snapshot::CodeTransparency;
use celebtir_core::{Address, Evidence, Fact};
use celebtir_registry::{ProgramSource, SourceCode};
use serde_json::Value;

use crate::rpc::TransportError;

pub const OTTERSEC_STATUS_URL: &str = "https://verify.osec.io/status/";
const ATTEMPTS: u32 = 3;

/// A JSON GET, so tests can serve recorded responses.
pub trait HttpGet: Send + Sync {
    fn get_json(&self, url: &str) -> impl Future<Output = Result<Value, TransportError>> + Send;
}

pub struct ReqwestGet(reqwest::Client);

impl Default for ReqwestGet {
    fn default() -> Self {
        Self(reqwest::Client::new())
    }
}

impl HttpGet for ReqwestGet {
    async fn get_json(&self, url: &str) -> Result<Value, TransportError> {
        let r = self
            .0
            .get(url)
            .timeout(Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    TransportError::Timeout
                } else {
                    TransportError::Network(e.without_url().to_string())
                }
            })?;
        let status = r.status().as_u16();
        if !(200..300).contains(&status) {
            return Err(TransportError::Status(status));
        }
        r.json()
            .await
            .map_err(|e| TransportError::Decode(e.without_url().to_string()))
    }
}

fn retryable(e: &TransportError) -> bool {
    matches!(e, TransportError::Status(s) if *s == 429 || *s >= 500)
        || matches!(e, TransportError::Timeout | TransportError::Network(_))
}

/// `https://github.com/Org/Repo/tree/<ref>` → (`Org/Repo`, Some(ref)).
fn repo_and_ref(url: &str) -> Option<(String, Option<String>)> {
    let rest = url
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let mut parts = rest.split('/').filter(|p| !p.is_empty());
    let host = parts.next()?.to_ascii_lowercase();
    if !matches!(
        host.as_str(),
        "github.com" | "www.github.com" | "gitlab.com"
    ) {
        return None;
    }
    let owner = parts.next()?;
    let repo = parts.next()?.trim_end_matches(".git");
    let reference = match parts.next() {
        Some("tree" | "commit") => parts.next().map(str::to_string),
        _ => None,
    };
    Some((format!("{owner}/{repo}"), reference))
}

fn full_commit(c: &str) -> bool {
    c.len() == 40 && c.bytes().all(|b| b.is_ascii_hexdigit())
}

fn non_empty(v: &Value, key: &str) -> Option<String> {
    v.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// The verification status of one program, keyed by program ID (F1).
/// `source` is the program's `registry/programs.yaml` entry: the public
/// repository (or a sourced finding that none exists), never a protocol's
/// GitHub org. `origin` is evidence for the program read the status
/// belongs to.
pub async fn read_verification<H: HttpGet>(
    http: &H,
    program: &Address,
    source: Option<&ProgramSource>,
    origin: Evidence,
) -> Fact<CodeTransparency> {
    Fact::single(verification_status(http, program, source).await, origin)
}

/// The status alone, so a run can reuse it for every protocol that shares
/// the program.
pub async fn verification_status<H: HttpGet>(
    http: &H,
    program: &Address,
    source: Option<&ProgramSource>,
) -> CodeTransparency {
    let url = format!("{OTTERSEC_STATUS_URL}{program}");
    match get_with_retry(http, &url).await {
        Err(last) => CodeTransparency::Unavailable {
            reason: format!("OtterSec status unavailable: {last}"),
        },
        Ok(v) => interpret(&v, source),
    }
}

fn interpret(v: &Value, source: Option<&ProgramSource>) -> CodeTransparency {
    let Some(verified) = v.get("is_verified").and_then(Value::as_bool) else {
        return CodeTransparency::Unavailable {
            reason: "OtterSec response has no is_verified flag".into(),
        };
    };
    let registry = source.map(|s| &s.source_code);
    if verified {
        let reported = non_empty(v, "repo_url").unwrap_or_default();
        let parsed = repo_and_ref(&reported);
        let mut commit = non_empty(v, "commit").unwrap_or_default();
        if commit.is_empty() {
            commit = parsed
                .as_ref()
                .and_then(|(_, r)| r.clone())
                .unwrap_or_default();
        }
        // A truncated commit is not a commit link (H5).
        if !full_commit(&commit) {
            commit = String::new();
        }
        let repo = match &parsed {
            Some((name, _)) => format!("https://github.com/{name}"),
            None => reported.clone(),
        };
        let warning = match (registry, &parsed) {
            (Some(SourceCode::Public { repo: expected, .. }), Some((found, _))) => {
                match repo_and_ref(expected.as_str()) {
                    Some((want, _)) if !want.eq_ignore_ascii_case(found) => Some(format!(
                        "verified repo {repo} differs from the registry's source {expected}"
                    )),
                    _ => None,
                }
            }
            (Some(SourceCode::NoneFound { .. }), _) => Some(format!(
                "OtterSec reports a verified build from {repo}, but the registry found no public source"
            )),
            (_, None) => Some(format!(
                "verified repo {reported:?} is not a GitHub repository"
            )),
            (None, _) => Some("program has no registry source entry".into()),
        };
        return CodeTransparency::Verified {
            repo,
            commit,
            on_chain_hash: non_empty(v, "on_chain_hash").unwrap_or_default(),
            executable_hash: non_empty(v, "executable_hash").unwrap_or_default(),
            warning,
        };
    }
    match registry {
        Some(SourceCode::Public { repo, commit, .. }) => CodeTransparency::PublicSourceUnverified {
            repo: repo.as_str().trim_end_matches('/').to_string(),
            commit: commit.clone(),
            on_chain_hash: non_empty(v, "on_chain_hash"),
        },
        Some(SourceCode::NoneFound { .. }) => CodeTransparency::NoPublicSource,
        None => CodeTransparency::Unavailable {
            reason: "the program has no registry/programs.yaml source entry".into(),
        },
    }
}

/// A GET with up to 3 attempts on 429, 5xx, timeouts and network errors.
pub async fn get_with_retry<H: HttpGet>(http: &H, url: &str) -> Result<Value, TransportError> {
    let mut attempt = 1;
    loop {
        match http.get_json(url).await {
            Ok(v) => return Ok(v),
            Err(e) if retryable(&e) && attempt < ATTEMPTS => {
                tokio::time::sleep(Duration::from_secs(u64::from(attempt))).await;
                attempt += 1;
            }
            Err(e) => return Err(e),
        }
    }
}
