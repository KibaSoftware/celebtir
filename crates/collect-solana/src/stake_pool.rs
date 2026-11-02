//! SPL stake pool authorities as admin powers (F3, methodology v0.3 D2).
//!
//! The account layout is the Solana Program Library's `StakePool`
//! (`stake-pool/program/src/state.rs`, borsh), which Sanctum's forks keep.
//! The manager sets fees and roles, the staker the validator set and stake
//! distribution, and the deposit/withdraw authorities gate those paths. None
//! moves or freezes user funds, so all hold the `parameters` power. A stake
//! deposit authority equal to the program's default `[pool, "deposit"]`
//! address gates nothing and is not reported.

use std::collections::BTreeMap;

use celebtir_core::snapshot::{AdminFact, AdminPower};
use celebtir_core::{Address, Chain, Controller, Evidence, Fact};

use crate::pda;
use crate::reads::SolanaRpc;
use crate::resolve::Resolver;
use crate::rpc::{RpcError, Transport};

/// One registry `stake_pools` entry.
pub struct StakePoolSpec {
    pub account: Address,
    pub pool_mint: Address,
}

/// The authorities a pool account names. `None` for an optional authority
/// that is not set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolAuthorities {
    pub manager: [u8; 32],
    pub staker: [u8; 32],
    pub stake_deposit_authority: [u8; 32],
    pub pool_mint: [u8; 32],
    pub sol_deposit_authority: Option<[u8; 32]>,
    pub sol_withdraw_authority: Option<[u8; 32]>,
}

struct Cursor<'a> {
    data: &'a [u8],
    at: usize,
}

impl Cursor<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8], String> {
        let end = self.at + n;
        let out = self
            .data
            .get(self.at..end)
            .ok_or_else(|| format!("account data ends at byte {}", self.data.len()))?;
        self.at = end;
        Ok(out)
    }
    fn key(&mut self) -> Result<[u8; 32], String> {
        Ok(self.take(32)?.try_into().expect("32 bytes"))
    }
    fn byte(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }
    fn skip(&mut self, n: usize) -> Result<(), String> {
        self.take(n).map(|_| ())
    }
    /// `FutureEpoch<Fee>`: tag 0 none, 1 or 2 followed by a 16-byte fee.
    fn future_fee(&mut self) -> Result<(), String> {
        match self.byte()? {
            0 => Ok(()),
            1 | 2 => self.skip(16),
            t => Err(format!("unknown FutureEpoch tag {t}")),
        }
    }
    fn option_key(&mut self) -> Result<Option<[u8; 32]>, String> {
        match self.byte()? {
            0 => Ok(None),
            1 => self.key().map(Some),
            t => Err(format!("unknown Option tag {t}")),
        }
    }
}

/// Decodes the authorities from a full `StakePool` account.
pub fn decode(data: &[u8]) -> Result<PoolAuthorities, String> {
    let mut c = Cursor { data, at: 0 };
    let account_type = c.byte()?;
    if account_type != 1 {
        return Err(format!(
            "account type {account_type} is not an initialized stake pool"
        ));
    }
    let manager = c.key()?;
    let staker = c.key()?;
    let stake_deposit_authority = c.key()?;
    c.skip(1)?; // stake_withdraw_bump_seed
    c.skip(32 * 2)?; // validator_list, reserve_stake
    let pool_mint = c.key()?;
    c.skip(32 * 2)?; // manager_fee_account, token_program_id
    c.skip(8 * 3)?; // total_lamports, pool_token_supply, last_update_epoch
    c.skip(48)?; // lockup
    c.skip(16)?; // epoch_fee
    c.future_fee()?; // next_epoch_fee
    c.option_key()?; // preferred_deposit_validator_vote_address
    c.option_key()?; // preferred_withdraw_validator_vote_address
    c.skip(16 * 2)?; // stake_deposit_fee, stake_withdrawal_fee
    c.future_fee()?; // next_stake_withdrawal_fee
    c.skip(1)?; // stake_referral_fee
    let sol_deposit_authority = c.option_key()?;
    c.skip(16)?; // sol_deposit_fee
    c.skip(1)?; // sol_referral_fee
    let sol_withdraw_authority = c.option_key()?;
    Ok(PoolAuthorities {
        manager,
        staker,
        stake_deposit_authority,
        pool_mint,
        sol_deposit_authority,
        sol_withdraw_authority,
    })
}

/// The program's default stake deposit authority for `pool`.
pub fn default_deposit_authority(program: &Address, pool: &Address) -> Option<Address> {
    let pool = pda::decode(pool)?;
    let program = pda::decode(program)?;
    pda::find_program_address(&[&pool, b"deposit"], &program).map(|(a, _)| pda::encode(a))
}

