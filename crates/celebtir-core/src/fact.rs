use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::Evidence;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FactError {
    #[error("a fact must carry at least one evidence item")]
    NoEvidence,
}

/// A value together with the evidence it was derived from. The evidence list
/// is never empty (SNAP-1): construction and deserialization both reject it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(try_from = "RawFact<T>", bound(deserialize = "T: Deserialize<'de>"))]
pub struct Fact<T> {
    value: T,
    evidence: Vec<Evidence>,
}

impl<T> Fact<T> {
    pub fn new(value: T, evidence: Vec<Evidence>) -> Result<Self, FactError> {
        if evidence.is_empty() {
            return Err(FactError::NoEvidence);
        }
        Ok(Self { value, evidence })
    }

    pub fn single(value: T, evidence: Evidence) -> Self {
        Self {
            value,
            evidence: vec![evidence],
        }
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn evidence(&self) -> &[Evidence] {
        &self.evidence
    }
}

/// The wire shape of [`Fact`]; its schema is the published one.
#[derive(Deserialize, JsonSchema)]
struct RawFact<T> {
    value: T,
    #[schemars(length(min = 1))]
    evidence: Vec<Evidence>,
}

impl<T> TryFrom<RawFact<T>> for Fact<T> {
    type Error = FactError;

    fn try_from(raw: RawFact<T>) -> Result<Self, Self::Error> {
        Fact::new(raw.value, raw.evidence)
    }
}
