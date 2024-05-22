use blend_contract_sdk::pool::Client;
use soroban_sdk::{assert_with_error, contract, contractimpl, panic_with_error, Address, Env};

use crate::{errors::ContractError, storage};

#[contract]
pub struct AdminTransfer;

#[contractimpl]
impl AdminTransfer {
    /// Set the details for an admin transfer
    ///
    /// ### Arguments
    /// * `pool` - The address of the pool the admin transfer is for
    /// * `new_admin` - The deadline ledger sequence number of the distribution
    ///
    /// ### Panics
    /// * `AdminTransferExists` - If the contract has already been initialized
    pub fn set_admin_transfer(e: Env, pool: Address, new_admin: Address) {
        assert_with_error!(
            &e,
            !storage::has_admin_transfer(&e, &pool),
            ContractError::AdminTransferExists
        );
        storage::extend_instance(&e);

        storage::set_admin_transfer(&e, &pool, &new_admin);
    }

    /// Get the new admin for an admin transfer
    ///
    /// ### Arguments
    /// * `pool` - The address of the pool the admin transfer is for
    pub fn get_admin_transfer(e: Env, pool: Address) -> Option<Address> {
        storage::get_admin_transfer(&e, &pool)
    }

    /// Transfer the admin of a pool from the current admin to the new admin
    ///
    /// ### Arguments
    /// * `pool` - The address of the pool the admin transfer is for
    ///
    /// ### Panics
    /// * `NoAdminTransferExists` - If no admin transfer exists for the pool
    pub fn transfer_admin(e: Env, pool: Address) {
        let new_admin = match storage::get_admin_transfer(&e, &pool) {
            Some(admin) => admin,
            None => panic_with_error!(&e, ContractError::NoAdminTransferExists),
        };
        new_admin.require_auth();

        let pool_client = Client::new(&e, &pool);
        pool_client.set_admin(&new_admin);
    }
}
