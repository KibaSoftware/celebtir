//! COL-8 building blocks: Squads vault derivation and the curve check,
//! checked against Kamino's real addresses.

use celebtir_core::Address;
use collect_solana::fixtures::load;
use collect_solana::pda;
use collect_solana::squads;

#[test]
fn col_8_squads_vault_derivation_matches_kaminos_upgrade_authority() {
    let vault = pda::squads_vault(
        &Address::new("6hhBGCtmg7tPWUSgp3LG6X2rsmYWAc4tNsA6G4CnfQbM"),
        0,
    )
    .unwrap();
    assert_eq!(
        vault.as_str(),
        "GzFgdRJXmawPhGeBsyRCDLx4jAKPsvbUqoqitzppkzkW"
    );
}

#[test]
fn col_8_squads_v3_authority_derivation_matches_the_stake_pool_upgrade_authority() {
    let authority = pda::squads_v3_authority(
        &Address::new("3yqoHFE4nBGchuVH5rJuZMFvsmnaDTuLLdvGPDUEJcbW"),
        1,
    )
    .unwrap();
    assert_eq!(
        authority.as_str(),
        "dPpFfahSWhm5M2RB3nVaHsXfnh9ooz5bWtR3WeUguDH"
    );
}

#[test]
fn col_8_vault_is_off_curve_and_member_wallets_are_on_curve() {
    let vault = pda::decode(&Address::new(
        "GzFgdRJXmawPhGeBsyRCDLx4jAKPsvbUqoqitzppkzkW",
    ))
    .unwrap();
    assert!(!pda::is_on_curve(&vault));
    let m = squads::decode_multisig(&load("squads-multisig-kamino").read()).unwrap();
    let on_curve = m
        .members
        .iter()
        .filter(|mem| pda::is_on_curve(&pda::decode(&mem.key).unwrap()))
        .count();
    assert!(on_curve >= 1, "Kamino's members include wallets");
}

#[test]
fn col_8_invalid_base58_is_not_an_address() {
    assert!(pda::decode(&Address::new("not-base58-0OIl")).is_none());
    assert!(pda::decode(&Address::new("1111")).is_none(), "too short");
}
