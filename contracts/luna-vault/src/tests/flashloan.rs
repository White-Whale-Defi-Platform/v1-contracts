use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{coins, Addr, BlockInfo, Timestamp, Uint128};
use cw20::{Cw20Coin, Cw20Contract, Cw20ExecuteMsg};
use terra_multi_test::Executor;
use terraswap::asset::{Asset, AssetInfo};

use crate::tests::anchor_mock::{contract_anchor_mock, MockInstantiateMsg as AnchorMsg};
use crate::tests::tswap_mock::{contract_receiver_mock, set_liq_token_addr, MockInstantiateMsg};
use white_whale::denom::UST_DENOM;
use white_whale::treasury::msg::InstantiateMsg as TreasuryInitMsg;
use white_whale::luna_vault::msg::*;

use crate::contract::{execute, DEFAULT_LP_TOKEN_NAME, DEFAULT_LP_TOKEN_SYMBOL};
use crate::error::LunaVaultError;
use crate::state::STATE;
use crate::tests::common::{ARB_CONTRACT, TEST_CREATOR};
use crate::tests::common_integration::{
    contract_cw20_token, contract_stablecoin_vault, contract_treasury, instantiate_msg, mock_app,
};
use crate::tests::instantiate::mock_instantiate;
use crate::tests::mock_querier::mock_dependencies;

#[test]
fn unsuccessful_flashloan_no_base_token() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let whitelisted_contracts = STATE
        .load(deps.as_mut().storage)
        .unwrap()
        .whitelisted_contracts;
    assert_eq!(0, whitelisted_contracts.len());

    let msg = ExecuteMsg::FlashLoan {
        payload: FlashLoanPayload {
            requested_asset: Asset {
                info: AssetInfo::NativeToken {
                    denom: "uluna".to_string(),
                },
                amount: Default::default(),
            },
            callback: Default::default(),
        },
    };
    let info = mock_info(TEST_CREATOR, &[]);

    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(LunaVaultError::Std(_)) => (),
        _ => panic!("Must return StdError::generic_err from DepositInfo::assert"),
    }
}

#[test]
fn unsuccessful_flashloan_not_whitelisted() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let whitelisted_contracts = STATE
        .load(deps.as_mut().storage)
        .unwrap()
        .whitelisted_contracts;
    assert_eq!(0, whitelisted_contracts.len());

    let msg = ExecuteMsg::FlashLoan {
        payload: FlashLoanPayload {
            requested_asset: Asset {
                info: AssetInfo::NativeToken {
                    denom: UST_DENOM.to_string(),
                },
                amount: Default::default(),
            },
            callback: Default::default(),
        },
    };
    let info = mock_info(TEST_CREATOR, &[]);

    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(LunaVaultError::NotWhitelisted {}) => (),
        _ => panic!("Must return StableVaultError::NotWhitelisted"),
    }
}

