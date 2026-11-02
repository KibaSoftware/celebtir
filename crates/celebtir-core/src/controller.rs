use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::Address;

/// Who can change something, resolved to its effective controller.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Controller {
    Immutable,
    /// A registry-declared PDA re-derived from its program and typed seeds.
    ProgramControlled {
        key: Address,
        program: Address,
        upgrade: Box<Controller>,
    },
    SingleKey {
        key: Address,
    },
    Multisig {
        #[serde(rename = "multisig_kind")]
        kind: MultisigKind,
        address: Address,
        threshold: u16,
        voters: u16,
        time_lock_secs: u32,
        config_authority: Option<Box<Controller>>,
        /// Keys of the members with the Vote permission, sorted (UI-20).
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        signers: Vec<Address>,
        /// Signers that are not plain wallet keys (another multisig's vault,
        /// or an unknown program-derived address), resolved; each still
        /// counts as one voter (edge case 5).
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        nested_signers: BTreeMap<Address, Box<Controller>>,
    },
    /// EVM, wave 2.
    Timelock {
        address: Address,
        min_delay_secs: u64,
        proposer: Box<Controller>,
        /// A `DEFAULT_ADMIN_ROLE` holder other than the timelock itself: it
        /// can change roles and the delay without waiting (ADR 024 D4).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        admin: Option<Box<Controller>>,
    },
    /// A multisig that can never reach its threshold (ADR 004).
    CannotAct {
        address: Address,
        threshold: u16,
        voters: u16,
    },
    Unrecognized {
        key: Address,
        owner_program: Option<Address>,
    },
    /// An off-curve address (no private key can exist for it) whose
    /// controlling program is not resolved by a verified registry
    /// declaration. Never a key, never "unrecognized" (methodology v0.3).
    Pda {
        key: Address,
        /// Owner of the account at that address; None when no account exists.
        owner_program: Option<Address>,
    },
    Unavailable {
        reason: String,
    },
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum MultisigKind {
    SquadsV4,
    SquadsV3,
    /// EVM, wave 2.
    Safe,
}
