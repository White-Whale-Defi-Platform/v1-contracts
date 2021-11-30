use crate::error::FederatorError;
use crate::state::INSTRUCTION_SET;
use cosmwasm_std::{
    from_binary, to_binary, CosmosMsg, Deps, Empty, StdError, StdResult, Uint64, WasmMsg,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, to_string, Map, Value};
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Operation {
    op_code: Uint64,
    attributes: Vec<Value>,
}

impl Operation {
    pub fn execute(
        &mut self,
        deps: Deps,
    ) -> Result<(CosmosMsg<Empty>, Vec<(String, String)>), FederatorError> {
        // Get binary from storage
        let (contract_addr, bin_template) =
            INSTRUCTION_SET.load(deps.storage, &self.op_code.to_string())?;
        let mut template: Value = from_binary(&bin_template)?;
        // This template has the following structure:
        // template = {
        //     "execute_msg": {
        //         "attr1": {
        //                 ...
        //             },
        //         "...": ...
        //     }
        // }
        //
        // So we should iterate in a branching manner over all the Values and fill them iteratively
        // if Value => deconstruct until actual value is found.
        // Compare type of template with attribute type.
        // Fill in with provided attribute.
        // go to next key and repeat.
        // values of the same depth are filled in alphabetic order
        // attributes are popped so they should be stored in reverse order

        // Fill template
        let logs = rec_fill_template(&mut template, &mut self.attributes)?;

        Ok((
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&to_string(&template)?)?,
                funds: vec![],
            }),
            logs,
        ))
    }
}

pub fn is_same_value_type(v1: &Value, v2: &Value) -> bool {
    if v1.is_array() && v2.is_array()
        || v1.is_number() && v2.is_number()
        || v1.is_boolean() && v2.is_boolean()
        || v1.is_string() && v2.is_string()
        || v1.is_object() && v2.is_object()
    {
        return true;
    }
    false
}

/// Recursively fills the template with the provided attribute stack.
fn rec_fill_template(
    object: &mut Value,
    attr_stack: &mut Vec<Value>,
) -> StdResult<Vec<(String, String)>> {
    // Parse to Map, if val is not an object,
    let obj_map: &mut Map<String, Value> = object.as_object_mut().unwrap();
    let template = obj_map.values_mut().collect::<Vec<&mut Value>>();
    let prints: Vec<(String, String)> = vec![];
    for value in template {
        prints.push(("Before fill".to_string(), format!("{}", value)));
        if value.is_object() {
            rec_fill_template(value, attr_stack);
        } else {
            // attr_stack should be same length as nr. of to-full values.
            // Throws error otherwise
            if is_same_value_type(value, attr_stack.last().unwrap()) {
                *value = json!(attr_stack.pop().unwrap());
            } else {
                return Err(StdError::GenericErr {
                    msg: format!(
                        "Stack attribute {} and template {} are not the same type",
                        attr_stack.last().unwrap(),
                        value
                    ),
                });
            }
        }
        prints.push(("After fill".to_string(), format!("{}", value)));
    }
    Ok(prints)
}
