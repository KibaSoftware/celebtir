//! Offline registry validation: schema (REG-1, REG-3), structure (REG-5) and
//! vendored IDLs (REG-8), shared addresses (REG-4) and tokens (TOK-1, TOK-2). Chain checks (REG-2) live in the collector.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Component, Path, PathBuf};

use celebtir_core::snapshot::ProgramRole;
use jsonschema::Validator;
use jsonschema::error::ValidationErrorKind;
use serde::de::DeserializeOwned;
use serde_json::Value;

use celebtir_core::Chain;
use serde::Deserialize;

use crate::{
    ArchivedEntry, ArchivedFile, Category, EvmController, EvmOracle, IdlOrigins, ProgramSource,
    ProgramsFile, ProtocolEntry, SourceCode, TokenEntry, TokensFile, archived_schema, evm,
    programs_schema, protocol_schema, tokens_schema,
};

/// One problem, located by file (relative to the registry root) and JSON
/// pointer inside it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub file: PathBuf,
    pub pointer: String,
    pub message: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let file = self.file.to_string_lossy().replace('\\', "/");
        if self.pointer.is_empty() {
            write!(f, "{file}: {}", self.message)
        } else {
            write!(f, "{file}: {}: {}", self.pointer, self.message)
        }
    }
}

#[derive(Debug, Default)]
pub struct Report {
    /// Entries that parsed and matched the schema, sorted by ID.
    pub protocols: Vec<ProtocolEntry>,
    /// `hyperliquid/*.yaml` entries (HyperEVM, wave 2), sorted by ID. Kept
    /// apart so the Solana collector never sees them.
    pub hyperevm_protocols: Vec<ProtocolEntry>,
    /// Entries of `tokens.yaml`, in file order.
    pub tokens: Vec<TokenEntry>,
    /// Entries of `archived.yaml`, sorted by ID (ADR 017).
    pub archived: Vec<ArchivedEntry>,
    /// Entries of `programs.yaml`, keyed by program ID (F1).
    pub programs: BTreeMap<String, ProgramSource>,
    pub diagnostics: Vec<Diagnostic>,
}

impl Report {
    pub fn is_ok(&self) -> bool {
        self.diagnostics.is_empty()
    }

    fn error(&mut self, file: &Path, pointer: impl Into<String>, message: impl Into<String>) {
        self.diagnostics.push(Diagnostic {
            file: file.to_path_buf(),
            pointer: pointer.into(),
            message: message.into(),
        });
    }
}

const ORIGINS_FILE: &str = "idl/origins.yaml";
/// IDs that would collide with files the site data step writes.
const RESERVED_IDS: &[&str] = &["summary", "all"];

fn yaml_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|dir| {
            dir.flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "yaml"))
                .collect()
        })
        .unwrap_or_default();
    files.sort();
    files
}

/// Validates every `solana/*.yaml` and `hyperliquid/*.yaml` file under the
/// registry root, plus the shared files.
pub fn validate(root: &Path) -> Report {
    let mut report = Report::default();
    let validator = schema_validator(&protocol_schema());

    let origins = load_origins(root, &mut report);

    let mut parsed: Vec<(PathBuf, ProtocolEntry)> = Vec::new();
    for path in yaml_files(&root.join("solana")) {
        let rel = Path::new("solana").join(path.file_name().expect("listed files have names"));
        let Some(entry) = load_yaml::<ProtocolEntry>(&path, &rel, &validator, &mut report) else {
            continue;
        };
        check_entry(&entry, &path, &rel, root, origins.as_ref(), &mut report);
        parsed.push((rel, entry));
    }
    let mut evm_parsed: Vec<(PathBuf, ProtocolEntry)> = Vec::new();
    for path in yaml_files(&root.join("hyperliquid")) {
        let rel = Path::new("hyperliquid").join(path.file_name().expect("listed files have names"));
        // `hip3.yaml` is DEX metadata, not a protocol (HREG-4, HIP-4).
        if rel.file_name().is_some_and(|n| n == "hip3.yaml") {
            check_hip3(&path, &rel, &mut report);
            continue;
        }
        let Some(entry) = load_yaml::<ProtocolEntry>(&path, &rel, &validator, &mut report) else {
            continue;
        };
        check_evm_entry(&entry, &path, &rel, &mut report);
        evm_parsed.push((rel, entry));
    }

    let all: Vec<(PathBuf, ProtocolEntry)> =
        parsed.iter().chain(evm_parsed.iter()).cloned().collect();
    check_shared_addresses(&all, &mut report);
    check_tokens(root, &parsed, &mut report);
    check_programs(root, &parsed, &mut report);
    check_archived(root, &all, &mut report);
    check_evm_known(root, &mut report);
    report.protocols = parsed.into_iter().map(|(_, e)| e).collect();
    report.protocols.sort_by(|a, b| a.id.cmp(&b.id));
    report.hyperevm_protocols = evm_parsed.into_iter().map(|(_, e)| e).collect();
    report.hyperevm_protocols.sort_by(|a, b| a.id.cmp(&b.id));
    report
}

