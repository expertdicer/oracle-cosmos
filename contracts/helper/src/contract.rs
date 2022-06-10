use crate::error::ContractError;
use crate::state::{read_config, store_config, Config};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::external::query::{EpochStateResponse, QueryEpochState};
use crate::msgs::{ConfigResponse, DepositRateResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use cosmwasm_bignumber::Decimal256;
use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, HandleResponse, HumanAddr, InitResponse, MessageInfo,
    QueryRequest, StdResult, WasmQuery,
};
use cw20::Cw20QueryMsg;

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
            market_contract: msg.market_contract,
            overseer_contract: msg.overseer_contract,
            collateral_contract: msg.collateral_contract,
            custody_borai_contract: msg.custody_borai_contract,
            interest_contract: msg.interest_contract,
            orchai_contract: msg.orchai_contract,
            stable_addr: msg.stable_addr,
            staking_contract: msg.staking_contract,
            denom_token: msg.denom_token,
            aterra_contract: msg.aterra_contract,
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
            market_contract,
            overseer_contract,
            collateral_contract,
            custody_borai_contract,
            interest_contract,
            orchai_contract,
            stable_addr,
            staking_contract,
            denom_token,
            aterra_contract,
        } => update_config(
            deps,
            _env,
            info,
            market_contract,
            overseer_contract,
            collateral_contract,
            custody_borai_contract,
            interest_contract,
            orchai_contract,
            stable_addr,
            staking_contract,
            denom_token,
            aterra_contract,
        ),
    }
}

pub fn update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    market_contract: Option<HumanAddr>,
    overseer_contract: Option<HumanAddr>,
    collateral_contract: Option<HumanAddr>,
    custody_borai_contract: Option<HumanAddr>,
    interest_contract: Option<HumanAddr>,
    orchai_contract: Option<HumanAddr>,
    stable_addr: Option<HumanAddr>,
    staking_contract: Option<HumanAddr>,
    denom_token: Option<String>,
    aterra_contract: Option<HumanAddr>,
) -> Result<HandleResponse, ContractError> {
    let mut config: Config = read_config(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(market_contract) = market_contract {
        config.market_contract = market_contract;
    }
    if let Some(overseer_contract) = overseer_contract {
        config.overseer_contract = overseer_contract;
    }
    if let Some(collateral_contract) = collateral_contract {
        config.collateral_contract = collateral_contract;
    }
    if let Some(custody_borai_contract) = custody_borai_contract {
        config.custody_borai_contract = custody_borai_contract;
    }
    if let Some(interest_contract) = interest_contract {
        config.interest_contract = interest_contract;
    }
    if let Some(orchai_contract) = orchai_contract {
        config.orchai_contract = orchai_contract;
    }
    if let Some(stable_addr) = stable_addr {
        config.stable_addr = stable_addr;
    }
    if let Some(staking_contract) = staking_contract {
        config.staking_contract = staking_contract;
    }
    if let Some(denom_token) = denom_token {
        config.denom_token = denom_token;
    }
    if let Some(aterra_contract) = aterra_contract {
        config.aterra_contract = aterra_contract;
    }

    store_config(deps.storage, &config)?;
    Ok(HandleResponse::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::DepositRate {} => to_binary(&query_deposit_rate(deps, _env)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = read_config(deps.storage)?;
    let resp = ConfigResponse {
        owner: config.owner,
        market_contract: config.market_contract,
        overseer_contract: config.overseer_contract,
        collateral_contract: config.collateral_contract,
        custody_borai_contract: config.custody_borai_contract,
        interest_contract: config.interest_contract,
        orchai_contract: config.orchai_contract,
        stable_addr: config.stable_addr,
        staking_contract: config.staking_contract,
        denom_token: config.denom_token,
        aterra_contract: config.aterra_contract,
    };

    Ok(resp)
}

fn query_deposit_rate(deps: Deps, env: Env) -> StdResult<DepositRateResponse> {
    let config = read_config(deps.storage)?;
    let epochstate: EpochStateResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.market_contract,
            msg: to_binary(&QueryEpochState {
                block_height: env.block.height,
                distributed_interest: Uint256::zero(),
            })?,
        }))?;

    Ok(DepositRateResponse {
        deposit_rate: epochstate.exchange_rate,
    })
}
