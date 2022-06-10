use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use helpers::msgs::{
    BorrowerInfoResponse, CollateralBallanceResponse, ConfigResponse, DepositRateResponse,
    ExecuteMsg, InstantiateMsg, OraiBalanceResponse, QueryMsg, TotalBallanceDepositResponse,
};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(BorrowerInfoResponse), &out_dir);
    export_schema(&schema_for!(CollateralBallanceResponse), &out_dir);
    export_schema(&schema_for!(ConfigResponse), &out_dir);
    export_schema(&schema_for!(DepositRateResponse), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(OraiBalanceResponse), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(TotalBallanceDepositResponse), &out_dir);
}
