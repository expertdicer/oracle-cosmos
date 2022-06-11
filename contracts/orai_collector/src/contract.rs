use cosmwasm_std::{
    attr,entry_point, CosmosMsg, DepsMut, Env, HandleResponse, InitResponse, MessageInfo,
    StdResult, BankMsg, QueryRequest, BankQuery, BalanceResponse, HumanAddr, Coin, Deps, Binary,
};
use crate::error::ContractError;
use crate::msgs::{InstantiateMsg, ExecuteMsg , QueryMsg};
// version info for migration info

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn init(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<InitResponse> {
    Ok(InitResponse::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn handle(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<HandleResponse, ContractError> {
    match msg {
        ExecuteMsg::Release {} => release(
            deps,
            env,
            info,
        ),
    }
}

pub fn release(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
) -> Result<HandleResponse, ContractError> {
    let balance: BalanceResponse = deps.querier.query(
        &QueryRequest::Bank(BankQuery::Balance {
            address: env.contract.address.clone(),
            denom: "orai".to_string(),
        })
    ).unwrap();

    let messages = vec![CosmosMsg::Bank(BankMsg::Send {
        from_address: env.contract.address,
        to_address: HumanAddr("orai18uzz3c2fd4an5xj8785mwwn80d47af9axkaqz8".to_string()),
        amount:  vec![Coin{
            denom: "orai".to_string(),
            amount: balance.amount.amount.into(),
        }]
    })];

    let res = HandleResponse {
        attributes: vec![
            attr("action", "release orai"),
            attr("to", HumanAddr("orai18uzz3c2fd4an5xj8785mwwn80d47af9axkaqz8".to_string())),
            attr("amount", balance.amount.amount),
        ],
        messages: messages,
        data: None,
    };
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    Ok(Binary::from(&[1u8]))
}