#[test]
fn unsuccessful_flashloan_broke() {
    // Create the owner account
    let owner = Addr::unchecked("owner");

    // Define a mock_app to be used for storing code and instantiating
    let mut router = mock_app();
    router
        .init_bank_balance(&owner, coins(1000, "uusd"))
        .unwrap();
    // Store the stablecoin vault as a code object
    let vault_id = router.store_code(contract_stablecoin_vault());
    // Store the gov contract as a code object
    let treasury_id = router.store_code(contract_treasury());
    // Store the profit check needed for the vault on provide and withdrawal of liquidity as well as trading actions
    let anchor_id = router.store_code(contract_anchor_mock());

    // Set the block height and time, we will later modify this to simulate time passing
    let initial_block = BlockInfo {
        height: 0,
        time: Timestamp::from_seconds(1000),
        chain_id: "terra-cosmwasm-testnet".to_string(),
    };
    router.set_block(initial_block);
    // Lastly, store our terrswap mock which is a slimmed down Terraswap with no real functionality
    let terraswap_id = router.store_code(contract_receiver_mock());

    // Store whale token which is a CW20 and get its code ID
    let cw20_code_id = router.store_code(contract_cw20_token());

    // Create the Whale token giving owner some initial balance
    let msg = cw20_base::msg::InstantiateMsg {
        name: "White Whale".to_string(),
        symbol: "WHALE".to_string(),
        decimals: 6,
        initial_balances: vec![Cw20Coin {
            address: owner.to_string(),
            amount: Uint128::new(5000),
        }],
        mint: None,
        marketing: None,
    };
    let whale_token_instance = router
        .instantiate_contract(cw20_code_id, owner.clone(), &msg, &[], "WHALE", None)
        .unwrap();

    // Create the Whale token giving owner some initial balance
    let msg = cw20_base::msg::InstantiateMsg {
        name: "Anchor UST".to_string(),
        symbol: "aUST".to_string(),
        decimals: 2,
        initial_balances: vec![Cw20Coin {
            address: owner.to_string(),
            amount: Uint128::new(5000),
        }],
        mint: None,
        marketing: None,
    };
    let aust_token_instance = router
        .instantiate_contract(cw20_code_id, owner.clone(), &msg, &[], "aUST", None)
        .unwrap();

    // set up cw20 helpers
    let cash = Cw20Contract(whale_token_instance.clone());

    // get owner balance
    let owner_balance = cash.balance(&router, owner.clone()).unwrap();
    // Verify the funds have been received
    assert_eq!(owner_balance, Uint128::new(5000));

    // Setup Treasury
    let chest_msg = TreasuryInitMsg {
        // admin_addr: owner.to_string(),
        // whale_token_addr: whale_token_instance.to_string(),
        // spend_limit: Uint128::from(1_000_000u128),
    };

    // Instantiate the Terraswap Mock, note this just has a simple init as we have removed everything except mocks
    let tswap_addr = router
        .instantiate_contract(
            terraswap_id,
            owner.clone(),
            &MockInstantiateMsg {},
            &[],
            "TSWAP",
            None,
        )
        .unwrap();

    // Setup the treasury contract
    let treasury_addr = router
        .instantiate_contract(
            treasury_id,
            owner.clone(),
            &chest_msg,
            &[],
            "TREASURY",
            None,
        )
        .unwrap();

    // Instantiate the Terraswap Mock, note this just has a simple init as we have removed everything except mocks
    let anchor_addr = router
        .instantiate_contract(anchor_id, owner.clone(), &AnchorMsg {}, &[], "TSWAP", None)
        .unwrap();

    // First prepare an InstantiateMsg for vault contract with the mock terraswap token_code_id
    let vault_msg = instantiate_msg(
        terraswap_id,
        treasury_addr.to_string(),
        anchor_addr.to_string(),
        aust_token_instance.to_string(),
    );

    // Next setup the vault with the gov contract as the 'owner'
    let vault_addr = router
        .instantiate_contract(
            vault_id,
            owner.clone(),
            &vault_msg,
            &[],
            "VAULT",
            Some(owner.to_string()),
        )
        .unwrap();

    // Make a mock LP token
    let msg = cw20_base::msg::InstantiateMsg {
        name: DEFAULT_LP_TOKEN_NAME.to_string(),
        symbol: DEFAULT_LP_TOKEN_SYMBOL.to_string(),
        decimals: 6,
        initial_balances: vec![Cw20Coin {
            address: owner.to_string(),
            amount: Uint128::new(5000),
        }],
        mint: None,
        marketing: None,
    };
    let lp_token = router
        .instantiate_contract(
            cw20_code_id,
            vault_addr.clone(),
            &msg,
            &[],
            DEFAULT_LP_TOKEN_SYMBOL,
            Some(vault_addr.to_string()),
        )
        .unwrap();
    // Set the address of the liquidity token mock
    set_liq_token_addr(Addr::unchecked("Contract #7").to_string());

    // Need to give a mocked token to user
    // Prepare
    let send_msg = Cw20ExecuteMsg::Transfer {
        recipient: lp_token.to_string(),
        amount: Uint128::new(1000),
    };
    let _ = router
        .execute_contract(owner.clone(), lp_token.clone(), &send_msg, &[])
        .unwrap();

    // Ensure addresses are not equal to each other
    assert_ne!(treasury_addr, vault_addr);
    assert_ne!(vault_addr, tswap_addr);

    // Whitelist contract
    let msg = ExecuteMsg::AddToWhitelist {
        contract_addr: ARB_CONTRACT.to_string(),
    };
    router
        .execute_contract(owner.clone(), vault_addr.clone(), &msg, &[])
        .unwrap();

    let msg = ExecuteMsg::FlashLoan {
        payload: FlashLoanPayload {
            requested_asset: Asset {
                info: AssetInfo::NativeToken {
                    denom: UST_DENOM.to_string(),
                },
                amount: Uint128::new(1000),
            },
            callback: Default::default(),
        },
    };

    let res = router.execute_contract(Addr::unchecked(ARB_CONTRACT), vault_addr.clone(), &msg, &[]);

    match res {
        Err(_) => (), //match StableVaultError::Broke
        _ => panic!("Must return StableVaultError::Broke"),
    }
}

