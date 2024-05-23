#![cfg(test)]

use crate::{errors::ContractError, testutils::EnvTestUtils, AdminTransferClient};
use blend_contract_sdk::pool::Client as PoolClient;
use blend_contract_sdk::testutils::BlendFixture;
use soroban_sdk::{
    testutils::{
        Address as _, AuthorizedFunction, AuthorizedInvocation, BytesN as _, MockAuth,
        MockAuthInvoke,
    },
    vec, Address, BytesN, Env, Error, IntoVal, String, Symbol,
};

mod admin_transfer_wasm {
    soroban_sdk::contractimport!(
        file = "./target/wasm32-unknown-unknown/optimized/pool_admin_transfer.wasm"
    );
}

#[test]
fn test_admin_transfer() {
    let env = Env::default();
    env.set_default_info();

    let admin_transfer_id = env.register_contract_wasm(None, admin_transfer_wasm::WASM);
    let admin_transfer_client = AdminTransferClient::new(&env, &admin_transfer_id);

    let admin = Address::generate(&env);
    let blnd = env.register_stellar_asset_contract(admin.clone());
    let usdc = env.register_stellar_asset_contract(admin.clone());

    let new_admin = Address::generate(&env);
    let sauron = Address::generate(&env);

    let blend_fixture = BlendFixture::deploy(&env, &admin, &blnd, &usdc);
    let pool = blend_fixture.pool_factory.mock_all_auths().deploy(
        &admin,
        &String::from_str(&env, "test"),
        &BytesN::<32>::random(&env),
        &Address::generate(&env),
        &0,
        &2,
    );
    let pool_2 = blend_fixture.pool_factory.mock_all_auths().deploy(
        &admin,
        &String::from_str(&env, "test_2"),
        &BytesN::<32>::random(&env),
        &Address::generate(&env),
        &0,
        &2,
    );

    // validate that only the pool admin can create an admin transfer
    let result = admin_transfer_client
        .mock_auths(&[MockAuth {
            address: &sauron,
            invoke: &MockAuthInvoke {
                contract: &admin_transfer_id,
                fn_name: &"set_admin_transfer",
                args: vec![
                    &env,
                    pool.clone().into_val(&env),
                    sauron.clone().into_val(&env),
                    new_admin.clone().into_val(&env),
                ],
                sub_invokes: &[MockAuthInvoke {
                    contract: &pool,
                    fn_name: &"set_admin",
                    args: vec![&env, admin_transfer_id.clone().into_val(&env)],
                    sub_invokes: &[],
                }],
            },
        }])
        .try_set_admin_transfer(&pool, &sauron, &new_admin);
    assert!(result.is_err());

    // create admin transfer
    admin_transfer_client
        .mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &admin_transfer_id,
                fn_name: &"set_admin_transfer",
                args: vec![
                    &env,
                    pool.clone().into_val(&env),
                    admin.clone().into_val(&env),
                    new_admin.clone().into_val(&env),
                ],
                sub_invokes: &[MockAuthInvoke {
                    contract: &pool,
                    fn_name: &"set_admin",
                    args: vec![&env, admin_transfer_id.clone().into_val(&env)],
                    sub_invokes: &[],
                }],
            },
        }])
        .set_admin_transfer(&pool, &admin, &new_admin);

    // -> validate auths
    assert_eq!(
        env.auths()[0],
        (
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    admin_transfer_id.clone(),
                    Symbol::new(&env, "set_admin_transfer"),
                    vec![
                        &env,
                        pool.clone().into_val(&env),
                        admin.clone().into_val(&env),
                        new_admin.clone().into_val(&env),
                    ]
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        pool.clone(),
                        Symbol::new(&env, "set_admin"),
                        vec![&env, admin_transfer_id.clone().into_val(&env),]
                    )),
                    sub_invocations: std::vec![]
                }]
            }
        )
    );

    // -> validate chain state
    let result = admin_transfer_client.get_admin_transfer(&pool).unwrap();
    assert_eq!(result.new_admin, new_admin);
    assert_eq!(result.cur_admin, admin);

    // -> validate admin is no longer the admin
    let pool_client = PoolClient::new(&env, &pool);
    let result = pool_client
        .mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &pool,
                fn_name: &"set_status",
                args: vec![&env, 4u32.into_val(&env)],
                sub_invokes: &[],
            },
        }])
        .try_set_status(&4);
    assert!(result.is_err());

    // validate another admin transfer cannot be created
    let result = admin_transfer_client
        .mock_all_auths()
        .try_set_admin_transfer(&pool, &admin, &sauron);
    assert_eq!(
        result.err(),
        Some(Ok(Error::from_contract_error(
            ContractError::AdminTransferExists as u32
        )))
    );

    // perform admin transfer
    admin_transfer_client
        .mock_auths(&[MockAuth {
            address: &new_admin,
            invoke: &MockAuthInvoke {
                contract: &admin_transfer_id,
                fn_name: &"transfer_admin",
                args: vec![&env, pool.clone().into_val(&env)],
                sub_invokes: &[MockAuthInvoke {
                    contract: &pool,
                    fn_name: &"set_admin",
                    args: vec![&env, new_admin.clone().into_val(&env)],
                    sub_invokes: &[],
                }],
            },
        }])
        .transfer_admin(&pool);

    // -> validate auths
    assert_eq!(
        env.auths()[0],
        (
            new_admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    admin_transfer_id.clone(),
                    Symbol::new(&env, "transfer_admin"),
                    vec![&env, pool.clone().into_val(&env),]
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        pool.clone(),
                        Symbol::new(&env, "set_admin"),
                        vec![&env, new_admin.clone().into_val(&env),]
                    )),
                    sub_invocations: std::vec![]
                }]
            }
        )
    );

    // -> validate chain state by checking that set status can be called by new admin
    //    and the transfer was deleted
    pool_client
        .mock_auths(&[MockAuth {
            address: &new_admin,
            invoke: &MockAuthInvoke {
                contract: &pool,
                fn_name: &"set_status",
                args: vec![&env, 4u32.into_val(&env)],
                sub_invokes: &[],
            },
        }])
        .set_status(&4);

    let result = admin_transfer_client.get_admin_transfer(&pool);
    assert!(result.is_none());

    // validate that transfer admin cannot be re-run since contract is no longer admin
    let result = admin_transfer_client
        .mock_auths(&[MockAuth {
            address: &new_admin,
            invoke: &MockAuthInvoke {
                contract: &admin_transfer_id,
                fn_name: &"transfer_admin",
                args: vec![&env, pool.clone().into_val(&env)],
                sub_invokes: &[MockAuthInvoke {
                    contract: &pool,
                    fn_name: &"set_admin",
                    args: vec![&env, new_admin.clone().into_val(&env)],
                    sub_invokes: &[],
                }],
            },
        }])
        .try_transfer_admin(&pool);
    assert!(result.is_err());

    // validate that transfer admin for a random pool fails
    let result = admin_transfer_client
        .mock_all_auths()
        .try_transfer_admin(&pool_2);
    assert_eq!(
        result.err(),
        Some(Ok(Error::from_contract_error(
            ContractError::NoAdminTransferExists as u32
        )))
    );
}