/// File-name, ID and core-program checks shared by both chains.
fn check_common(entry: &ProtocolEntry, path: &Path, rel: &Path, report: &mut Report) {
    if RESERVED_IDS.contains(&entry.id.as_str()) {
        report.error(
            rel,
            "/id",
            format!("`{}` is reserved for a site data file", entry.id),
        );
    }
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    if stem != entry.id {
        report.error(
            rel,
            "/id",
            format!(
                "id `{}` does not match the file name `{stem}.yaml`",
                entry.id
            ),
        );
    }
    if !entry.programs.iter().any(|p| p.role == ProgramRole::Core) {
        report.error(
            rel,
            "/programs",
            "at least one program must have role `core` (REG-5)",
        );
    }
}

/// HREG-1, HREG-2: a `hyperliquid/` file is HyperEVM, with EIP-55
/// addresses, a proxy kind per contract and proposers per timelock.
fn check_evm_entry(entry: &ProtocolEntry, path: &Path, rel: &Path, report: &mut Report) {
    check_common(entry, path, rel, report);
    if entry.chain != Chain::HyperEvm {
        report.error(
            rel,
            "/chain",
            "files in `hyperliquid/` must have `chain: hyper_evm`",
        );
    }
    let mut addresses: Vec<(String, &str)> = Vec::new();
    for (i, p) in entry.programs.iter().enumerate() {
        addresses.push((format!("/programs/{i}/id"), p.id.as_str()));
        if p.proxy.is_none() {
            report.error(
                rel,
                format!("/programs/{i}/proxy"),
                "HyperEVM contracts need `proxy: eip1967 | uups | none` (HREG-1)",
            );
        }
    }
    for (i, s) in entry.safes.iter().enumerate() {
        addresses.push((format!("/safes/{i}/address"), s.address.as_str()));
    }
    for (i, t) in entry.timelocks.iter().enumerate() {
        addresses.push((format!("/timelocks/{i}/address"), t.address.as_str()));
        if t.proposers.is_empty() {
            report.error(
                rel,
                format!("/timelocks/{i}/proposers"),
                "a timelock must name at least one proposer (HREG-2)",
            );
        }
        for (j, p) in t.proposers.iter().enumerate() {
            addresses.push((format!("/timelocks/{i}/proposers/{j}"), p.as_str()));
        }
    }
    for (i, o) in entry.oracles.iter().enumerate() {
        addresses.push((format!("/oracles/{i}/account"), o.account.as_str()));
    }
    for (i, a) in entry.evm_admins.iter().enumerate() {
        addresses.push((format!("/evm_admins/{i}/contract"), a.contract.as_str()));
        if let Some(h) = &a.holder {
            addresses.push((format!("/evm_admins/{i}/holder"), h.as_str()));
        }
        if a.role.is_some() != a.holder.is_some() {
            report.error(
                rel,
                format!("/evm_admins/{i}"),
                "`role` and `holder` go together (HCOL-7)",
            );
        }
    }
    for (i, d) in entry.dependencies.iter().enumerate() {
        addresses.push((format!("/dependencies/{i}/address"), d.address.as_str()));
    }
    for (pointer, address) in addresses {
        check_evm_address(address, rel, pointer, report);
    }
    for (field, empty) in [
        ("/multisigs", entry.multisigs.is_empty()),
        ("/admin_accounts", entry.admin_accounts.is_empty()),
        ("/tokens", entry.tokens.is_empty()),
    ] {
        if !empty {
            report.error(rel, field, "not supported for HyperEVM in wave 2 (ADR 024)");
        }
    }
}