#[test]
fn successful_flashloan_without_withdrawing_aust() {
    // Create the owner account
    let owner = Addr::unchecked("owner");

    // Define a mock_app to be used for storing code and instantiating
    let mut router = mock_app();
    router
        .init_bank_balance(&owner, coins(100_000_000, UST_DENOM))
        .unwrap();
    // Store the stablecoin vault as a code object
    let vault_id = router.store_code(contract_stablecoin_vault());
    // Store the gov contract as a code object
    let treasury_id = router.store_code(contract_treasury());
    // Store the profit check needed for the vault on provide and withdrawal of liquidity as well as trading actions
    let anchor_id = router.store_code(contract_anchor_mock());

    // Set the block height and time, we will later modify this to simulate time passing
    let initial_block = BlockInfo {
        height: 0,
        time: Timestamp::from_seconds(100),
        chain_id: "terra-cosmwasm-testnet".to_string(),
    };
    router.set_block(initial_block);
    // Lastly, store our terrswap mock which is a slimmed down Terraswap with no real functionality
    let terraswap_id = router.store_code(contract_receiver_mock());

    // Store whale token which is a CW20 and get its code ID
    let cw20_code_id = router.store_code(contract_cw20_token());

    // Create the Whale token giving owner some initial balance
    let msg = cw20_base::msg::InstantiateMsg {
        name: "White Whale".to_string(),
        symbol: "WHALE".to_string(),
        decimals: 6,
        initial_balances: vec![Cw20Coin {
            address: owner.to_string(),
            amount: Uint128::new(5_000),
        }],
        mint: None,
        marketing: None,
    };
    let whale_token_instance = router
        .instantiate_contract(cw20_code_id, owner.clone(), &msg, &[], "WHALE", None)
        .unwrap();

    // Create the Anchor UST token giving owner some initial balance
    let msg = cw20_base::msg::InstantiateMsg {
        name: "Anchor UST".to_string(),
        symbol: "aUST".to_string(),
        decimals: 2,
        initial_balances: vec![Cw20Coin {
            address: owner.to_string(),
            amount: Uint128::new(5_000),
        }],
        mint: None,
        marketing: None,
    };
    let aust_token_instance = router
        .instantiate_contract(cw20_code_id, owner.clone(), &msg, &[], "aUST", None)
        .unwrap();

    // set up cw20 helpers
    let cash = Cw20Contract(whale_token_instance.clone());

    // get owner balance
    let owner_balance = cash.balance(&router, owner.clone()).unwrap();
    // Verify the funds have been received
    assert_eq!(owner_balance, Uint128::new(5_000));

    // Setup Treasury
    let chest_msg = TreasuryInitMsg {
        // admin_addr: owner.to_string(),
        // whale_token_addr: whale_token_instance.to_string(),
        // spend_limit: Uint128::from(1_000_000u128),
    };

    // Instantiate the Terraswap Mock, note this just has a simple init as we have removed everything except mocks
    let tswap_addr = router
        .instantiate_contract(
            terraswap_id,
            owner.clone(),
            &MockInstantiateMsg {},
            &[],
            "TSWAP",
            None,
        )
        .unwrap();

    // Setup the treasury contract
    let treasury_addr = router
        .instantiate_contract(
            treasury_id,
            owner.clone(),
            &chest_msg,
            &[],
            "TREASURY",
            None,
        )
        .unwrap();

    // Instantiate the Terraswap Mock, note this just has a simple init as we have removed everything except mocks
    let anchor_addr = router
        .instantiate_contract(anchor_id, owner.clone(), &AnchorMsg {}, &[], "TSWAP", None)
        .unwrap();

    // First prepare an InstantiateMsg for vault contract with the mock terraswap token_code_id
    let vault_msg = instantiate_msg(
        terraswap_id,
        treasury_addr.to_string(),
        anchor_addr.to_string(),
        aust_token_instance.to_string(),
    );

    // Next setup the vault with the gov contract as the 'owner'
    let vault_addr = router
        .instantiate_contract(
            vault_id,
            owner.clone(),
            &vault_msg,
            &coins(50_000_000, UST_DENOM),
            "VAULT",
            Some(owner.to_string()),
        )
        .unwrap();

    // Make a mock LP token
    let msg = cw20_base::msg::InstantiateMsg {
        name: DEFAULT_LP_TOKEN_NAME.to_string(),
        symbol: DEFAULT_LP_TOKEN_SYMBOL.to_string(),
        decimals: 6,
        initial_balances: vec![Cw20Coin {
            address: owner.to_string(),
            amount: Uint128::new(5_000),
        }],
        mint: None,
        marketing: None,
    };
    let lp_token = router
        .instantiate_contract(
            cw20_code_id,
            vault_addr.clone(),
            &msg,
            &[],
            DEFAULT_LP_TOKEN_SYMBOL,
            Some(vault_addr.to_string()),
        )
        .unwrap();
    // Set the address of the liquidity token mock
    set_liq_token_addr(Addr::unchecked("Contract #7").to_string());

    // Need to give a mocked token to user
    // Prepare
    let send_msg = Cw20ExecuteMsg::Transfer {
        recipient: lp_token.to_string(),
        amount: Uint128::new(1_000),
    };
    router
        .execute_contract(owner.clone(), lp_token.clone(), &send_msg, &[])
        .unwrap();

    // Ensure addresses are not equal to each other
    assert_ne!(treasury_addr, vault_addr);
    assert_ne!(vault_addr, tswap_addr);

    ////////////

    // Whitelist contract
    let msg = ExecuteMsg::AddToWhitelist {
        contract_addr: ARB_CONTRACT.to_string(),
    };
    let _ = router
        .execute_contract(owner.clone(), vault_addr.clone(), &msg, &[])
        .unwrap();

    // send the flashloan
    let msg = ExecuteMsg::FlashLoan {
        payload: FlashLoanPayload {
            requested_asset: Asset {
                info: AssetInfo::NativeToken {
                    denom: UST_DENOM.to_string(),
                },
                amount: Uint128::new(1_000),
            },
            callback: Default::default(),
        },
    };

    let _ = router.execute_contract(Addr::unchecked(ARB_CONTRACT), vault_addr.clone(), &msg, &[]);
}
