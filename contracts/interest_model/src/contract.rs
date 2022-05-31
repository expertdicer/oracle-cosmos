use crate::error::ContractError;
use crate::state::{read_config, store_config, Config};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::msgs::{BorrowRateResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use cosmwasm_bignumber::Decimal256;
use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, HandleResponse, HumanAddr, InitResponse, MessageInfo,
    StdResult,
};

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
            owner: deps.api.canonical_address(&msg.owner.unwrap())?,
            base_rate: msg.base_rate,
            interest_multiplier: msg.interest_multiplier,
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
            base_rate,
            interest_multiplier,
        } => update_config(deps, info, owner, base_rate, interest_multiplier),
    }
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<HumanAddr>,
    base_rate: Option<Decimal256>,
    interest_multiplier: Option<Decimal256>,
) -> Result<HandleResponse, ContractError> {
    let mut config: Config = read_config(deps.storage)?;
    if deps
        .api
        .canonical_address(&HumanAddr(info.sender.to_string()))?
        != config.owner
    {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(owner) = owner {
        config.owner = deps.api.canonical_address(&owner)?;
    }

    if let Some(base_rate) = base_rate {
        config.base_rate = base_rate;
    }

    if let Some(interest_multiplier) = interest_multiplier {
        config.interest_multiplier = interest_multiplier;
    }

    store_config(deps.storage, &config)?;
    Ok(HandleResponse::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::BorrowRate {
            market_balance,
            total_liabilities,
            total_reserves,
        } => to_binary(&query_borrow_rate(
            deps,
            market_balance,
            total_liabilities,
            total_reserves,
        )?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state = read_config(deps.storage)?;
    let resp = ConfigResponse {
        owner: deps.api.human_address(&state.owner)?.to_string(),
        base_rate: state.base_rate,
        interest_multiplier: state.interest_multiplier,
    };

    Ok(resp)
}

fn query_borrow_rate(
    deps: Deps,
    market_balance: Uint256,
    total_liabilities: Decimal256,
    total_reserves: Decimal256,
) -> StdResult<BorrowRateResponse> {
    let config: Config = read_config(deps.storage)?;

    // ignore decimal parts
    let total_value_in_market =
        Decimal256::from_uint256(market_balance) + total_liabilities - total_reserves;

    let utilization_ratio = if total_value_in_market.is_zero() {
        Decimal256::zero()
    } else {
        total_liabilities / total_value_in_market
    };

    Ok(BorrowRateResponse {
        rate: utilization_ratio * config.interest_multiplier + config.base_rate,
    })
}
