// Add the custom dapp-specific message commands here
use cosmwasm_std::{to_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, Uint128, WasmMsg};
use cw20::{Cw20ExecuteMsg};
use terraswap::asset::{AssetInfo};
use white_whale::treasury::dapp_base::error::BaseDAppError;
use white_whale::treasury::dapp_base::state::BASESTATE;
use terraswap::pair::{Cw20HookMsg};
use white_whale::treasury::msg::{send_to_treasury};
use crate::state::{State, STATE};
use white_whale::query::terraswap::query_asset_balance;



use crate::contract::BuyBackResult;
use crate::error::BuyBackError;

pub fn handle_buyback_whale(deps: DepsMut, _env: Env, msg_info: MessageInfo, amount_to_buy: Uint128) -> BuyBackResult {
        let state = BASESTATE.load(deps.storage)?;
        let config: State = STATE.load(deps.storage)?;

        // Check if caller is trader.
        if msg_info.sender != state.trader {
            return Err(BuyBackError::BaseDAppError(BaseDAppError::Unauthorized {}));
        }
        // Prepare empty message vec
        let mut messages: Vec<CosmosMsg> = vec![];
        let treasury_address = state.treasury_address;
        // vUST INFO
        let vust_info = AssetInfo::Token{
            contract_addr: config.vust_token.to_string()
        };

        // Get balance and ensure Treasury has enough vUST
        if query_asset_balance(deps.as_ref(), &vust_info, treasury_address.clone())? < amount_to_buy {
            return Err(BuyBackError::NotEnoughFunds {});
        }


        // Define the stake voting tokens msg and wrap it in a Cw20ExecuteMsg
        let msg = Cw20HookMsg::Swap {
            belief_price: None,
            max_spread: None,
            to: None
        };

        // Prepare cw20 message with our attempt to buy tokens
        let send_msg = Cw20ExecuteMsg::Send {
            contract: config.whale_vust_lp.to_string(),
            amount: amount_to_buy,
            msg: to_binary(&msg).unwrap(),
        };
        // Prepare the final CosmosMsg to be sent to vUST token to trigger the Receive() on the pair via the CW20Hook
        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.vust_token.to_string(),
            msg: to_binary(&send_msg)?,
            funds: vec![],
        });

        println!("{:?}", msg);

        messages.push(msg);
        Ok(Response::new().add_message(send_to_treasury(messages, &treasury_address)?))
    }
