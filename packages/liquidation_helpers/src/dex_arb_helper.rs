use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{to_binary, Addr, CosmosMsg, Decimal, StdResult, Uint128, WasmMsg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use terraswap::asset::AssetInfo;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: String,
    pub ust_vault_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateConfigMsg {
    pub owner: Option<String>,
    pub ust_vault_address: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig {
        new_config: UpdateConfigMsg,
    },
    AddPool {
        dex: DexInfo,
        new_asset: PoolInfo,
    },
    InitiateUstArbitrage {
        buy_side: DexInfo,
        sell_side: DexInfo,
        ust_to_borrow: Uint256,
        asset: AssetInfo,
    },
    InitiateCustomArbitrage {
        tx_infos: Vec<TxInfo>,
    },

    /// Callbacks; only callable by the contract itself.
    Callback(CallbackMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CallbackMsg {
    InitiateArbCallback {
        buy_side: DexInfo,
        sell_side: DexInfo,
        asset: AssetInfo,
    },
    AfterBuyCallback {
        sell_side: DexInfo,
        asset: AssetInfo,
        amount: Uint256,
    },
    AfterSellCallback {
        arb_amount: Uint256,
    },
}

// Modified from
// https://github.com/CosmWasm/cosmwasm-plus/blob/v0.2.3/packages/cw20/src/receiver.rs#L15
impl CallbackMsg {
    pub fn to_cosmos_msg(&self, contract_addr: &Addr) -> StdResult<CosmosMsg> {
        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: String::from(contract_addr),
            msg: to_binary(&ExecuteMsg::Callback(self.clone()))?,
            funds: vec![],
        }))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    State {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub ust_vault_address: String,
    pub loop_pools: Vec<PoolInfo>,
    pub terraswap_pools: Vec<PoolInfo>,
    pub astroport_pools: Vec<PoolInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub total_arbs: u64,
    pub total_ust_profit: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PoolInfo {
    pub asset_token: AssetInfo,
    pub pair_address: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DexInfo {
    Astroport {},
    Terraswap {},
    Loop {},
}
