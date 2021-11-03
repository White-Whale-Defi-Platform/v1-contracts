use cosmwasm_std::{
Uint64,CosmosMsg, from_binary, Binary, StdResult, Response};
use schemars::JsonSchema;
use std::fmt;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Operation <T: Clone + fmt::Debug + PartialEq + JsonSchema> {
    op_code: Uint64,
    attributes: Vec<Option<T>>,
}

pub fn process_operation<T: Clone + fmt::Debug + PartialEq + JsonSchema>(template: Binary, operation: Operation<T>) -> StdResult<None> {
    let json_template: Value = from_binary(&template)?;
    fn amend(config: Config, new_rules: &Value) -> crate::Result<Config> {
        let config: Value = serde_json::to_value(&config).unwrap();
    
        let mut config: BTreeMap<String, Value> = serde_json::from_value(config).unwrap();
        let new_rules: BTreeMap<String, Value> = serde_json::from_value(new_rules.clone())?;
        for (k, v) in new_rules {
            config.insert(k, v);
        }
    
        let config: Value = serde_json::to_value(&config).unwrap();
        Ok(serde_json::from_value(config)?)
    }
}