/// HREG-1: `0x` + 40 hex digits in EIP-55 checksum case.
fn check_evm_address(address: &str, rel: &Path, pointer: String, report: &mut Report) {
    match evm::eip55(address) {
        None => report.error(
            rel,
            pointer,
            format!("`{address}` is not a 0x-prefixed 20-byte hex address (HREG-1)"),
        ),
        Some(c) if c != address => report.error(
            rel,
            pointer,
            format!("`{address}` is not EIP-55 checksummed; expected `{c}` (HREG-1)"),
        ),
        Some(_) => {}
    }
}

#[derive(Deserialize)]
struct EvmSection<T> {
    #[serde(default = "Vec::new")]
    evm: Vec<T>,
}

/// HREG-3: the `evm:` sections of `controllers.yaml` and `oracles.yaml` use
/// EIP-55 addresses, and no address appears twice across them.
fn check_evm_known(root: &Path, report: &mut Report) {
    let mut seen: BTreeMap<String, String> = BTreeMap::new();
    for file in ["controllers.yaml", "oracles.yaml"] {
        let rel = Path::new(file);
        let Ok(text) = std::fs::read_to_string(root.join(rel)) else {
            continue;
        };
        let addresses: Result<Vec<String>, String> = if file == "oracles.yaml" {
            serde_norway::from_str::<EvmSection<EvmOracle>>(&text)
                .map(|s| s.evm.into_iter().map(|e| e.address.to_string()).collect())
                .map_err(|e| e.to_string())
        } else {
            serde_norway::from_str::<EvmSection<EvmController>>(&text)
                .map(|s| s.evm.into_iter().map(|e| e.address.to_string()).collect())
                .map_err(|e| e.to_string())
        };
        let addresses = match addresses {
            Ok(a) => a,
            Err(e) => {
                report.error(rel, "/evm", e);
                continue;
            }
        };
        for (i, address) in addresses.into_iter().enumerate() {
            let pointer = format!("/evm/{i}/address");
            check_evm_address(&address, rel, pointer.clone(), report);
            if let Some(first) =
                seen.insert(address.to_ascii_lowercase(), format!("{file}{pointer}"))
            {
                report.error(
                    rel,
                    pointer,
                    format!("{address} is already listed at {first} (HREG-3)"),
                );
            }
        }
    }
}

fn check_entry(
    entry: &ProtocolEntry,
    path: &Path,
    rel: &Path,
    root: &Path,
    origins: Option<&IdlOrigins>,
    report: &mut Report,
) {
    check_pdas(&entry.authority_pdas, rel, "/authority_pdas", report);
    check_addresses(entry, rel, report);
    check_common(entry, path, rel, report);
    check_reviews(entry, rel, report);
    for (i, p) in entry.programs.iter().enumerate() {
        if let Some(why) = rejected_source(p.source.as_str()) {
            report.error(rel, format!("/programs/{i}/source"), why);
        }
    }
    if entry.chain != Chain::Solana {
        report.error(
            rel,
            "/chain",
            "files in `solana/` must have `chain: solana`",
        );
    }
    for (field, empty) in [
        ("/safes", entry.safes.is_empty()),
        ("/timelocks", entry.timelocks.is_empty()),
        ("/evm_admins", entry.evm_admins.is_empty()),
        ("/dependencies", entry.dependencies.is_empty()),
    ] {
        if !empty {
            report.error(rel, field, "HyperEVM only");
        }
    }
    for (i, p) in entry.programs.iter().enumerate() {
        if p.proxy.is_some() {
            report.error(
                rel,
                format!("/programs/{i}/proxy"),
                "`proxy` is HyperEVM only",
            );
        }
    }

    let idls = entry
        .admin_accounts
        .iter()
        .enumerate()
        .map(|(i, a)| (format!("/admin_accounts/{i}/idl"), &a.idl))
        .chain(entry.oracles.iter().enumerate().filter_map(|(i, o)| {
            o.scope_chain
                .as_ref()
                .map(|c| (format!("/oracles/{i}/scope_chain/idl"), &c.idl))
        }));
    for (pointer, idl) in idls {
        let Some(name) = idl_file_name(idl) else {
            report.error(
                rel,
                pointer,
                format!("`{idl}` must be a file directly inside `idl/`"),
            );
            continue;
        };
        if !root.join(idl).is_file() {
            report.error(rel, &pointer, format!("`{idl}` does not exist"));
        }
        if !origins.is_some_and(|o| o.contains_key(name)) {
            report.error(
                rel,
                &pointer,
                format!("`{idl}` has no recorded origin in `{ORIGINS_FILE}`"),
            );
        }
    }
}

