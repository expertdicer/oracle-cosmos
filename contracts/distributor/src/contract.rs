#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::state::{read_config, store_config, Config};

use cosmwasm_std::{
    attr, to_binary, Binary, CanonicalAddr, CosmosMsg, Deps, DepsMut, Env, MessageInfo, InitResponse,
    HandleResponse, MigrateResponse, StdError, StdResult, Uint128, WasmMsg, HumanAddr,
};

use anchor_token::distributor::{ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

use cw20::Cw20HandleMsg;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn init(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<InitResponse> {
    let whitelist = msg
        .whitelist
        .into_iter()
        .map(|w| deps.api.canonical_address(&w))
        .collect::<StdResult<Vec<CanonicalAddr>>>()?;

    store_config(
        deps.storage,
        &Config {
            gov_contract: deps.api.canonical_address(&msg.gov_contract)?,
            anchor_token: deps.api.canonical_address(&msg.anchor_token)?,
            whitelist,
            spend_limit: msg.spend_limit,
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
) -> StdResult<HandleResponse> {
    match msg {
        ExecuteMsg::UpdateConfig { spend_limit } => update_config(deps, info, spend_limit),
        ExecuteMsg::Spend { recipient, amount } => spend(deps, info, recipient, amount),
        ExecuteMsg::AddDistributor { distributor } => add_distributor(deps, info, distributor),
        ExecuteMsg::RemoveDistributor { distributor } => {
            remove_distributor(deps, info, distributor)
        }
    }
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    spend_limit: Option<Uint128>,
) -> StdResult<HandleResponse> {
    let mut config: Config = read_config(deps.storage)?;
    if config.gov_contract != deps.api.canonical_address(&info.sender)? {
        return Err(StdError::generic_err("unauthorized"));
    }

    if let Some(spend_limit) = spend_limit {
        config.spend_limit = spend_limit;
    }

    store_config(deps.storage, &config)?;

    Ok( HandleResponse {
        attributes: vec![attr("action", "update_config")],
        messages: vec![],
        data: None,
    })
}

pub fn add_distributor(
    deps: DepsMut,
    info: MessageInfo,
    distributor: HumanAddr,
) -> StdResult<HandleResponse> {
    let mut config: Config = read_config(deps.storage)?;
    if config.gov_contract != deps.api.canonical_address(&info.sender)? {
        return Err(StdError::generic_err("unauthorized"));
    }

    let distributor_raw = deps.api.canonical_address(&distributor)?;
    if config
        .whitelist
        .clone()
        .into_iter()
        .any(|w| w == distributor_raw)
    {
        return Err(StdError::generic_err("Distributor already registered"));
    }

    config.whitelist.push(distributor_raw);
    store_config(deps.storage, &config)?;

    let res = HandleResponse {
        attributes: vec![
            attr("action", "add_distributor"),
            attr("distributor", distributor.as_str()),
        ],
        messages: vec![],
        data: None,
    };
    Ok(res)
}

pub fn remove_distributor(
    deps: DepsMut,
    info: MessageInfo,
    distributor: HumanAddr,
) -> StdResult<HandleResponse> {
    let mut config: Config = read_config(deps.storage)?;
    if config.gov_contract != deps.api.canonical_address(&info.sender)? {
        return Err(StdError::generic_err("unauthorized"));
    }

    let distributor_raw = deps.api.canonical_address(&distributor)?;
    let whitelist_len = config.whitelist.len();
    let whitelist: Vec<CanonicalAddr> = config
        .whitelist
        .into_iter()
        .filter(|w| *w != distributor_raw)
        .collect();

    if whitelist_len == whitelist.len() {
        return Err(StdError::generic_err("Distributor not found"));
    }

    config.whitelist = whitelist;
    store_config(deps.storage, &config)?;

    let res = HandleResponse {
        attributes: vec![
            attr("action", "remove_distributor"),
            attr("distributor", distributor.as_str()),
        ],
        messages: vec![],
        data: None,
    };
    Ok(res)
}

/// Spend
/// Owner can execute spend operation to send
/// `amount` of MIR token to `recipient` for community purpose
pub fn spend(
    deps: DepsMut,
    info: MessageInfo,
    recipient: HumanAddr,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    let config: Config = read_config(deps.storage)?;
    let sender_raw = deps.api.canonical_address(&info.sender)?;

    if !config.whitelist.into_iter().any(|w| w == sender_raw) {
        return Err(StdError::generic_err("unauthorized"));
    }

    if config.spend_limit < amount {
        return Err(StdError::generic_err("Cannot spend more than spend_limit"));
    }

    let anchor_token = deps.api.human_address(&config.anchor_token)?;

    let res = HandleResponse {
        attributes: vec![
            attr("action", "spend"),
            attr("recipient", recipient.as_str()),
            attr("amount", amount.to_string().as_str()),
        ],
        messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: anchor_token,
            send: vec![],
            msg: to_binary(&Cw20HandleMsg::Transfer {
                recipient: recipient.clone(),
                amount,
            })?,
        })],
        data: None,
    };
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state = read_config(deps.storage)?;
    let resp = ConfigResponse {
        gov_contract: deps.api.human_address(&state.gov_contract)?.to_string(),
        anchor_token: deps.api.human_address(&state.anchor_token)?.to_string(),
        whitelist: state
            .whitelist
            .into_iter()
            .map(|w| match deps.api.human_address(&w) {
                Ok(addr) => Ok(addr.to_string()),
                Err(e) => Err(e),
            })
            .collect::<StdResult<Vec<String>>>()?,
        spend_limit: state.spend_limit,
    };

    Ok(resp)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _info: MessageInfo, _msg: MigrateMsg) -> StdResult<MigrateResponse> {
    Ok(MigrateResponse::default())
}
