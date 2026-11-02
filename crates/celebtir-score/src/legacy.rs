//! Byte-identical replay of v0.1 and v0.2 (SNAP-4). Facts collected after
//! v0.3 can carry representations those versions never knew; before a legacy
//! version scores them, they are lowered to what the collector wrote then.
//! Today that is one rule: an unresolved program-derived address
//! (`Controller::Pda`) was written as `Controller::Unrecognized`.

use celebtir_core::snapshot::{
    Dependencies, DependencyFact, FactsFile, ProtocolFacts, TokenExtensions, TokenFacts,
    TransferHook,
};
use celebtir_core::{Controller, Fact};

/// The controller as v0.1/v0.2 facts expressed it.
pub fn controller(c: &Controller) -> Controller {
    let boxed = |c: &Controller| Box::new(controller(c));
    match c {
        Controller::Pda { key, owner_program } => Controller::Unrecognized {
            key: key.clone(),
            owner_program: owner_program.clone(),
        },
        Controller::ProgramControlled {
            key,
            program,
            upgrade,
        } => Controller::ProgramControlled {
            key: key.clone(),
            program: program.clone(),
            upgrade: boxed(upgrade),
        },
        Controller::Multisig {
            kind,
            address,
            threshold,
            voters,
            time_lock_secs,
            config_authority,
            signers,
            nested_signers,
        } => Controller::Multisig {
            kind: *kind,
            address: address.clone(),
            threshold: *threshold,
            voters: *voters,
            time_lock_secs: *time_lock_secs,
            config_authority: config_authority.as_deref().map(boxed),
            signers: signers.clone(),
            nested_signers: nested_signers
                .iter()
                .map(|(k, v)| (k.clone(), boxed(v)))
                .collect(),
        },
        Controller::Timelock {
            address,
            min_delay_secs,
            proposer,
            admin,
        } => Controller::Timelock {
            address: address.clone(),
            min_delay_secs: *min_delay_secs,
            proposer: boxed(proposer),
            admin: admin.as_deref().map(boxed),
        },
        other => other.clone(),
    }
}

fn fact(f: &Fact<Controller>) -> Fact<Controller> {
    Fact::new(controller(f.value()), f.evidence().to_vec()).expect("facts keep their evidence")
}

fn option(c: &Option<Controller>) -> Option<Controller> {
    c.as_ref().map(controller)
}

pub fn lower_protocol(p: &ProtocolFacts) -> ProtocolFacts {
    let mut p = p.clone();
    for f in p.programs.values_mut() {
        f.upgrade_control = fact(&f.upgrade_control);
    }
    for a in p.admin.values_mut() {
        a.controller = fact(&a.controller);
    }
    if let Dependencies::Sampled {
        callees,
        stale,
        declared,
    } = p.dependencies.value()
    {
        let lowered = Dependencies::Sampled {
            callees: callees
                .iter()
                .map(|(k, d)| {
                    (
                        k.clone(),
                        DependencyFact {
                            upgrade_control: fact(&d.upgrade_control),
                        },
                    )
                })
                .collect(),
            stale: *stale,
            declared: *declared,
        };
        p.dependencies = Fact::new(lowered, p.dependencies.evidence().to_vec())
            .expect("facts keep their evidence");
    }
    p
}

pub fn lower_token(t: &TokenFacts) -> TokenFacts {
    let mut t = t.clone();
    t.mint_authority = fact(&t.mint_authority);
    t.freeze_authority = fact(&t.freeze_authority);
    let e = t.extensions.value();
    let lowered = TokenExtensions {
        permanent_delegate: option(&e.permanent_delegate),
        transfer_hook: e.transfer_hook.as_ref().map(|h| TransferHook {
            program: h.program.clone(),
            authority: controller(&h.authority),
            program_controller: option(&h.program_controller),
        }),
        transfer_fee_authority: option(&e.transfer_fee_authority),
        pause_authority: option(&e.pause_authority),
        default_frozen: e.default_frozen,
        mint_close_authority: option(&e.mint_close_authority),
        metadata_update_authority: option(&e.metadata_update_authority),
        other: e.other.clone(),
        unknown: e.unknown.clone(),
    };
    t.extensions =
        Fact::new(lowered, t.extensions.evidence().to_vec()).expect("facts keep their evidence");
    t
}

pub fn lower(facts: &FactsFile) -> FactsFile {
    let mut f = facts.clone();
    for p in f.protocols.values_mut() {
        *p = lower_protocol(p);
    }
    for t in f.tokens.values_mut() {
        *t = lower_token(t);
    }
    f
}