/// `idl/<name>` → `<name>`; anything else (absolute, nested, `..`) → None.
fn idl_file_name(idl: &str) -> Option<&str> {
    let mut parts = Path::new(idl).components();
    match (parts.next(), parts.next(), parts.next()) {
        (Some(Component::Normal(dir)), Some(Component::Normal(name)), None) if dir == "idl" => {
            name.to_str()
        }
        _ => None,
    }
}

fn load_origins(root: &Path, report: &mut Report) -> Option<IdlOrigins> {
    let rel = Path::new(ORIGINS_FILE);
    let text = std::fs::read_to_string(root.join(rel)).ok()?;
    let origins: IdlOrigins = match serde_norway::from_str(&text) {
        Ok(o) => o,
        Err(e) => {
            report.error(rel, "", format!("invalid origins file: {e}"));
            return None;
        }
    };
    for (name, origin) in &origins {
        if origin.commit.is_none() && origin.slot.is_none() {
            report.error(
                rel,
                format!("/{name}"),
                format!("`{name}` must record the `commit` or `slot` it came from"),
            );
        }
    }
    Some(origins)
}

/// One place a protocol lists an address that may only be shared explicitly.
struct Listing<'a> {
    protocol: &'a str,
    file: &'a Path,
    pointer: String,
    shared_with: &'a [String],
}

/// REG-4: an address listed by two protocols must name the other protocol in
/// `shared_with` on both sides, and `shared_with` must not name a protocol
/// that does not list the address. Oracle feeds are exempt (ADR 010).
fn check_shared_addresses(parsed: &[(PathBuf, ProtocolEntry)], report: &mut Report) {
    let mut by_address: BTreeMap<&str, Vec<Listing<'_>>> = BTreeMap::new();
    for (file, entry) in parsed {
        let listings = entry
            .programs
            .iter()
            .enumerate()
            .map(|(i, p)| (p.id.as_str(), format!("/programs/{i}"), &p.shared_with[..]))
            .chain(entry.multisigs.iter().enumerate().map(|(i, m)| {
                (
                    m.address.as_str(),
                    format!("/multisigs/{i}"),
                    &m.shared_with[..],
                )
            }))
            .chain(entry.admin_accounts.iter().enumerate().map(|(i, a)| {
                (
                    a.account.as_str(),
                    format!("/admin_accounts/{i}"),
                    &a.shared_with[..],
                )
            }))
            .chain(entry.safes.iter().enumerate().map(|(i, s)| {
                (
                    s.address.as_str(),
                    format!("/safes/{i}"),
                    &s.shared_with[..],
                )
            }))
            .chain(entry.timelocks.iter().enumerate().map(|(i, t)| {
                (
                    t.address.as_str(),
                    format!("/timelocks/{i}"),
                    &t.shared_with[..],
                )
            }))
            .chain(entry.stake_pools.iter().enumerate().map(|(i, s)| {
                (
                    s.account.as_str(),
                    format!("/stake_pools/{i}"),
                    &s.shared_with[..],
                )
            }));
        for (address, pointer, shared_with) in listings {
            by_address.entry(address).or_default().push(Listing {
                protocol: &entry.id,
                file,
                pointer,
                shared_with,
            });
        }
    }

    for (address, listings) in &by_address {
        let listed_by: BTreeSet<&str> = listings.iter().map(|l| l.protocol).collect();
        for l in listings {
            for other in &listed_by {
                if *other != l.protocol && !l.shared_with.iter().any(|s| s == other) {
                    report.error(
                        l.file,
                        &l.pointer,
                        format!(
                            "{address} is also listed by `{other}`; add `{other}` to `shared_with` here and `{}` there (REG-4)",
                            l.protocol
                        ),
                    );
                }
            }
            for named in l.shared_with {
                if !listed_by.contains(named.as_str()) {
                    report.error(
                        l.file,
                        format!("{}/shared_with", l.pointer),
                        format!("`{named}` does not list {address} (REG-4)"),
                    );
                }
            }
        }
    }
}

