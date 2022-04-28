use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas, schema_for};

use luna_vault::pool_info::PoolInfo;
use luna_vault::state::State;
use white_whale::luna_vault::msg::{ExecuteMsg, InstantiateMsg, PoolResponse, VaultQueryMsg};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema_with_title(
        &mut schema_for!(VaultQueryMsg),
        &out_dir,
        "LunaVaultQueryMsg",
    );
    export_schema(&schema_for!(State), &out_dir);
    export_schema(&schema_for!(PoolResponse), &out_dir);
    export_schema(&schema_for!(PoolInfo), &out_dir);
}
