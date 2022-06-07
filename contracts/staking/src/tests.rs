use crate::contract::{handle, init, query};
use crate::error::ContractError;
use crate::msgs::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, HumanAddr};

#[test]
fn staking() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: HumanAddr::from("owner0000"),
        native_token_denom: "orai".to_string(),
        native_token: HumanAddr::from("owner0000"),
        asset_token: HumanAddr::from("owner0000"),
        base_apr: Decimal256::percent(30),
        orchai_token: HumanAddr::from("owner0000"),
    };

    let info = mock_info("addr0000", &[]);
    let _res = init(deps.as_mut(), mock_env(), info, msg).unwrap();

    // update owner
    let info = mock_info("owner0000", &[]);
    let msg = ExecuteMsg::StakingOrai {
        amount: Uint256::from(100u128),
    };

    let res = handle(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
}