const TOKENS_FILE: &str = "tokens.yaml";
const ARCHIVED_FILE: &str = "archived.yaml";

/// ADR 017: archived IDs are unique, dated and no longer active.
fn check_archived(root: &Path, parsed: &[(PathBuf, ProtocolEntry)], report: &mut Report) {
    let rel = Path::new(ARCHIVED_FILE);
    let path = root.join(rel);
    if !path.is_file() {
        return;
    }
    let Some(file) =
        load_yaml::<ArchivedFile>(&path, rel, &schema_validator(&archived_schema()), report)
    else {
        return;
    };
    let active: BTreeSet<&str> = parsed.iter().map(|(_, e)| e.id.as_str()).collect();
    let mut seen = BTreeSet::new();
    let mut ok = Vec::new();
    for (i, a) in file.archived.into_iter().enumerate() {
        let before = report.diagnostics.len();
        if !seen.insert(a.id.clone()) {
            report.error(
                rel,
                format!("/archived/{i}/id"),
                format!("duplicate id `{}`", a.id),
            );
        }
        if active.contains(a.id.as_str()) {
            report.error(
                rel,
                format!("/archived/{i}/id"),
                format!("`{}` is still an active protocol", a.id),
            );
        }
        if celebtir_core::date::days(&a.archived_on).is_none() {
            report.error(
                rel,
                format!("/archived/{i}/archived_on"),
                format!("`{}` is not a calendar date", a.archived_on),
            );
        }
        if report.diagnostics.len() == before {
            ok.push(a);
        }
    }
    ok.sort_by(|a, b| a.id.cmp(&b.id));
    report.archived = ok;
}

fn schema_validator(schema: &Value) -> Validator {
    jsonschema::options()
        .should_validate_formats(true)
        .build(schema)
        .expect("generated registry schemas are valid")
}

/// Reads a YAML file, checks it against `validator` and deserializes it.
/// Every problem is reported against `rel`; returns None if there was one.
fn load_yaml<T: DeserializeOwned>(
    path: &Path,
    rel: &Path,
    validator: &Validator,
    report: &mut Report,
) -> Option<T> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            report.error(rel, "", format!("cannot read: {e}"));
            return None;
        }
    };
    let value: Value = match serde_norway::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            report.error(rel, "", format!("invalid YAML: {e}"));
            return None;
        }
    };

    let before = report.diagnostics.len();
    for err in validator.iter_errors(&value) {
        let mut pointer = err.instance_path().to_string();
        if let ValidationErrorKind::Required { property } = err.kind() {
            if let Some(name) = property.as_str() {
                pointer = format!("{pointer}/{name}");
            }
        }
        report.error(rel, pointer, err.to_string());
    }
    if report.diagnostics.len() > before {
        return None;
    }

    match serde_json::from_value(value) {
        Ok(v) => Some(v),
        Err(e) => {
            report.error(rel, "", e.to_string());
            None
        }
    }
}

