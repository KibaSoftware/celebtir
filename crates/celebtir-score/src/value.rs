//! Value by control type (VAL-2): each protocol's TVL is assigned to the
//! control class of its weakest core program. Per-program TVL is not
//! available from free sources, so the weakest-link attribution is a stated
//! simplification (Decisions table).

use std::collections::BTreeMap;

use celebtir_core::Controller;
use celebtir_core::snapshot::{FactsFile, ProgramRole};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::factors::effective_delay;

/// Ordered weakest first.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ControlClass {
    SingleKey,
    /// Controllers Celebtir cannot read or does not support yet.
    Unrecognized,
    /// Effective delay of zero: no time lock, or one a config authority
    /// bypasses (v0.3; before v0.3 also any delay below the bar).
    MultisigWithoutDelay,
    /// v0.3 (D4 option B): a delay above zero but below the delayed bar.
    MultisigShortDelay,
    MultisigWithDelay,
    Immutable,
}

impl ControlClass {
    /// The five classes of v0.1 and v0.2.
    pub const ALL: [ControlClass; 5] = [
        ControlClass::SingleKey,
        ControlClass::Unrecognized,
        ControlClass::MultisigWithoutDelay,
        ControlClass::MultisigWithDelay,
        ControlClass::Immutable,
    ];

    /// The six classes from v0.3.
    pub const V03: [ControlClass; 6] = [
        ControlClass::SingleKey,
        ControlClass::Unrecognized,
        ControlClass::MultisigWithoutDelay,
        ControlClass::MultisigShortDelay,
        ControlClass::MultisigWithDelay,
        ControlClass::Immutable,
    ];

    /// The classes a methodology publishes, so older days keep their keys.
    pub fn for_methodology(m: &crate::Methodology) -> &'static [ControlClass] {
        if m.at_least(0, 3) {
            &Self::V03
        } else {
            &Self::ALL
        }
    }
}

pub fn control_class(c: &Controller) -> ControlClass {
    control_class_at_delay(c, 1)
}

pub fn control_class_with_methodology(c: &Controller, m: &crate::Methodology) -> ControlClass {
    if let Some(v) = m.value_classes.as_ref().filter(|_| m.at_least(0, 3)) {
        return control_class_v03(c, v.delayed_min_secs);
    }
    control_class_at_delay(
        c,
        if !m.at_least(0, 2) {
            1
        } else {
            m.upgrade_delay.medium_min_secs
        },
    )
}

/// v0.3 (D4 option B): no delay, a delay below `minimum`, or `minimum`+.
fn control_class_v03(c: &Controller, minimum: u64) -> ControlClass {
    let by_delay = |c: &Controller| match effective_delay(c) {
        None | Some(0) => ControlClass::MultisigWithoutDelay,
        Some(d) if d < minimum => ControlClass::MultisigShortDelay,
        Some(_) => ControlClass::MultisigWithDelay,
    };
    match c {
        Controller::Immutable => ControlClass::Immutable,
        Controller::ProgramControlled { upgrade, .. } => control_class_v03(upgrade, minimum),
        Controller::SingleKey { .. } => ControlClass::SingleKey,
        Controller::Unrecognized { .. }
        | Controller::Pda { .. }
        | Controller::Unavailable { .. }
        | Controller::CannotAct { .. } => ControlClass::Unrecognized,
        Controller::Timelock {
            proposer, admin, ..
        } => {
            let own = control_class_v03(proposer, minimum).min(by_delay(c));
            match admin {
                Some(a) => own.min(control_class_v03(a, minimum)),
                None => own,
            }
        }
        Controller::Multisig {
            config_authority, ..
        } => {
            let own = by_delay(c);
            match config_authority {
                Some(ca) => own.min(control_class_v03(ca, minimum)),
                None => own,
            }
        }
    }
}

fn control_class_at_delay(c: &Controller, minimum: u64) -> ControlClass {
    match c {
        Controller::Immutable => ControlClass::Immutable,
        Controller::ProgramControlled { upgrade, .. } => control_class_at_delay(upgrade, minimum),
        Controller::SingleKey { .. } => ControlClass::SingleKey,
        Controller::Unrecognized { .. }
        | Controller::Pda { .. }
        | Controller::Unavailable { .. }
        | Controller::CannotAct { .. } => ControlClass::Unrecognized,
        Controller::Timelock {
            proposer, admin, ..
        } => {
            let ceiling = if minimum == 1 || effective_delay(c).is_some_and(|d| d >= minimum) {
                ControlClass::MultisigWithDelay
            } else {
                ControlClass::MultisigWithoutDelay
            };
            let own = control_class_at_delay(proposer, minimum).min(ceiling);
            match admin {
                Some(a) => own.min(control_class_at_delay(a, minimum)),
                None => own,
            }
        }
        Controller::Multisig {
            config_authority, ..
        } => {
            let own = if effective_delay(c).is_some_and(|d| d >= minimum) {
                ControlClass::MultisigWithDelay
            } else {
                ControlClass::MultisigWithoutDelay
            };
            // ADR 005: the config authority can act for the multisig.
            match config_authority {
                Some(ca) => own.min(control_class_at_delay(ca, minimum)),
                None => own,
            }
        }
    }
}

/// TVL per class; every class present, `total` the sum.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ValueRow {
    pub classes: BTreeMap<ControlClass, u64>,
    pub total: u64,
}

impl ValueRow {
    /// Every class of the given set at zero.
    pub fn empty_of(classes: &[ControlClass]) -> Self {
        Self {
            classes: classes.iter().map(|c| (*c, 0)).collect(),
            total: 0,
        }
    }

    fn add(&mut self, class: ControlClass, usd: u64) {
        *self.classes.entry(class).or_default() += usd;
        self.total += usd;
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ValueByControl {
    pub all: ValueRow,
    /// Keyed by registry category.
    pub by_category: BTreeMap<String, ValueRow>,
    /// Protocols left out because their TVL is unavailable.
    pub excluded: Vec<String>,
}

pub fn value_by_control(facts: &FactsFile) -> ValueByControl {
    value_by_control_using(facts, &ControlClass::ALL, control_class)
}

pub fn value_by_control_with_methodology(
    facts: &FactsFile,
    m: &crate::Methodology,
) -> ValueByControl {
    value_by_control_using(facts, ControlClass::for_methodology(m), |c| {
        control_class_with_methodology(c, m)
    })
}

fn value_by_control_using(
    facts: &FactsFile,
    classes: &[ControlClass],
    classify: impl Fn(&Controller) -> ControlClass,
) -> ValueByControl {
    let mut out = ValueByControl {
        all: ValueRow::empty_of(classes),
        ..ValueByControl::default()
    };
    for (id, p) in &facts.protocols {
        let Some(tvl) = facts.metrics.get(id).and_then(|m| m.tvl.usd()) else {
            out.excluded.push(id.clone());
            continue;
        };
        let class = p
            .programs
            .values()
            .filter(|f| f.role == ProgramRole::Core)
            .map(|f| classify(f.upgrade_control.value()))
            .min()
            .unwrap_or(ControlClass::Unrecognized);
        out.all.add(class, tvl);
        let category = p.category.clone().unwrap_or_else(|| "uncategorized".into());
        out.by_category
            .entry(category)
            .or_insert_with(|| ValueRow::empty_of(classes))
            .add(class, tvl);
    }
    out
}
