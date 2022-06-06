use crate::error::ContractError;
use crate::state::{read_config, store_config, Config};
use cosmwasm_bignumber::{Decimal256, Uint256};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    attr, to_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, HandleResponse,
    HumanAddr, InitResponse, MessageInfo, StakingMsg, StdResult, WasmMsg,
};

use crate::msgs::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cw20::Cw20HandleMsg;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn init(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<InitResponse, ContractError> {
    store_config(
        deps.storage,
        &Config {
            owner: msg.owner,
            native_token_denom: msg.native_token_denom,
            native_token: msg.native_token,
            asset_token: msg.asset_token,
            base_apr: msg.base_apr,
            orchai_token: msg.orchai_token,
        },
    )?;

    Ok(InitResponse::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn handle(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<HandleResponse, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig {
            owner,
            base_apr,
            asset_token,
        } => update_config(deps, _env, info, owner, base_apr, asset_token),
        ExecuteMsg::StakingOrai { amount } => staking_orai(deps, _env, info, amount),
    }
}

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    owner: Option<HumanAddr>,
    base_apr: Option<Uint256>,
    asset_token: Option<HumanAddr>,
) -> Result<HandleResponse, ContractError> {
    let mut config: Config = read_config(deps.storage)?;
    if HumanAddr(_info.sender.to_string()) != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(owner) = owner {
        config.owner = owner;
    }

    if let Some(base_apr) = base_apr {
        config.base_apr = base_apr;
    }

    if let Some(asset_token) = asset_token {
        config.asset_token = asset_token;
    }

    store_config(deps.storage, &config)?;
    Ok(HandleResponse::default())
}

pub fn staking_orai(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    amount: Uint256,
) -> Result<HandleResponse, ContractError> {
    let config: Config = read_config(deps.storage)?;
    // user send orai to contract

    // mint orai for user
    let mut messages: Vec<CosmosMsg> = vec![];
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.asset_token,
        send: vec![],
        msg: to_binary(&Cw20HandleMsg::Mint {
            recipient: _info.sender,
            amount: amount.clone().into(),
        })?,
    }));

    let res = HandleResponse {
        attributes: vec![attr("action", "staking_orai"), attr("amount", amount)],
        messages: messages,
        data: None,
    };
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    Ok(Binary::default())
}