/// TOK-1 and TOK-2: `tokens.yaml` holds each mint once; every token a
/// protocol uses is registered; every registered token is used by a protocol.
fn check_tokens(root: &Path, parsed: &[(PathBuf, ProtocolEntry)], report: &mut Report) {
    let rel = Path::new(TOKENS_FILE);
    let path = root.join(rel);
    let tokens: Vec<TokenEntry> = if path.is_file() {
        load_yaml::<TokensFile>(&path, rel, &schema_validator(&tokens_schema()), report)
            .map(|f| f.tokens)
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let loaded = path.is_file() && !report.diagnostics.iter().any(|d| d.file == rel);

    let mut seen = BTreeSet::new();
    for (i, t) in tokens.iter().enumerate() {
        check_pdas(
            &t.authority_pdas,
            rel,
            &format!("/tokens/{i}/authority_pdas"),
            report,
        );
        if !seen.insert(t.mint.as_str()) {
            report.error(
                rel,
                format!("/tokens/{i}/mint"),
                format!("duplicate mint {}", t.mint),
            );
        }
    }

    let mut used = BTreeSet::new();
    for (file, entry) in parsed {
        for (i, u) in entry.tokens.iter().enumerate() {
            used.insert(u.mint.as_str());
            let registered = seen.contains(u.mint.as_str());
            if !path.is_file() || (loaded && !registered) {
                report.error(
                    file,
                    format!("/tokens/{i}/mint"),
                    format!("{} is not registered in `{TOKENS_FILE}` (TOK-2)", u.mint),
                );
            }
        }
    }

    for (i, t) in tokens.iter().enumerate() {
        if !used.contains(t.mint.as_str()) {
            report.error(
                rel,
                format!("/tokens/{i}"),
                format!(
                    "{} ({}) is used by no rated protocol, so it is out of scope (TOK-2)",
                    t.symbol, t.mint
                ),
            );
        }
    }
    report.tokens = tokens;
}

/// REG-2 (offline): `address` must decode from base58 to exactly 32 bytes.
fn check_address(address: &str, rel: &Path, pointer: String, report: &mut Report) {
    match bs58::decode(address).into_vec() {
        Err(_) => report.error(
            rel,
            pointer,
            format!("`{address}` is not valid base58 (REG-2)"),
        ),
        Ok(bytes) if bytes.len() != 32 => report.error(
            rel,
            pointer,
            format!(
                "`{address}` decodes to {} bytes, not 32 bytes (REG-2)",
                bytes.len()
            ),
        ),
        Ok(_) => {}
    }
}

fn check_addresses(entry: &ProtocolEntry, rel: &Path, report: &mut Report) {
    for (i, p) in entry.programs.iter().enumerate() {
        check_address(p.id.as_str(), rel, format!("/programs/{i}/id"), report);
    }
    for (i, m) in entry.multisigs.iter().enumerate() {
        check_address(
            m.address.as_str(),
            rel,
            format!("/multisigs/{i}/address"),
            report,
        );
    }
    for (i, a) in entry.admin_accounts.iter().enumerate() {
        check_address(
            a.account.as_str(),
            rel,
            format!("/admin_accounts/{i}/account"),
            report,
        );
    }
    for (i, o) in entry.oracles.iter().enumerate() {
        check_address(
            o.account.as_str(),
            rel,
            format!("/oracles/{i}/account"),
            report,
        );
        if let Some(c) = &o.scope_chain {
            check_address(
                c.account.as_str(),
                rel,
                format!("/oracles/{i}/scope_chain/account"),
                report,
            );
        }
    }
    for (i, t) in entry.tokens.iter().enumerate() {
        check_address(t.mint.as_str(), rel, format!("/tokens/{i}/mint"), report);
    }
    for (i, s) in entry.stake_pools.iter().enumerate() {
        check_address(
            s.account.as_str(),
            rel,
            format!("/stake_pools/{i}/account"),
            report,
        );
        check_address(
            s.pool_mint.as_str(),
            rel,
            format!("/stake_pools/{i}/pool_mint"),
            report,
        );
        if !entry.tokens.iter().any(|t| t.mint == s.pool_mint) {
            report.error(
                rel,
                format!("/stake_pools/{i}/pool_mint"),
                "the pool mint must be one of the protocol's tokens",
            );
        }
    }
}

/// F3: a reviewed-none statement must not contradict the entry, and
/// lending or perps can never be reviewed as using no prices.
fn check_reviews(entry: &ProtocolEntry, rel: &Path, report: &mut Report) {
    if let Some(r) = &entry.admin_reviewed_none {
        if !entry.admin_accounts.is_empty() || !entry.stake_pools.is_empty() {
            report.error(
                rel,
                "/admin_reviewed_none",
                "a review that found no admin accounts conflicts with listed admin accounts or stake pools",
            );
        }
        check_review_date(
            &r.reviewed_on,
            rel,
            "/admin_reviewed_none/reviewed_on",
            report,
        );
    }
    if let Some(r) = &entry.oracles_reviewed_none {
        if !entry.oracles.is_empty() {
            report.error(
                rel,
                "/oracles_reviewed_none",
                "a review that found no price feeds conflicts with listed oracles",
            );
        }
        if matches!(entry.category, Category::Lending | Category::Perps) {
            report.error(
                rel,
                "/oracles_reviewed_none",
                "lending and perps always depend on prices; register their feeds instead",
            );
        }
        check_review_date(
            &r.reviewed_on,
            rel,
            "/oracles_reviewed_none/reviewed_on",
            report,
        );
    }
}

fn check_review_date(date: &str, rel: &Path, pointer: &str, report: &mut Report) {
    if celebtir_core::date::days(date).is_none() {
        report.error(rel, pointer, format!("`{date}` is not a calendar date"));
    }
}

/// Repositories that restate another project's interface (IDL parsers, data
/// adapters, docs) are not a program's source (F1).
const DENIED_REPOS: &[&str] = &[
    "julianfssen/jupiter-perps-anchor-idl-parsing",
    "defillama/defillama-adapters",
    "pump-fun/pump-public-docs",
];

fn repo_path(url: &str) -> Option<(String, Vec<String>)> {
    let rest = url.strip_prefix("https://")?;
    let mut parts = rest.split('/').filter(|p| !p.is_empty());
    let host = parts.next()?.to_ascii_lowercase();
    Some((host, parts.map(str::to_string).collect()))
}

/// F1: `repo` must name one repository. Org pages, docs and README links,
/// explorers, IDL or parsing repos and adapters are rejected.
pub fn check_repo_url(url: &str) -> Result<(), String> {
    let Some((host, segments)) = repo_path(url) else {
        return Err(format!("`{url}` is not an https URL"));
    };
    if !matches!(host.as_str(), "github.com" | "gitlab.com") {
        return Err(format!(
            "`{url}` is not a GitHub or GitLab repository (explorers and docs are not source)"
        ));
    }
    if segments.len() != 2 {
        return Err(format!(
            "`{url}` must be exactly `https://{host}/<owner>/<repo>`: an org page, file, tree or docs link is not a repository"
        ));
    }
    let name = format!(
        "{}/{}",
        segments[0].to_ascii_lowercase(),
        segments[1].trim_end_matches(".git").to_ascii_lowercase()
    );
    if DENIED_REPOS.contains(&name.as_str()) {
        return Err(format!(
            "`{url}` restates the program's interface; it is not the program's source"
        ));
    }
    let repo = segments[1].to_ascii_lowercase();
    if ["idl", "parsing", "parser", "adapters"]
        .iter()
        .any(|w| repo.contains(w))
    {
        return Err(format!(
            "`{url}` looks like an IDL, parser or adapter repository, not program source"
        ));
    }
    Ok(())
}

/// F1: a commit is the full 40-character hex hash.
pub fn check_commit(commit: &str) -> Result<(), String> {
    if commit.len() == 40
        && commit
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err(format!(
            "`{commit}` is not a full 40-character lowercase commit hash"
        ))
    }
}

