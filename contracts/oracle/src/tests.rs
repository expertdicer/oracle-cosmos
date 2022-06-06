use crate::contract::{handle, init, query};
use crate::error::ContractError;
use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{from_binary, HumanAddr};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use crate::msgs::{
    ConfigResponse, ExecuteMsg, FeederResponse, InstantiateMsg, PriceResponse, PricesResponse,
    PricesResponseElem, QueryMsg,
};
use std::str::FromStr;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: HumanAddr::from("owner0000"),
        base_asset: "base0000".to_string(),
    };

    let info = mock_info("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let res = init(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let value: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!("owner0000", value.owner.as_str());
    assert_eq!("base0000", &value.base_asset);
}

#[test]
fn update_config() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: HumanAddr::from("owner0000"),
        base_asset: "base0000".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    let _res = init(deps.as_mut(), mock_env(), info, msg).unwrap();

    // update owner
    let info = mock_info("owner0000", &[]);
    let msg = ExecuteMsg::UpdateConfig {
        owner: Some(HumanAddr::from("owner0001")),
    };

    let res = handle(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let value: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!("owner0001", value.owner.as_str());
    assert_eq!("base0000", &value.base_asset);

    // Unauthorized err
    let info = mock_info("owner0000", &[]);
    let msg = ExecuteMsg::UpdateConfig { owner: None };

    let res = handle(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::Unauthorized {}) => (),
        _ => panic!("Must return unauthorized error"),
    }
}

#[test]
fn register_feeder() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: HumanAddr::from("owner0000"),
        base_asset: "base0000".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    let _res = init(deps.as_mut(), mock_env(), info, msg).unwrap();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::RegisterFeeder {
        asset: HumanAddr::from("mAAPL"),
        feeder: HumanAddr::from("feeder0000"),
    };

    let res = handle(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::Unauthorized {}) => (),
        _ => panic!("DO NOT ENTER HERE"),
    }

    let info = mock_info("owner0000", &[]);
    let _res = handle(deps.as_mut(), mock_env(), info, msg).unwrap();
    let feeder_res: FeederResponse = from_binary(
        &query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Feeder {
                asset: HumanAddr::from("mAAPL"),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        feeder_res,
        FeederResponse {
            asset: HumanAddr::from("mAAPL"),
            feeder: HumanAddr::from("feeder0000"),
        }
    );
}

#[test]
fn feed_price() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: HumanAddr::from("owner0000"),
        base_asset: "base0000".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    let _res = init(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Register feeder for mAAPL
    let msg = ExecuteMsg::RegisterFeeder {
        asset: HumanAddr::from("mAAPL"),
        feeder: HumanAddr::from("feeder0000"),
    };
    let info = mock_info("owner0000", &[]);
    let _res = handle(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    // Register feeder for mGOGL
    let msg = ExecuteMsg::RegisterFeeder {
        asset: HumanAddr::from("mGOGL"),
        feeder: HumanAddr::from("feeder0000"),
    };
    let _res = handle(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Feed prices
    let info = mock_info("feeder0000", &[]);
    let env = mock_env();
    let msg = ExecuteMsg::FeedPrice {
        prices: vec![
            ("mAAPL".to_string(), Decimal256::from_str("1.2").unwrap()),
            ("mGOGL".to_string(), Decimal256::from_str("2.2").unwrap()),
        ],
    };
    let _res = handle(deps.as_mut(), env.clone(), info, msg).unwrap();

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Price {
            base: "mAAPL".to_string(),
            quote: "base0000".to_string(),
        },
    )
    .unwrap();
    let value: PriceResponse = from_binary(&res).unwrap();

    assert_eq!(
        value,
        PriceResponse {
            rate: Decimal256::from_str("1.2").unwrap(),
            last_updated_base: env.block.time,
            last_updated_quote: 9999999999,
        }
    );

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Price {
            base: "mGOGL".to_string(),
            quote: "mAAPL".to_string(),
        },
    )
    .unwrap();
    let value: PriceResponse = from_binary(&res).unwrap();

    assert_eq!(
        value,
        PriceResponse {
            rate: Decimal256::from_str("1.833333333333333333").unwrap(),
            last_updated_base: env.block.time,
            last_updated_quote: env.block.time,
        }
    );

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Prices {
            start_after: None,
            limit: None,
        },
    )
    .unwrap();
    let value: PricesResponse = from_binary(&res).unwrap();

    assert_eq!(
        value,
        PricesResponse {
            prices: vec![
                PricesResponseElem {
                    asset: "mAAPL".to_string(),
                    price: Decimal256::from_str("1.2").unwrap(),
                    last_updated_time: env.block.time,
                },
                PricesResponseElem {
                    asset: "mGOGL".to_string(),
                    price: Decimal256::from_str("2.2").unwrap(),
                    last_updated_time: env.block.time,
                }
            ],
        }
    );

    // Unauthorized try
    let info = mock_info("addr0001", &[]);
    let msg = ExecuteMsg::FeedPrice {
        prices: vec![("mAAPL".to_string(), Decimal256::from_str("1.2").unwrap())],
    };

    let res = handle(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::Unauthorized {}) => (),
        _ => panic!("Must return unauthorized error"),
    }
}
