//! COL-9: Squads v4 multisig decoding: owner and discriminator checked;
//! threshold, time lock, config authority and members read; only members with
//! the Vote permission count as voters.

use celebtir_core::Address;
use collect_solana::fixtures::load;
use collect_solana::reads::{AccountRead, RawAccount};
use collect_solana::squads::{self, MULTISIG_DISCRIMINATOR, SQUADS_V4_PROGRAM, permission};

fn kamino() -> AccountRead {
    load("squads-multisig-kamino").read()
}

/// Builds Multisig account bytes in the v4 layout (synthetic variants).
fn encode(
    config_authority: [u8; 32],
    threshold: u16,
    time_lock: u32,
    rent_collector: Option<[u8; 32]>,
    members: &[([u8; 32], u8)],
) -> Vec<u8> {
    let mut d = MULTISIG_DISCRIMINATOR.to_vec();
    d.extend([9u8; 32]); // create_key
    d.extend(config_authority);
    d.extend(threshold.to_le_bytes());
    d.extend(time_lock.to_le_bytes());
    d.extend(7u64.to_le_bytes()); // transaction_index
    d.extend(3u64.to_le_bytes()); // stale_transaction_index
    match rent_collector {
        Some(k) => {
            d.push(1);
            d.extend(k);
        }
        None => d.push(0),
    }
    d.push(255); // bump
    d.extend((members.len() as u32).to_le_bytes());
    for (k, m) in members {
        d.extend(k);
        d.push(*m);
    }
    d.extend([0u8; 40]); // allocated slack
    d
}

fn synthetic(data: Vec<u8>) -> AccountRead {
    AccountRead {
        address: Address::new("SyntheticMu1tisig11111111111111111111111111"),
        slot: 1,
        account: Some(RawAccount {
            owner: Address::new(SQUADS_V4_PROGRAM),
            lamports: 1,
            executable: false,
            data,
        }),
    }
}

const ALL: u8 = permission::INITIATE | permission::VOTE | permission::EXECUTE;

#[test]
fn col_9_decodes_kaminos_multisig() {
    let m = squads::decode_multisig(&kamino()).unwrap();
    assert_eq!(m.threshold, 5);
    assert_eq!(m.time_lock_secs, 86_400);
    assert_eq!(m.config_authority, None);
    assert_eq!(m.members.len(), 10);
    assert_eq!(m.voter_count(), 10);
    assert_eq!(
        m.create_key.as_str(),
        "Cyv5n1Ct4wLzCJoM7BDgwxkQ6rAZyyPAGWUu1yqF1Se1"
    );
}

#[test]
fn col_9_five_members_one_without_vote_report_four_voters() {
    let members: Vec<([u8; 32], u8)> = (1..=5u8)
        .map(|i| {
            (
                [i; 32],
                if i == 3 {
                    permission::INITIATE | permission::EXECUTE
                } else {
                    ALL
                },
            )
        })
        .collect();
    let m = squads::decode_multisig(&synthetic(encode([0; 32], 3, 0, None, &members))).unwrap();
    assert_eq!(m.members.len(), 5);
    assert_eq!(m.voter_count(), 4);
    assert!(
        !m.voters()
            .any(|k| *k == Address::new(bs58::encode([3u8; 32]).into_string()))
    );
}

#[test]
fn col_9_reads_a_set_config_authority() {
    let m = squads::decode_multisig(&synthetic(encode(
        [4; 32],
        2,
        3600,
        None,
        &[([1; 32], ALL), ([2; 32], ALL)],
    )))
    .unwrap();
    assert_eq!(
        m.config_authority,
        Some(Address::new(bs58::encode([4u8; 32]).into_string()))
    );
    assert_eq!(m.time_lock_secs, 3600);
}

#[test]
fn col_9_rent_collector_shifts_the_members_correctly() {
    let m = squads::decode_multisig(&synthetic(encode(
        [0; 32],
        1,
        0,
        Some([8; 32]),
        &[([1; 32], ALL), ([2; 32], permission::VOTE)],
    )))
    .unwrap();
    assert_eq!(m.members.len(), 2);
    assert_eq!(m.voter_count(), 2);
}

#[test]
fn col_9_wrong_owner_is_rejected() {
    let mut read = kamino();
    read.account.as_mut().unwrap().owner = Address::new("11111111111111111111111111111111");
    let err = squads::decode_multisig(&read).unwrap_err();
    assert!(err.0.contains("owner"), "{err}");
}

#[test]
fn col_9_wrong_discriminator_is_rejected() {
    let mut read = kamino();
    read.account.as_mut().unwrap().data[0] ^= 0xff;
    let err = squads::decode_multisig(&read).unwrap_err();
    assert!(err.0.contains("discriminator"), "{err}");
}

#[test]
fn col_9_truncated_member_list_is_an_error() {
    let data = encode([0; 32], 1, 0, None, &[([1; 32], ALL), ([2; 32], ALL)]);
    let cut = data.len() - 40 - 20; // drop the slack and part of the last member
    assert!(squads::decode_multisig(&synthetic(data[..cut].to_vec())).is_err());
}

#[test]
fn col_9_missing_account_is_an_error() {
    let read = AccountRead {
        address: Address::new("x"),
        slot: 1,
        account: None,
    };
    assert!(squads::decode_multisig(&read).is_err());
}

#[test]
fn col_9_discriminator_is_sha256_of_account_multisig() {
    // First 8 bytes of sha256("account:Multisig"), checked against the
    // captured mainnet account rather than recomputed here.
    assert_eq!(
        &kamino().account.unwrap().data[..8],
        &MULTISIG_DISCRIMINATOR
    );
}