/// Sources that only restate another project's interface are rejected as
/// confirmation of a program ID too.
fn rejected_source(url: &str) -> Option<String> {
    let (host, segments) = repo_path(url)?;
    if host != "github.com" || segments.len() < 2 {
        return None;
    }
    let name = format!(
        "{}/{}",
        segments[0].to_ascii_lowercase(),
        segments[1].to_ascii_lowercase()
    );
    (name == DENIED_REPOS[0]).then(|| {
        format!("`{url}` is a third-party IDL parsing repository; cite the publisher's own page")
    })
}

const PROGRAMS_FILE: &str = "programs.yaml";

/// F1: every Solana program has exactly one `programs.yaml` entry, and every
/// public source is a repository (and full commit, when given).
fn check_programs(root: &Path, parsed: &[(PathBuf, ProtocolEntry)], report: &mut Report) {
    let rel = Path::new(PROGRAMS_FILE);
    let path = root.join(rel);
    // Optional so partial registries (tests, replays) validate; when present,
    // every Solana program must have exactly one entry. The published registry
    // is held to having it by `reg_1_published_registry_has_a_source_for_every_program`.
    if !path.is_file() {
        return;
    }
    let Some(file) =
        load_yaml::<ProgramsFile>(&path, rel, &schema_validator(&programs_schema()), report)
    else {
        return;
    };
    let mut by_id: BTreeMap<String, ProgramSource> = BTreeMap::new();
    for (i, p) in file.programs.into_iter().enumerate() {
        check_address(p.id.as_str(), rel, format!("/programs/{i}/id"), report);
        if let SourceCode::Public { repo, commit, .. } = &p.source_code {
            if let Err(e) = check_repo_url(repo.as_str()) {
                report.error(rel, format!("/programs/{i}/source_code/repo"), e);
            }
            if let Some(c) = commit {
                if let Err(e) = check_commit(c) {
                    report.error(rel, format!("/programs/{i}/source_code/commit"), e);
                }
            }
        }
        if by_id.insert(p.id.to_string(), p).is_some() {
            report.error(rel, format!("/programs/{i}/id"), "duplicate program ID");
        }
    }
    let used: BTreeSet<String> = parsed
        .iter()
        .flat_map(|(_, e)| e.programs.iter().map(|p| p.id.to_string()))
        .collect();
    for (file, entry) in parsed {
        for (i, p) in entry.programs.iter().enumerate() {
            if !by_id.contains_key(p.id.as_str()) {
                report.error(
                    file,
                    format!("/programs/{i}/id"),
                    format!("{} has no `{PROGRAMS_FILE}` source entry (F1)", p.id),
                );
            }
        }
    }
    for id in by_id.keys() {
        if !used.contains(id) {
            report.error(
                rel,
                "/programs",
                format!("{id} is listed by no protocol; remove it"),
            );
        }
    }
    report.programs = by_id;
}

