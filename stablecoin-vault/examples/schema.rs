use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use stablecoin_vault::pool_info::PoolInfo;
use stablecoin_vault::msg::{PoolResponse, HandleMsg, InitMsg, QueryMsg};
use stablecoin_vault::state::State;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InitMsg), &out_dir);
    export_schema(&schema_for!(HandleMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(State), &out_dir);
    export_schema(&schema_for!(PoolResponse), &out_dir);
    export_schema(&schema_for!(PoolInfo), &out_dir);
}
