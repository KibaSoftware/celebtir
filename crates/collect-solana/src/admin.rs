//! Admin keys held in protocol state accounts (COL-11): read each account
//! once, check its owner is one of the protocol's programs, decode the named
//! field with the vendored IDL, and resolve the key like an upgrade authority.
//! Any failure records `Unavailable` with the reason, never a default key.

use std::collections::BTreeMap;

use celebtir_core::snapshot::{AdminFact, AdminPower};
use celebtir_core::{Address, Chain, Controller, Evidence, Fact};

use crate::idl::Idl;
use crate::pda;
use crate::reads::SolanaRpc;
use crate::resolve::Resolver;
use crate::rpc::{RpcError, Transport};

/// One registry `admin_accounts` entry with its parsed IDL.
pub struct AdminSpec<'a> {
    pub account: Address,
    pub field: String,
    pub power: AdminPower,
    pub idl: &'a Idl,
}

/// Admin facts keyed `<account>.<field>`, from one batched read.
pub async fn read_admins<T: Transport>(
    rpc: &SolanaRpc<T>,
    resolver: &Resolver<'_, T>,
    specs: &[AdminSpec<'_>],
    protocol_programs: &[Address],
) -> Result<BTreeMap<String, AdminFact>, RpcError> {
    let accounts: Vec<Address> = specs.iter().map(|s| s.account.clone()).collect();
    let reads = rpc.get_multiple_accounts(&accounts, None).await?;
    let mut out = BTreeMap::new();
    for (spec, read) in specs.iter().zip(reads) {
        let evidence = Evidence::new(
            Chain::Solana,
            spec.account.clone(),
            read.slot,
            spec.field.clone(),
        );
        let unavailable =
            |reason: String| Fact::single(Controller::Unavailable { reason }, evidence.clone());

        let controller = match &read.account {
            None => unavailable(format!("admin account {} does not exist", spec.account)),
            Some(acct) if !protocol_programs.contains(&acct.owner) => unavailable(format!(
                "admin account {} is owned by {}, not by one of the protocol's programs",
                spec.account, acct.owner
            )),
            Some(acct) => match spec.idl.read_pubkey(&acct.data, &spec.field) {
                Err(e) => unavailable(format!("cannot decode `{}`: {e}", spec.field)),
                Ok(None) => Fact::single(Controller::Immutable, evidence.clone()),
                Ok(Some(bytes)) if bytes == [0u8; 32] => {
                    Fact::single(Controller::Immutable, evidence.clone())
                }
                Ok(Some(bytes)) => {
                    resolver
                        .resolve(&pda::encode(bytes), evidence.clone())
                        .await?
                        .fact
                }
            },
        };
        out.insert(
            format!("{}.{}", spec.account, spec.field),
            AdminFact {
                account: spec.account.clone(),
                field: spec.field.clone(),
                power: spec.power,
                controller,
            },
        );
    }
    Ok(out)
}
