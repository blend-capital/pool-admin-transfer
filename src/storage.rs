use soroban_sdk::{Address, Env};

//********** Storage Utils **********//

pub const ONE_DAY_LEDGERS: u32 = 17280; // assumes 5 seconds per ledger on average

const LEDGER_BUMP_SHARED: u32 = 31 * ONE_DAY_LEDGERS;
const LEDGER_THRESHOLD_SHARED: u32 = LEDGER_BUMP_SHARED - ONE_DAY_LEDGERS;

const LEDGER_BUMP_TRANSFER: u32 = 120 * ONE_DAY_LEDGERS;
const LEDGER_THRESHOLD_TRANSFER: u32 = LEDGER_BUMP_TRANSFER - 20 * ONE_DAY_LEDGERS;

/// Bump the instance lifetime by the defined amount
pub fn extend_instance(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(LEDGER_THRESHOLD_SHARED, LEDGER_BUMP_SHARED);
}

/********** Persistent **********/

/// Check if an admin transfer exists
///
/// ### Arguments
/// * `pool` - The address of the pool the admin transfer is for
pub fn has_admin_transfer(e: &Env, pool: &Address) -> bool {
    e.storage().persistent().has(&pool)
}

/// Set the admin transfer details
///
/// ### Arguments
/// * `pool` - The address of the pool the admin transfer is for
/// * `new_admin` - The address of the new admin
pub fn set_admin_transfer(e: &Env, pool: &Address, new_admin: &Address) {
    e.storage()
        .persistent()
        .set::<Address, Address>(&pool, &new_admin);
    e.storage()
        .persistent()
        .extend_ttl(&pool, LEDGER_THRESHOLD_TRANSFER, LEDGER_BUMP_TRANSFER);
}

/// Get the new admin for an admin transfer
///
/// ### Arguments
/// * `pool` - The address of the pool the admin transfer is for
pub fn get_admin_transfer(e: &Env, pool: &Address) -> Option<Address> {
    e.storage().persistent().get(&pool)
}
