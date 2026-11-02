//! Chain-neutral facts model. Collectors emit these types; the scorer and the
//! site read only these, never raw chain data.

pub mod activity;
pub mod budget;
mod canonical;
mod controller;
pub mod date;
mod evidence;
mod fact;
mod rpc;
pub mod schema;
pub mod snapshot;

pub use canonical::to_canonical_json;
pub use controller::{Controller, MultisigKind};
pub use evidence::{Address, Chain, Evidence};
pub use fact::{Fact, FactError};
pub use rpc::RpcEndpoint;
