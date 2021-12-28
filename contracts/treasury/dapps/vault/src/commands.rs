use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    from_binary, to_binary, Addr, BankMsg, Binary, CosmosMsg, Decimal, Deps, DepsMut, Env,
    Fraction, MessageInfo, Response, StdResult, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::{Cw20HookMsg, PoolResponse};
use white_whale::treasury::dapp_base::state::ADMIN;

use crate::contract::VaultResult;
use crate::error::VaultError;
use crate::msg::{DepositHookMsg, ExecuteMsg};
use crate::state::{Pool, State, FEE, POOL, STATE};
use terraswap::querier::{query_supply, query_token_balance};
use white_whale::fee::Fee;
use white_whale::query::memory::query_assets_from_mem;
use white_whale::query::terraswap::{query_asset_balance, query_pool};
use white_whale::query::vault::query_total_value;
use white_whale::treasury::dapp_base::common::PAIR_POSTFIX;
use white_whale::treasury::dapp_base::error::BaseDAppError;
use white_whale::treasury::vault_assets::{get_identifier, VaultAsset};

use white_whale::treasury::msg::send_to_treasury;

/// handler function invoked when the stablecoin-vault contract receives
/// a transaction. In this case it is triggered when the LP tokens are deposited
/// into the contract
pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> VaultResult {
    match from_binary(&cw20_msg.msg)? {
        DepositHookMsg::WithdrawLiquidity {} => {
            let state: State = STATE.load(deps.storage)?;
            if msg_info.sender != state.lp_token_addr {
                return Err(VaultError::NotLPToken {
                    token: msg_info.sender.to_string(),
                });
            }
            try_withdraw_liquidity(deps, env, cw20_msg.sender, cw20_msg.amount)
        }
        DepositHookMsg::ProvideLiquidity { asset } => {
            if asset.amount != cw20_msg.amount {
                return Err(VaultError::InvalidAmount {});
            }
            try_provide_liquidity(deps, msg_info, asset, Some(cw20_msg.sender))
        }
    }
}

pub fn try_provide_liquidity(
    deps: DepsMut,
    msg_info: MessageInfo,
    asset: Asset,
    sender: Option<String>,
) -> VaultResult {
    let pool: Pool = POOL.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;

    let liq_provider = match sender {
        Some(addr) => Addr::unchecked(addr),
        None => msg_info.sender.clone(),
    };

    let assets = query_assets_from_mem(deps.as_ref(), state.base.memory_addr, &pool.assets)?;

    // Init vector for logging
    let mut attrs = vec![];
    // Check if deposit matches claimed deposit.
    if asset.is_native_token() {
        asset.assert_sent_native_token_balance(&msg_info)?;
        attrs.push(("Action:", String::from("Deposit to vault")));
        attrs.push(("Received funds:", asset.to_string()));
    } else {
        // Sender must be vault deposit asset
        if &msg_info.sender.to_string() != get_identifier(assets.get(&pool.deposit_asset).unwrap())
        {
            return Err(VaultError::WrongToken {});
        }
    }

    // Received deposit to vault
    let deposit: Uint128 = asset.amount;

    // Get total value in Vault
    let value = query_total_value(deps.as_ref(), &state.base.treasury_address)?;
    // Get total supply of LP tokens and calculate share
    let total_share = query_supply(&deps.querier, state.lp_token_addr.clone())?;

    let share = if total_share == Uint128::zero() || value.checked_sub(deposit)? == Uint128::zero()
    {
        // Initial share = collateral amount
        deposit
    } else {
        deposit.multiply_ratio(total_share, value - deposit)
    };

    // mint LP token to liq_provider
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: state.lp_token_addr.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Mint {
            recipient: liq_provider.to_string(),
            amount: share,
        })?,
        funds: vec![],
    });

    let response = Response::new().add_attributes(attrs).add_message(msg);

    Ok(response)
}

