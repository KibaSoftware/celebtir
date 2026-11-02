//! REG-2's on-chain check: each registry program ID is an existing,
//! executable account. One batched existence read, no data downloaded.

use celebtir_core::Address;

use crate::reads::SolanaRpc;
use crate::rpc::{RpcError, Transport};

/// `(program, problem)` for every ID that is missing or not executable.
pub async fn check_programs<T: Transport>(
    rpc: &SolanaRpc<T>,
    programs: &[Address],
) -> Result<Vec<(Address, String)>, RpcError> {
    let reads = rpc.get_multiple_accounts(programs, Some((0, 0))).await?;
    Ok(reads
        .into_iter()
        .filter_map(|r| match &r.account {
            None => Some((
                r.address.clone(),
                format!(
                    "program {} does not exist on mainnet (closed or mistyped)",
                    r.address
                ),
            )),
            Some(a) if !a.executable => Some((
                r.address.clone(),
                format!(
                    "{} exists but is not executable (owner {})",
                    r.address, a.owner
                ),
            )),
            Some(_) => None,
        })
        .collect())
}
