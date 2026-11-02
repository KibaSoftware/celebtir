//! A program's upgrade control as a fact: ProgramData read (COL-6), loader
//! label (COL-7), then the authority resolved to its controller (COL-8).
//! Shared by core programs, dependency callees (ADR 006) and transfer hooks.

use celebtir_core::{Address, Chain, Controller, Evidence, Fact};

use crate::program::{ProgramControlRead, ProgramKind, read_program_control};
use crate::reads::SolanaRpc;
use crate::resolve::Resolver;
use crate::rpc::{RpcError, Transport};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramControl {
    pub read: ProgramControlRead,
    pub upgrade_control: Fact<Controller>,
    /// Registry review notes from resolution.
    pub review_needed: Vec<String>,
}

pub async fn upgrade_controls<T: Transport>(
    rpc: &SolanaRpc<T>,
    resolver: &Resolver<'_, T>,
    programs: &[Address],
) -> Result<Vec<ProgramControl>, RpcError> {
    let mut out = Vec::new();
    for read in read_program_control(rpc, programs).await? {
        let program_ev = Evidence::new(
            Chain::Solana,
            read.program.clone(),
            read.program_slot,
            "owner",
        );
        let single = |c: Controller, ev: Evidence| (Fact::single(c, ev), Vec::new());
        let (upgrade_control, review_needed) = match (&read.kind, &read.header, &read.error) {
            (_, _, Some(e)) => single(Controller::Unavailable { reason: e.clone() }, program_ev),
            (ProgramKind::Closed, ..) => single(
                Controller::Unavailable {
                    reason: format!("program account {} is closed", read.program),
                },
                program_ev,
            ),
            (ProgramKind::Immutable { .. }, ..) => single(Controller::Immutable, program_ev),
            (ProgramKind::UnsupportedLoader { owner }, ..) => single(
                Controller::Unrecognized {
                    key: read.program.clone(),
                    owner_program: Some(owner.clone()),
                },
                program_ev,
            ),
            (ProgramKind::Upgradeable { programdata }, Some(header), None) => {
                let ev = Evidence::new(
                    Chain::Solana,
                    programdata.clone(),
                    read.programdata_slot.unwrap_or(read.program_slot),
                    "upgrade_authority",
                );
                match &header.authority {
                    None => single(Controller::Immutable, ev),
                    Some(key) => {
                        let r = resolver.resolve(key, ev).await?;
                        (r.fact, r.review_needed)
                    }
                }
            }
            (ProgramKind::Upgradeable { .. }, None, None) => single(
                Controller::Unavailable {
                    reason: "ProgramData was not read".into(),
                },
                program_ev,
            ),
        };
        out.push(ProgramControl {
            read,
            upgrade_control,
            review_needed,
        });
    }
    Ok(out)
}