fn check_hip3(path: &Path, rel: &Path, report: &mut Report) {
    let parsed = std::fs::read_to_string(path)
        .map_err(|e| e.to_string())
        .and_then(|t| serde_norway::from_str::<crate::Hip3File>(&t).map_err(|e| e.to_string()));
    match parsed {
        Err(e) => report.error(rel, "", e),
        Ok(f) if f.required_stake.parse::<f64>().is_err() => report.error(
            rel,
            "/required_stake",
            format!("`{}` is not a decimal number", f.required_stake),
        ),
        Ok(_) => {}
    }
}

fn check_pdas(declarations: &[crate::AuthorityPda], rel: &Path, prefix: &str, report: &mut Report) {
    let mut seen = BTreeSet::new();
    for (i, d) in declarations.iter().enumerate() {
        let at = format!("{prefix}/{i}");
        check_address(d.key.as_str(), rel, format!("{at}/key"), report);
        check_address(d.program.as_str(), rel, format!("{at}/program"), report);
        if !seen.insert(&d.key) {
            report.error(rel, &at, "duplicate PDA authority declaration");
        }
        if d.seeds.len() > 15 {
            report.error(
                rel,
                &at,
                "PDA declarations allow at most 15 seeds plus the derived bump",
            );
        }
        if !matches!(d.source.scheme(), "https" | "http") {
            report.error(rel, &at, "PDA source must be an HTTP(S) URL");
        }
        for (j, seed) in d.seeds.iter().enumerate() {
            match seed {
                crate::PdaSeed::Literal(s) if s.len() > 32 => {
                    report.error(rel, format!("{at}/seeds/{j}"), "PDA seed exceeds 32 bytes")
                }
                crate::PdaSeed::Address(a) => {
                    check_address(a.as_str(), rel, format!("{at}/seeds/{j}"), report)
                }
                _ => {}
            }
        }
    }
}