/// Attempt to withdraw deposits. Fees are calculated and deducted in lp tokens.
/// This allowes the war-chest to accumulate a stake in the vault.
/// The refund is taken out of Anchor if possible.
/// Luna holdings are not eligible for withdrawal.
pub fn try_withdraw_liquidity(
    deps: DepsMut,
    env: Env,
    sender: String,
    amount: Uint128,
) -> VaultResult {
    let pool: Pool = POOL.load(deps.storage)?;
    let state: State = STATE.load(deps.storage)?;
    let fee: Fee = FEE.load(deps.storage)?;
    // Get assets
    let assets = query_assets_from_mem(deps.as_ref(), state.base.memory_addr, &pool.assets)?;

    // Logging var
    let mut attrs = vec![];

    // Calculate share of pool and requested pool value
    let total_share: Uint128 = query_supply(&deps.querier, state.lp_token_addr.clone())?;
    // Get treasury fee in LP tokens
    let treasury_fee = fee.share * amount;
    // Share with fee deducted.
    let share_ratio: Decimal = Decimal::from_ratio(amount - treasury_fee, total_share);

    // Init response
    let response = Response::new();

    // LP token fee
    let lp_token_treasury_fee = Asset {
        info: AssetInfo::Token {
            contract_addr: state.lp_token_addr.to_string(),
        },
        amount: treasury_fee,
    };

    // Construct treasury fee msg
    let treasury_fee_msg = fee.msg(
        deps.as_ref(),
        lp_token_treasury_fee,
        state.base.treasury_address.clone(),
    )?;
    attrs.push(("Treasury fee:", treasury_fee.to_string()));

    // Get asset holdings of vault and calculate amount to return
    let mut pay_back_assets: Vec<Asset> = vec![];
    // Get asset holdings of vault and calculate amount to return
    for info in assets.into_values().into_iter() {
        pay_back_assets.push(Asset {
            info: info.clone(),
            amount: share_ratio
                // query asset held in treasury
                * query_asset_balance(
                    deps.as_ref(),
                    &info.clone(),
                    state.base.treasury_address.clone(),
                )
                .unwrap(),
        });
    }

    // Construct repay msgs
    let mut refunds: Vec<CosmosMsg> = vec![];
    for asset in pay_back_assets.into_iter() {
        if asset.amount != Uint128::zero() {
            // Unchecked ok as sender is already validated by VM
            refunds.push(asset.into_msg(&deps.querier, Addr::unchecked(sender.clone()))?);
        }
    }

    // LP burn msg
    let burn_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: state.lp_token_addr.into(),
        // Burn exludes fee
        msg: to_binary(&Cw20ExecuteMsg::Burn {
            amount: (amount - treasury_fee),
        })?,
        funds: vec![],
    });

    Ok(response
        .add_messages(refunds)
        .add_message(burn_msg)
        .add_message(treasury_fee_msg)
        .add_attribute("action:", "withdraw_liquidity")
        .add_attributes(attrs))
}

pub fn update_pool(
    deps: DepsMut,
    msg_info: MessageInfo,
    deposit_asset: Option<String>,
    assets_to_add: Vec<String>,
    assets_to_remove: Vec<String>,
) -> VaultResult {
    // Only the admin should be able to call this
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    let mut pool = POOL.load(deps.storage)?;

    if let Some(deposit_asset) = deposit_asset {
        pool.deposit_asset = deposit_asset;
    }

    // Add the asset to the vector if not already present
    for asset in assets_to_add.into_iter() {
        if !pool.assets.contains(&asset) {
            pool.assets.push(asset)
        } else {
            return Err(VaultError::AssetAlreadyPresent { asset });
        }
    }

    // Remove asset from vector if present
    for asset in assets_to_remove.into_iter() {
        if pool.assets.contains(&asset) {
            pool.assets.retain(|x| *x != asset)
        } else {
            return Err(VaultError::AssetNotPresent { asset });
        }
    }

    // Save pool
    POOL.save(deps.storage, &pool)?;
    Ok(Response::new().add_attribute("Update:", "Successful"))
}

pub fn set_fee(deps: DepsMut, msg_info: MessageInfo, new_fee: Fee) -> VaultResult {
    // Only the admin should be able to call this
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    let mut fee = FEE.load(deps.storage)?;

    if fee.share > Decimal::one() || fee.share < Decimal::zero() {
        return Err(VaultError::InvalidFee {});
    }

    fee = new_fee;
    FEE.save(deps.storage, &fee)?;
    Ok(Response::new().add_attribute("Update:", "Successful"))
}