/// Admin facts keyed `<pool>.<field>`, from one batched read.
pub async fn read_stake_pools<T: Transport>(
    rpc: &SolanaRpc<T>,
    resolver: &Resolver<'_, T>,
    specs: &[StakePoolSpec],
    protocol_programs: &[Address],
) -> Result<BTreeMap<String, AdminFact>, RpcError> {
    let accounts: Vec<Address> = specs.iter().map(|s| s.account.clone()).collect();
    let reads = rpc.get_multiple_accounts(&accounts, None).await?;
    let mut out = BTreeMap::new();
    for (spec, read) in specs.iter().zip(reads) {
        let ev = |field: &str| Evidence::new(Chain::Solana, spec.account.clone(), read.slot, field);
        let decoded = match &read.account {
            None => Err(format!("stake pool {} does not exist", spec.account)),
            Some(a) if !protocol_programs.contains(&a.owner) => Err(format!(
                "stake pool {} is owned by {}, not by one of the protocol's programs",
                spec.account, a.owner
            )),
            Some(a) => decode(&a.data).and_then(|d| {
                if pda::encode(d.pool_mint) == spec.pool_mint {
                    Ok((a.owner.clone(), d))
                } else {
                    Err(format!(
                        "stake pool {} names mint {}, not {}",
                        spec.account,
                        pda::encode(d.pool_mint),
                        spec.pool_mint
                    ))
                }
            }),
        };
        let (owner, d) = match decoded {
            Ok(x) => x,
            Err(reason) => {
                for field in ["manager", "staker"] {
                    out.insert(
                        format!("{}.{field}", spec.account),
                        AdminFact {
                            account: spec.account.clone(),
                            field: field.into(),
                            power: AdminPower::Parameters,
                            controller: Fact::single(
                                Controller::Unavailable {
                                    reason: reason.clone(),
                                },
                                ev(field),
                            ),
                        },
                    );
                }
                continue;
            }
        };
        let default_deposit = default_deposit_authority(&owner, &spec.account);
        let mut fields: Vec<(&str, Option<[u8; 32]>)> =
            vec![("manager", Some(d.manager)), ("staker", Some(d.staker))];
        if default_deposit.as_ref() != Some(&pda::encode(d.stake_deposit_authority)) {
            fields.push(("stake_deposit_authority", Some(d.stake_deposit_authority)));
        }
        if d.sol_deposit_authority.is_some() {
            fields.push(("sol_deposit_authority", d.sol_deposit_authority));
        }
        if d.sol_withdraw_authority.is_some() {
            fields.push(("sol_withdraw_authority", d.sol_withdraw_authority));
        }
        for (field, key) in fields {
            let origin = ev(field);
            let controller = match key {
                None => continue,
                Some(bytes) if bytes == [0u8; 32] => Fact::single(Controller::Immutable, origin),
                Some(bytes) => resolver.resolve(&pda::encode(bytes), origin).await?.fact,
            };
            out.insert(
                format!("{}.{field}", spec.account),
                AdminFact {
                    account: spec.account.clone(),
                    field: field.into(),
                    power: AdminPower::Parameters,
                    controller,
                },
            );
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pool(sol_deposit: Option<[u8; 32]>) -> Vec<u8> {
        let mut d = vec![1u8];
        d.extend([2u8; 32]); // manager
        d.extend([3u8; 32]); // staker
        d.extend([4u8; 32]); // stake deposit authority
        d.push(255);
        d.extend([0u8; 64]);
        d.extend([5u8; 32]); // pool mint
        d.extend([0u8; 64 + 24 + 48 + 16]);
        d.push(1);
        d.extend([0u8; 16]); // next_epoch_fee: One(fee)
        d.push(0);
        d.push(0);
        d.extend([0u8; 32]);
        d.push(0);
        d.push(0);
        match sol_deposit {
            Some(k) => {
                d.push(1);
                d.extend(k);
            }
            None => d.push(0),
        }
        d.extend([0u8; 17]);
        d.push(0);
        d.extend([0u8; 16 + 1 + 16]);
        d
    }

    #[test]
    fn aud_3_stake_pool_layout_decodes_authorities() {
        let d = decode(&pool(Some([9u8; 32]))).unwrap();
        assert_eq!(d.manager, [2u8; 32]);
        assert_eq!(d.staker, [3u8; 32]);
        assert_eq!(d.stake_deposit_authority, [4u8; 32]);
        assert_eq!(d.pool_mint, [5u8; 32]);
        assert_eq!(d.sol_deposit_authority, Some([9u8; 32]));
        assert_eq!(d.sol_withdraw_authority, None);
    }

    #[test]
    fn aud_3_stake_pool_rejects_other_accounts() {
        let mut data = pool(None);
        data[0] = 2;
        assert!(decode(&data).is_err());
        assert!(decode(&data[..40]).is_err());
    }
}
