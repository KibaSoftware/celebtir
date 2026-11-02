//! Registry file format (one YAML file per protocol, plus `tokens.yaml`) and
//! its validation.

pub mod evm;
mod model;
mod validate;

pub use model::{
    AdminAccountEntry, AdminPower, ArchivedEntry, ArchivedFile, AuthorityPda, Category,
    DependencyEntry, EvmAdminEntry, EvmController, EvmControllerKind, EvmOracle, Hip3DexEntry,
    Hip3File, IdlOrigin, IdlOrigins, Issuer, MultisigEntry, MultisigProgram, OracleEntry,
    OracleRole, PdaSeed, ProgramEntry, ProgramSource, ProgramsFile, ProtocolEntry, Provenance,
    ProxyKind, Review, ReviewedNone, SafeEntry, ScopeChain, ScopeReserve, SourceCode,
    StakePoolEntry, TimelockEntry, TokenEntry, TokenKind, TokenRole, TokenUse, TokensFile,
};
pub use validate::{Diagnostic, Report, check_commit, check_repo_url, validate};

/// The published JSON Schema for `registry/solana/<id>.yaml`.
pub fn protocol_schema() -> serde_json::Value {
    serde_json::to_value(schemars::schema_for!(ProtocolEntry)).expect("schemas serialize")
}

/// The published JSON Schema for `registry/archived.yaml`.
pub fn archived_schema() -> serde_json::Value {
    serde_json::to_value(schemars::schema_for!(ArchivedFile)).expect("schemas serialize")
}

/// The published JSON Schema for `registry/programs.yaml`.
pub fn programs_schema() -> serde_json::Value {
    serde_json::to_value(schemars::schema_for!(ProgramsFile)).expect("schemas serialize")
}

/// The published JSON Schema for `registry/tokens.yaml`.
pub fn tokens_schema() -> serde_json::Value {
    serde_json::to_value(schemars::schema_for!(TokensFile)).expect("schemas serialize")
}
