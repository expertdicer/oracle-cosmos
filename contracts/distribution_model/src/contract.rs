#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, HumanAddr, Binary, Deps, DepsMut, Env, MessageInfo, InitResponse, HandleResponse, StdResult};

use crate::error::ContractError;
use crate::state::{read_config, store_config, Config};

use cosmwasm_bignumber::Decimal256;
use moneymarket::distribution_model::{
    AncEmissionRateResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn init(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<InitResponse> {
    store_config(
        deps.storage,
        &Config {
            owner: deps.api.canonical_address(&msg.owner)?,
            emission_cap: msg.emission_cap,
            emission_floor: msg.emission_floor,
            increment_multiplier: msg.increment_multiplier,
            decrement_multiplier: msg.decrement_multiplier,
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
            emission_cap,
            emission_floor,
            increment_multiplier,
            decrement_multiplier,
        } => {
            update_config(
                deps,
                info,
                owner,
                emission_cap,
                emission_floor,
                increment_multiplier,
                decrement_multiplier,
            )
        }
    }
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<HumanAddr>,
    emission_cap: Option<Decimal256>,
    emission_floor: Option<Decimal256>,
    increment_multiplier: Option<Decimal256>,
    decrement_multiplier: Option<Decimal256>,
) -> Result<HandleResponse, ContractError> {
    let mut config: Config = read_config(deps.storage)?;
    if deps.api.canonical_address(&HumanAddr(info.sender.to_string()))? != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(owner) = owner {
        config.owner = deps.api.canonical_address(&owner)?;
    }

    if let Some(emission_cap) = emission_cap {
        config.emission_cap = emission_cap;
    }

    if let Some(emission_floor) = emission_floor {
        config.emission_floor = emission_floor
    }

    if let Some(increment_multiplier) = increment_multiplier {
        config.increment_multiplier = increment_multiplier;
    }

    if let Some(decrement_multiplier) = decrement_multiplier {
        config.decrement_multiplier = decrement_multiplier;
    }

    store_config(deps.storage, &config)?;
    Ok(HandleResponse::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::AncEmissionRate {
            deposit_rate,
            target_deposit_rate,
            threshold_deposit_rate,
            current_emission_rate,
        } => to_binary(&query_anc_emission_rate(
            deps,
            deposit_rate,
            target_deposit_rate,
            threshold_deposit_rate,
            current_emission_rate,
        )?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state = read_config(deps.storage)?;
    let resp = ConfigResponse {
        owner: deps.api.human_address(&state.owner)?.to_string(),
        emission_cap: state.emission_cap,
        emission_floor: state.emission_floor,
        increment_multiplier: state.increment_multiplier,
        decrement_multiplier: state.decrement_multiplier,
    };

    Ok(resp)
}

fn query_anc_emission_rate(
    deps: Deps,
    deposit_rate: Decimal256,
    target_deposit_rate: Decimal256,
    threshold_deposit_rate: Decimal256,
    current_emission_rate: Decimal256,
) -> StdResult<AncEmissionRateResponse> {
    let config: Config = read_config(deps.storage)?;

    let half_dec = Decimal256::one() + Decimal256::one();
    let mid_rate = (threshold_deposit_rate + target_deposit_rate) / half_dec;
    let high_trigger = (mid_rate + target_deposit_rate) / half_dec;
    let low_trigger = (mid_rate + threshold_deposit_rate) / half_dec;

    let emission_rate = if deposit_rate < low_trigger {
        current_emission_rate * config.increment_multiplier
    } else if deposit_rate > high_trigger {
        current_emission_rate * config.decrement_multiplier
    } else {
        current_emission_rate
    };

    let emission_rate = if emission_rate > config.emission_cap {
        config.emission_cap
    } else if emission_rate < config.emission_floor {
        config.emission_floor
    } else {
        emission_rate
    };

    Ok(AncEmissionRateResponse { emission_rate })
}
