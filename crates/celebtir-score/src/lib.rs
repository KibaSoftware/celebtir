//! Scorer: a pure function from facts and a methodology file to ratings and a
//! Stage. No I/O, no clock (SCO-1).

pub mod factors;
mod legacy;
mod lint;
mod methodology;
mod protocol;
mod rating;
pub mod sentences;
mod stage;
mod token;
pub mod upgrades;
pub mod value;

pub use methodology::{
    AdminPowers, Coverage, Methodology, MethodologyError, OracleDependency, SHIPPED_METHODOLOGY,
    Stage1, Stage2, UpgradeControl, UpgradeDelay, ValueClasses, meets_ratio,
};
pub use protocol::{
    Factor, FactorScore, HIP3_UNRATED, NodeRating, ProgramScore, ProtocolScore,
    SCORES_SCHEMA_VERSION, ScoresFile, TokenExposure, UnratedEntry, score, score_protocol,
};
pub use rating::{Assessment, Rating, weakest};
pub use sentences::{RiskCategory, RiskSentence, Severity};
pub use stage::{Requirement, Stage, StageResult};
pub use token::{TokenFactor, TokenRisk, TokenScore, score_token};

/// The published JSON Schema for `scores.json`.
pub fn scores_schema() -> schemars::Schema {
    schemars::schema_for!(ScoresFile)
}
