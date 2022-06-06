use crate::contract::{handle, init, query};
use crate::error::ContractError;
use crate::msgs::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, HumanAddr};

#[test]
fn update_config() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: HumanAddr::from("owner0000"),
        native_token_denom: "orai".to_string(),
        native_token: HumanAddr::from("owner0000"),
        asset_token: HumanAddr::from("owner0000"),
        base_apr: Uint256::zero(),
        orchai_token: HumanAddr::from("owner0000"),
    };

    let info = mock_info("addr0000", &[]);
    let _res = init(deps.as_mut(), mock_env(), info, msg).unwrap();

    // update owner
    let info = mock_info("owner0000", &[]);
    let msg = ExecuteMsg::UpdateConfig {
        owner: Some(HumanAddr::from("owner0001")),
        asset_token: Some(HumanAddr::from("owner0002")),
        base_apr: Some(Uint256::one()),
    };

    let res = handle(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
}