#[test]
fn test_admin_transfer_cancel() {
    let env = Env::default();
    env.set_default_info();

    let admin_transfer_id = env.register_contract_wasm(None, admin_transfer_wasm::WASM);
    let admin_transfer_client = AdminTransferClient::new(&env, &admin_transfer_id);

    let admin = Address::generate(&env);
    let blnd = env.register_stellar_asset_contract(admin.clone());
    let usdc = env.register_stellar_asset_contract(admin.clone());

    let new_admin = Address::generate(&env);

    let blend_fixture = BlendFixture::deploy(&env, &admin, &blnd, &usdc);
    let pool = blend_fixture.pool_factory.mock_all_auths().deploy(
        &admin,
        &String::from_str(&env, "test"),
        &BytesN::<32>::random(&env),
        &Address::generate(&env),
        &0,
        &2,
    );

    // create admin transfer
    admin_transfer_client
        .mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &admin_transfer_id,
                fn_name: &"set_admin_transfer",
                args: vec![
                    &env,
                    pool.clone().into_val(&env),
                    admin.clone().into_val(&env),
                    new_admin.clone().into_val(&env),
                ],
                sub_invokes: &[MockAuthInvoke {
                    contract: &pool,
                    fn_name: &"set_admin",
                    args: vec![&env, admin_transfer_id.clone().into_val(&env)],
                    sub_invokes: &[],
                }],
            },
        }])
        .set_admin_transfer(&pool, &admin, &new_admin);

    // -> validate chain state
    let result = admin_transfer_client.get_admin_transfer(&pool).unwrap();
    assert_eq!(result.new_admin, new_admin);
    assert_eq!(result.cur_admin, admin);

    // -> validate admin is no longer the admin
    let pool_client = PoolClient::new(&env, &pool);
    let result = pool_client
        .mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &pool,
                fn_name: &"set_status",
                args: vec![&env, 4u32.into_val(&env)],
                sub_invokes: &[],
            },
        }])
        .try_set_status(&4);
    assert!(result.is_err());

    // cancel admin transfer
    admin_transfer_client
        .mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &admin_transfer_id,
                fn_name: &"cancel_admin_transfer",
                args: vec![&env, pool.clone().into_val(&env)],
                sub_invokes: &[MockAuthInvoke {
                    contract: &pool,
                    fn_name: &"set_admin",
                    args: vec![&env, admin.clone().into_val(&env)],
                    sub_invokes: &[],
                }],
            },
        }])
        .cancel_admin_transfer(&pool);

    // -> validate auths
    assert_eq!(
        env.auths()[0],
        (
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    admin_transfer_id.clone(),
                    Symbol::new(&env, "cancel_admin_transfer"),
                    vec![&env, pool.clone().into_val(&env),]
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        pool.clone(),
                        Symbol::new(&env, "set_admin"),
                        vec![&env, admin.clone().into_val(&env),]
                    )),
                    sub_invocations: std::vec![]
                }]
            }
        )
    );

    // -> validate chain state by checking that set_status can be called by admin
    //    and the transfer was deleted
    pool_client
        .mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &pool,
                fn_name: &"set_status",
                args: vec![&env, 4u32.into_val(&env)],
                sub_invokes: &[],
            },
        }])
        .set_status(&4);

    let result = admin_transfer_client.get_admin_transfer(&pool);
    assert!(result.is_none());

    // validate another admin transfer can be created
    admin_transfer_client
        .mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &admin_transfer_id,
                fn_name: &"set_admin_transfer",
                args: vec![
                    &env,
                    pool.clone().into_val(&env),
                    admin.clone().into_val(&env),
                    new_admin.clone().into_val(&env),
                ],
                sub_invokes: &[MockAuthInvoke {
                    contract: &pool,
                    fn_name: &"set_admin",
                    args: vec![&env, admin_transfer_id.clone().into_val(&env)],
                    sub_invokes: &[],
                }],
            },
        }])
        .set_admin_transfer(&pool, &admin, &new_admin);

    // -> validate chain state
    let result = admin_transfer_client.get_admin_transfer(&pool).unwrap();
    assert_eq!(result.new_admin, new_admin);
    assert_eq!(result.cur_admin, admin);
}
