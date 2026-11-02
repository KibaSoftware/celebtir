//! The versioned methodology file. Every number the scorer compares against
//! comes from here (SCO-2).

use serde::Deserialize;

/// The default methodology (`methodology/v0.3.toml`), embedded at compile
/// time so scoring needs no file access (SCO-1). The crate keeps its own
/// copies so it packages for crates.io (CLI-5); a test checks they match the
/// published files.
pub const SHIPPED_METHODOLOGY: &str = include_str!("../methodology/v0.3.toml");

/// Every published version, for explicit replay (SNAP-4).
const BUNDLED: [(&str, &str); 3] = [
    ("0.1", include_str!("../methodology/v0.1.toml")),
    ("0.2", include_str!("../methodology/v0.2.toml")),
    ("0.3", include_str!("../methodology/v0.3.toml")),
];

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MethodologyError {
    #[error("methodology file does not parse: {0}")]
    Parse(String),
    #[error("methodology thresholds are inconsistent: {0}")]
    Inconsistent(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Methodology {
    pub version: String,
    pub upgrade_control: UpgradeControl,
    pub upgrade_delay: UpgradeDelay,
    pub admin_powers: AdminPowers,
    pub oracle_dependency: OracleDependency,
    pub stage1: Stage1,
    pub stage2: Stage2,
    /// v0.3+: value-class thresholds (D4).
    #[serde(default)]
    pub value_classes: Option<ValueClasses>,
    /// v0.3+: which chains are rated (F5).
    #[serde(default)]
    pub coverage: Option<Coverage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValueClasses {
    /// A multisig or timelock delay at or above this counts as delayed.
    pub delayed_min_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Coverage {
    pub assessed_chains: Vec<celebtir_core::Chain>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpgradeControl {
    pub low_min_voters: u16,
    pub low_min_threshold_pct: u16,
    pub medium_min_voters: u16,
    pub medium_min_threshold_pct: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpgradeDelay {
    pub low_min_secs: u64,
    pub medium_min_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminPowers {
    pub low_min_time_lock_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OracleDependency {
    pub established_networks: Vec<String>,
    #[serde(default)]
    pub required_for_categories: Vec<String>,
    /// v0.3+: distinct established networks that must bound a primary
    /// outside `established_networks` for it to rate Low (D3).
    #[serde(default)]
    pub guarded_primary_min_networks: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stage1 {
    pub min_upgrade_delay_secs: u64,
    pub emergency_min_voters: u16,
    pub emergency_min_threshold_pct: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stage2 {
    pub governance_min_delay_secs: u64,
}

impl Methodology {
    pub fn from_toml(text: &str) -> Result<Self, MethodologyError> {
        toml::from_str(text).map_err(|e| MethodologyError::Parse(e.to_string()))
    }

    /// The methodology this build ships with.
    pub fn shipped() -> Self {
        let m = Self::from_toml(SHIPPED_METHODOLOGY).expect("the shipped methodology parses");
        m.check().expect("the shipped methodology is consistent");
        m
    }

    /// A published version by number (`"0.1"`, `"0.2"`, `"0.3"`).
    pub fn bundled(version: &str) -> Option<Self> {
        BUNDLED
            .iter()
            .find(|(v, _)| *v == version)
            .map(|(_, text)| Self::from_toml(text).expect("bundled methodologies parse"))
    }

    /// True when this file is `major.minor` or later. A version that does not
    /// start with `MAJOR.MINOR` (a test file) counts as the latest rules.
    pub fn at_least(&self, major: u16, minor: u16) -> bool {
        let mut parts = self.version.split('.');
        let parsed = (|| {
            let a: u16 = parts.next()?.parse().ok()?;
            let b: u16 = parts
                .next()?
                .split(|c: char| !c.is_ascii_digit())
                .next()?
                .parse()
                .ok()?;
            Some((a, b))
        })();
        parsed.is_none_or(|v| v >= (major, minor))
    }

    /// Is a protocol on `chain` rated under this methodology? Before v0.3
    /// every chain was.
    pub fn assessed(&self, chain: celebtir_core::Chain) -> bool {
        self.coverage
            .as_ref()
            .is_none_or(|c| c.assessed_chains.contains(&chain))
    }

    /// The `scores.json` format this methodology writes: v0.3 added a value
    /// class, registry sources and the unassessed list.
    pub fn scores_schema_version(&self) -> &'static str {
        if self.at_least(0, 3) {
            "0.2.0"
        } else {
            "0.1.0"
        }
    }

    /// Rejects files whose Low bar is weaker than their Medium bar, or whose
    /// percentages are out of range.
    pub fn check(&self) -> Result<(), MethodologyError> {
        let bad = |msg: &str| Err(MethodologyError::Inconsistent(msg.to_string()));
        let uc = &self.upgrade_control;
        for pct in [
            uc.low_min_threshold_pct,
            uc.medium_min_threshold_pct,
            self.stage1.emergency_min_threshold_pct,
        ] {
            if pct == 0 || pct > 100 {
                return bad("threshold percentages must be between 1 and 100");
            }
        }
        if uc.low_min_voters < uc.medium_min_voters
            || uc.low_min_threshold_pct < uc.medium_min_threshold_pct
        {
            return bad("upgrade_control: low must be at least as strict as medium");
        }
        if self.upgrade_delay.low_min_secs < self.upgrade_delay.medium_min_secs {
            return bad("upgrade_delay: low_min_secs must be >= medium_min_secs");
        }
        if self.oracle_dependency.established_networks.is_empty() {
            return bad("oracle_dependency: established_networks is empty");
        }
        if self.stage2.governance_min_delay_secs < self.stage1.min_upgrade_delay_secs {
            return bad("stage2 delay must be >= stage1 delay");
        }
        let v03 = [
            self.value_classes.is_some(),
            self.coverage.is_some(),
            self.oracle_dependency
                .guarded_primary_min_networks
                .is_some(),
        ];
        if self.at_least(0, 3) && self.version.starts_with("0.") {
            if v03.iter().any(|x| !x) {
                return bad("v0.3+ needs value_classes, coverage and guarded_primary_min_networks");
            }
        } else if !self.at_least(0, 3) && v03.iter().any(|x| *x) {
            return bad("value_classes, coverage and guarded_primary_min_networks start in v0.3");
        }
        if self
            .value_classes
            .as_ref()
            .is_some_and(|v| v.delayed_min_secs == 0)
        {
            return bad("value_classes: delayed_min_secs must be above zero");
        }
        if self
            .oracle_dependency
            .guarded_primary_min_networks
            .is_some_and(|n| n < 2)
        {
            return bad("oracle_dependency: guarded_primary_min_networks must be at least 2");
        }
        if self
            .coverage
            .as_ref()
            .is_some_and(|c| c.assessed_chains.is_empty())
        {
            return bad("coverage: assessed_chains is empty");
        }
        Ok(())
    }
}

/// `threshold / voters >= pct%`, in integers so 3 of 5 is exactly 60 %.
pub fn meets_ratio(threshold: u16, voters: u16, pct: u16) -> bool {
    voters > 0 && u32::from(threshold) * 100 >= u32::from(pct) * u32::from(voters)
}
