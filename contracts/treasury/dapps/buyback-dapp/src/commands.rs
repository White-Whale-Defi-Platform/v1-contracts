// Add the custom dapp-specific message commands here
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Decimal, Deps, DepsMut, Env, Fraction, MessageInfo, Response, Uint128,
    WasmMsg,
};
use terraswap::asset::{Asset, AssetInfo};
use white_whale::treasury::dapp_base::error::BaseDAppError;
use white_whale::treasury::dapp_base::state::BASESTATE;
use terraswap::pair::ExecuteMsg as PairExecuteMsg;
use white_whale::treasury::msg::send_to_treasury;
use crate::state::{State, STATE};
use white_whale::query::terraswap::query_asset_balance;



use crate::contract::BuyBackResult;

pub fn handle_buyback_whale(deps: DepsMut, env: Env, msg_info: MessageInfo, amount_to_buy: Uint128) -> BuyBackResult {
        let state = BASESTATE.load(deps.storage)?;
        let config: State = STATE.load(deps.storage)?;
        
        // Check if caller is trader.
        if msg_info.sender != state.trader {
            return Err(BaseDAppError::Unauthorized {});
        }
        // Prepare empty message vec
        let mut messages: Vec<CosmosMsg> = vec![];
        let treasury_address = state.treasury_address;
        // Validate whale token and setup an AssetInfo
        let whale_token = config.whale_token;
        let whale_info = AssetInfo::Token {
            contract_addr: whale_token.to_string(),
        };
        // UST INFO 
        let ust_info = AssetInfo::NativeToken {
            denom: "uusd".to_string()
        };
        // vUST INFO
        let vust_info = AssetInfo::Token{
            contract_addr: config.vust_token.to_string()
        };
        
        // Get balance and ensure Treasury has enough vUST
        if query_asset_balance(deps.as_ref(), &vust_info, treasury_address.clone())? < amount_to_buy {
            return Err(BaseDAppError::Broke {});
        }
        // Prepare the swap amount with vUST Token Info and the amount_to_buy
        let swap_amount = Asset {
            info: vust_info.clone(),
            amount: amount_to_buy,
        }.deduct_tax(&deps.querier)?.amount;
        // Preapre the offer asset for buying WHALE
        let offer_asset = Asset {
            info: vust_info,
            amount: swap_amount,
        };
        // Buyback WHALE by providing vUST as the offer asset with None for all other options 
        // Buyback WHALE at any price or spread to fill the 'amount_to_buy'. Limited by the amount of vUST held by the Treasury
        let whale_purchase_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.whale_vust_lp.to_string(),
            msg: to_binary(&PairExecuteMsg::Swap {
                offer_asset: offer_asset,
                belief_price: None,
                max_spread: None,
                to: None,
            })?,
            funds: vec![],
        });

        messages.push(whale_purchase_msg);
        Ok(Response::new().add_message(send_to_treasury(messages, &treasury_address)?))
    }