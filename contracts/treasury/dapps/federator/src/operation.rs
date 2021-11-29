use cosmwasm_std::{Binary, CosmosMsg, Deps, Empty, Response, StdResult, Uint64, from_binary};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::fmt;
use crate::state::INSTRUCTION_SET;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Operation<T: Clone + fmt::Debug + PartialEq + JsonSchema> {
    op_code: Uint64,
    attributes: Vec<T>,
}

impl <T: Clone + fmt::Debug + PartialEq + JsonSchema> Operation <T> {
    pub fn process(
        &mut self,
        deps: Deps,
    ) -> StdResult<Binary> {
        // Get binary from storage 
        let mut template: Value = from_binary(&INSTRUCTION_SET.load(deps.storage, &self.op_code.to_string())?)?;
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
        rec_fill_value(&mut template, &mut self.attributes );

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

fn rec_fill_value(object: &mut Value, attr_stack: &mut Vec<Value>) -> StdResult<()>{
    // Parse to Map, if val is not an object, 
    let obj_map: &mut Map<String, Value> = object.as_object_mut().unwrap();
    let template = obj_map.values_mut().collect::<Vec<&mut Value>>();
    println!("template: {:?}", &template);

    for value in template {
        println!("Before filling: {:?}", &value);
        if value.is_object(){
            rec_fill_value(value, attr_stack);
        }else {
            // attr_stack should be same length as nr. of to-full values. 
            // Throws error otherwise
            if is_same_value_type(value, attr_stack.last().unwrap()) {
                *value = json!(attr_stack.pop()?);
            } else {
                panic!("Wrong type!")
            }
        }
        println!("After filling: {:?}", &value);

    }
    println!("{:?}", val);
    Ok(())
}
