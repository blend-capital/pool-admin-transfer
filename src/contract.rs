use blend_contract_sdk::pool::Client;
use soroban_sdk::{assert_with_error, contract, contractimpl, panic_with_error, Address, Env};

use crate::{
    errors::ContractError,
    storage::{self, TransferData},
};

#[contract]
pub struct AdminTransfer;

#[contractimpl]
impl AdminTransfer {
    /// Set the details for an admin transfer. Also sets the admin of the pool to this contract.
    /// Must be called by the current admin of the pool.
    ///
    /// ### Arguments
    /// * `pool` - The address of the pool the admin transfer is for
    /// * `cur_admin` - The current admin of the pool
    /// * `new_admin` - The new admin of the pool
    ///
    /// ### Panics
    /// * `AdminTransferExists` - If the contract has already been initialized
    pub fn set_admin_transfer(e: Env, pool: Address, cur_admin: Address, new_admin: Address) {
        assert_with_error!(
            &e,
            !storage::has_admin_transfer(&e, &pool),
            ContractError::AdminTransferExists
        );
        cur_admin.require_auth();
        storage::extend_instance(&e);

        let pool_client = Client::new(&e, &pool);
        pool_client.set_admin(&e.current_contract_address());

        let admin_transfer = TransferData {
            cur_admin,
            new_admin,
        };
        storage::set_admin_transfer(&e, &pool, &admin_transfer);
    }

    /// Get the new admin for an admin transfer
    ///
    /// ### Arguments
    /// * `pool` - The address of the pool the admin transfer is for
    pub fn get_admin_transfer(e: Env, pool: Address) -> Option<TransferData> {
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
        let admin_transfer = match storage::get_admin_transfer(&e, &pool) {
            Some(admin) => admin,
            None => panic_with_error!(&e, ContractError::NoAdminTransferExists),
        };
        admin_transfer.new_admin.require_auth();
        storage::extend_instance(&e);

        let pool_client = Client::new(&e, &pool);
        pool_client.set_admin(&admin_transfer.new_admin);

        storage::del_admin_transfer(&e, &pool);
    }

    /// Cancel an admin transfer. Must be called by the creator of the admin transfer.
    ///
    /// ### Arguments
    /// * `pool` - The address of the pool the admin transfer is for
    pub fn cancel_admin_transfer(e: Env, pool: Address) {
        let admin_transfer = match storage::get_admin_transfer(&e, &pool) {
            Some(admin) => admin,
            None => panic_with_error!(&e, ContractError::NoAdminTransferExists),
        };
        admin_transfer.cur_admin.require_auth();

        let pool_client = Client::new(&e, &pool);
        pool_client.set_admin(&admin_transfer.cur_admin);

        storage::extend_instance(&e);
        storage::del_admin_transfer(&e, &pool);
    }
}
