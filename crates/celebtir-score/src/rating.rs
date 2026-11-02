use celebtir_core::Evidence;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A factor rating. `Unknown` (missing data, SCO-7) and `NotApplicable` are
/// shown separately from High; neither is a risk level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Rating {
    Low,
    Medium,
    High,
    Unknown,
    NotApplicable,
}

impl Rating {
    /// Position on the risk scale; None for Unknown and NotApplicable.
    pub fn level(self) -> Option<u8> {
        match self {
            Rating::Low => Some(0),
            Rating::Medium => Some(1),
            Rating::High => Some(2),
            Rating::Unknown | Rating::NotApplicable => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Rating::Low => "LOW",
            Rating::Medium => "MEDIUM",
            Rating::High => "HIGH",
            Rating::Unknown => "UNKNOWN",
            Rating::NotApplicable => "N/A",
        }
    }
}

/// The weakest of several ratings. High wins outright, because missing data
/// cannot make it better. Otherwise any Unknown makes the result Unknown, since
/// the missing part could be worse than what is known. NotApplicable entries
/// are ignored. Returns None when nothing applies.
pub fn weakest(ratings: impl IntoIterator<Item = Rating>) -> Option<Rating> {
    let mut known: Option<Rating> = None;
    let mut unknown = false;
    for r in ratings {
        match r {
            Rating::High => return Some(Rating::High),
            Rating::Unknown => unknown = true,
            Rating::NotApplicable => {}
            r => {
                if known.is_none_or(|k| r.level() > k.level()) {
                    known = Some(r);
                }
            }
        }
    }
    if unknown {
        Some(Rating::Unknown)
    } else {
        known
    }
}

/// One factor's rating with the reason and the evidence behind it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assessment {
    pub rating: Rating,
    pub reason: String,
    /// What would raise the rating one level (SCO-5).
    pub next_step: String,
    pub evidence: Vec<Evidence>,
    /// Registry source pages behind a reviewed-none finding (v0.3).
    pub sources: Vec<String>,
}

impl Assessment {
    pub fn new(rating: Rating, reason: impl Into<String>, evidence: Vec<Evidence>) -> Self {
        Self {
            rating,
            reason: reason.into(),
            next_step: String::new(),
            evidence,
            sources: Vec::new(),
        }
    }

    pub fn with_sources(mut self, sources: Vec<String>) -> Self {
        self.sources = sources;
        self
    }

    pub fn with_next(mut self, next_step: impl Into<String>) -> Self {
        self.next_step = next_step.into();
        self
    }
}
