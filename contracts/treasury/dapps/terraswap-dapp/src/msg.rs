use cosmwasm_std::{Decimal, Uint128};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub treasury_address: String,
    pub trader: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    ProvideLiquidity {
        pool_id: String,
        main_asset_id: String,
        amount: Uint128,
    },
    WithdrawLiquidity {
        pool_id: String,
        amount: Uint128,
    },
    SwapAsset {
        offer_id: String,
        pool_id: String,
        amount: Uint128,
        max_spread: Option<Decimal>,
        belief_price: Option<Decimal>,
    },
    // Add methods
    UpdateConfig {
        treasury_address: Option<String>,
        trader: Option<String>,
    },
    UpdateAddressBook {
        to_add: Vec<(String, String)>,
        to_remove: Vec<String>,
    },
    SetAdmin {
        admin: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    AddressBook {
        id: String,
    },
}
