//! COL-9 for Squads v3 (squads-mpl): the `Ms` account behind the SPL Stake
//! Pool program's upgrade authority, captured from mainnet.

use celebtir_core::Address;
use collect_solana::fixtures::load;
use collect_solana::reads::{AccountRead, RawAccount};
use collect_solana::squads::{self, SQUADS_V3_PROGRAM, V3_MULTISIG_DISCRIMINATOR};

fn stake_pool() -> AccountRead {
    load("squads-v3-multisig-spl-stake-pool").read()
}

/// `Ms` bytes in the v3 layout (synthetic variants).
fn encode(threshold: u16, authority_index: u16, keys: &[[u8; 32]]) -> Vec<u8> {
    let mut d = V3_MULTISIG_DISCRIMINATOR.to_vec();
    d.extend(threshold.to_le_bytes());
    d.extend(authority_index.to_le_bytes());
    d.extend(12u32.to_le_bytes()); // transaction_index
    d.extend(3u32.to_le_bytes()); // ms_change_index
    d.push(254); // bump
    d.extend([9u8; 32]); // create_key
    d.push(0); // allow_external_execute
    d.extend((keys.len() as u32).to_le_bytes());
    for k in keys {
        d.extend(k);
    }
    d.extend([0u8; 64]); // allocated slack
    d
}

fn synthetic(data: Vec<u8>) -> AccountRead {
    AccountRead {
        address: Address::new("SyntheticV3Mu1tisig1111111111111111111111111"),
        slot: 1,
        account: Some(RawAccount {
            owner: Address::new(SQUADS_V3_PROGRAM),
            lamports: 1,
            executable: false,
            data,
        }),
    }
}

#[test]
fn col_9_v3_decodes_the_spl_stake_pool_multisig() {
    let m = squads::decode_v3_multisig(&stake_pool()).unwrap();
    assert_eq!(m.threshold, 6);
    assert_eq!(m.members.len(), 10);
    assert_eq!(m.voter_count(), 10, "every v3 member votes");
    assert_eq!(m.time_lock_secs, 0, "Squads v3 has no time lock");
    assert_eq!(
        m.config_authority, None,
        "v3 config changes need the members' vote"
    );
}

#[test]
fn col_9_v3_discriminator_is_sha256_of_account_ms() {
    assert_eq!(
        &stake_pool().account.unwrap().data[..8],
        &V3_MULTISIG_DISCRIMINATOR
    );
}

#[test]
fn col_9_v3_reads_threshold_and_keys_from_synthetic_bytes() {
    let m =
        squads::decode_v3_multisig(&synthetic(encode(2, 1, &[[1; 32], [2; 32], [3; 32]]))).unwrap();
    assert_eq!(m.threshold, 2);
    assert_eq!(m.voter_count(), 3);
    assert_eq!(
        m.create_key,
        Address::new(bs58::encode([9u8; 32]).into_string())
    );
}

#[test]
fn col_9_v3_wrong_owner_is_rejected() {
    let mut read = stake_pool();
    read.account.as_mut().unwrap().owner = Address::new("11111111111111111111111111111111");
    let err = squads::decode_v3_multisig(&read).unwrap_err();
    assert!(err.0.contains("owner"), "{err}");
}

#[test]
fn col_9_v3_wrong_discriminator_is_rejected() {
    let mut read = stake_pool();
    read.account.as_mut().unwrap().data[0] ^= 0xff;
    let err = squads::decode_v3_multisig(&read).unwrap_err();
    assert!(err.0.contains("discriminator"), "{err}");
}

#[test]
fn col_9_v3_truncated_key_list_is_an_error() {
    let data = encode(1, 1, &[[1; 32], [2; 32]]);
    let cut = data.len() - 64 - 10;
    assert!(squads::decode_v3_multisig(&synthetic(data[..cut].to_vec())).is_err());
}
