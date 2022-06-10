#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, from_binary, to_binary, Binary, Deps, DepsMut, Env, HandleResponse, HumanAddr,
    InitResponse, MessageInfo, MigrateResponse, StdResult,
};

use crate::collateral::{
    deposit_collateral, liquidate_collateral, lock_collateral, query_borrower, query_borrowers,
    unlock_collateral, withdraw_collateral,
};
use crate::distribution::{distribute_hook, distribute_rewards, swap_to_stable_denom};
use crate::error::ContractError;
use crate::state::{read_config, store_config, Config};

use cw20::Cw20ReceiveMsg;
use moneymarket::custody::{
    ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};

pub const CLAIM_REWARDS_OPERATION: u64 = 1u64;
pub const SWAP_TO_STABLE_OPERATION: u64 = 2u64;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn init(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<InitResponse> {
    let config = Config {
        owner: deps.api.canonical_address(&msg.owner)?,
        overseer_contract: deps.api.canonical_address(&msg.overseer_contract)?,
        collateral_token: deps.api.canonical_address(&msg.collateral_token)?,
        market_contract: deps.api.canonical_address(&msg.market_contract)?,
        reward_contract: deps.api.canonical_address(&msg.reward_contract)?,
        liquidation_contract: deps.api.canonical_address(&msg.liquidation_contract)?,
        swap_contract: deps.api.canonical_address(&msg.liquidation_contract)?,
        stable_addr: deps.api.canonical_address(&msg.stable_addr)?,
        basset_info: msg.basset_info,
    };

    store_config(deps.storage, &config)?;

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
        ExecuteMsg::Receive(msg) => receive_cw20(deps, info, msg),
        ExecuteMsg::UpdateConfig {
            owner,
            liquidation_contract,
            overseer_contract,
            market_contract,
            reward_contract,
        } => update_config(
            deps,
            info,
            owner,
            liquidation_contract,
            overseer_contract,
            market_contract,
            reward_contract,
        ),
        ExecuteMsg::LockCollateral { borrower, amount } => {
            lock_collateral(deps, info, borrower, amount)
        }
        ExecuteMsg::UnlockCollateral { borrower, amount } => {
            unlock_collateral(deps, info, borrower, amount)
        }
        ExecuteMsg::DistributeRewards {} => distribute_rewards(deps, env, info),
        ExecuteMsg::DistributeHook {} => distribute_hook(deps, env),
        ExecuteMsg::SwapToStableDenom {} => swap_to_stable_denom(deps, env),
        ExecuteMsg::WithdrawCollateral { amount } => withdraw_collateral(deps, info, amount),
        ExecuteMsg::LiquidateCollateral {
            liquidator,
            borrower,
            amount,
        } => liquidate_collateral(deps, info, liquidator, borrower, amount),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<HandleResponse, ContractError> {
    let contract_addr = info.sender;

    match from_binary(&cw20_msg.msg.unwrap()) {
        Ok(Cw20HookMsg::DepositCollateral {}) => {
            // only asset contract can execute this message
            let config: Config = read_config(deps.storage)?;
            if deps
                .api
                .canonical_address(&contract_addr)?
                != config.collateral_token
            {
                return Err(ContractError::Unauthorized {});
            }

            let cw20_sender_addr = cw20_msg.sender;
            deposit_collateral(deps, cw20_sender_addr, cw20_msg.amount.into())
        }
        _ => Err(ContractError::MissingDepositCollateralHook {}),
    }
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<HumanAddr>,
    liquidation_contract: Option<HumanAddr>,
    overseer_contract: Option<HumanAddr>,
    market_contract: Option<HumanAddr>,
    reward_contract: Option<HumanAddr>,
) -> Result<HandleResponse, ContractError> {
    let mut config: Config = read_config(deps.storage)?;

    if deps.api.canonical_address(&info.sender)? != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(owner) = owner {
        config.owner = deps.api.canonical_address(&owner)?;
    }

    if let Some(liquidation_contract) = liquidation_contract {
        config.liquidation_contract = deps.api.canonical_address(&liquidation_contract)?;
    }

    if let Some(overseer_contract) = overseer_contract {
        config.overseer_contract = deps.api.canonical_address(&overseer_contract)?;
    }

    if let Some(market_contract) = market_contract {
        config.market_contract = deps.api.canonical_address(&market_contract)?;
    }

    if let Some(reward_contract) = reward_contract {
        config.owner = deps.api.canonical_address(&reward_contract)?;
    }

    store_config(deps.storage, &config)?;

    let res = HandleResponse {
        attributes: vec![attr("action", "update_config")],
        messages: vec![],
        data: None,
    };
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Borrower { address } => to_binary(&query_borrower(deps, address)?),
        QueryMsg::Borrowers { start_after, limit } => {
            to_binary(&query_borrowers(deps, start_after, limit)?)
        }
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = read_config(deps.storage)?;
    Ok(ConfigResponse {
        owner: deps.api.human_address(&config.owner)?.to_string(),
        collateral_token: deps
            .api
            .human_address(&config.collateral_token)?
            .to_string(),
        overseer_contract: deps
            .api
            .human_address(&config.overseer_contract)?
            .to_string(),
        market_contract: deps.api.human_address(&config.market_contract)?.to_string(),
        reward_contract: deps.api.human_address(&config.reward_contract)?.to_string(),
        liquidation_contract: deps
            .api
            .human_address(&config.liquidation_contract)?
            .to_string(),
        stable_addr: deps.api.human_address(&config.stable_addr)?.to_string(),
        basset_info: config.basset_info,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: MigrateMsg,
) -> StdResult<MigrateResponse> {
    Ok(MigrateResponse::default())
}
