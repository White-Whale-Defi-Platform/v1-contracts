// Add the custom dapp-specific message commands here
use cosmwasm_std::{to_binary, Binary, CosmosMsg, Decimal, Deps, DepsMut, Env, Fraction, MessageInfo, Response, Uint128, WasmMsg, BankMsg};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use terraswap::asset::{Asset, AssetInfo};
use white_whale::treasury::dapp_base::error::BaseDAppError;
use white_whale::treasury::dapp_base::state::BASESTATE;
use terraswap::pair::{ExecuteMsg as PairExecuteMsg, Cw20HookMsg};
use white_whale::denom::UST_DENOM;
use white_whale::treasury::msg::{ExecuteMsg, send_to_treasury};
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
        };
        // Prepare the offer asset for buying WHALE
        let offer_asset = Asset {
            info: vust_info,
            amount: amount_to_buy,
        };

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
