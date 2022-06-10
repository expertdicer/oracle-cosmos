use crate::error::ContractError;
use crate::state::{read_config, store_config, Config};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::external::query::{
    BalanceResponse, BorrowerResponse, CustodyExternalMsg, EpochStateResponse,
    MarketEpochStateResponse, MarketExternalMsg, OverseerExternalMsg,
};
use crate::msgs::{
    BorrowerInfoResponse, CollateralBallanceResponse, ConfigResponse, DepositRateResponse,
    ExecuteMsg, InstantiateMsg, OraiBalanceResponse, QueryMsg, TotalBallanceDepositResponse,
};
use cosmwasm_bignumber::Decimal256;
use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    to_binary, BankQuery, Binary, Deps, DepsMut, Env, HandleResponse, HumanAddr, InitResponse,
    MessageInfo, QueryRequest, StdResult, WasmQuery,
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
        QueryMsg::TotalBallanceDeposit { user } => {
            to_binary(&query_total_ballance_deposit(deps, _env, user)?)
        }
        QueryMsg::CollateralBalance { user } => {
            to_binary(&query_collateral_ballance(deps, _env, user)?)
        }
        QueryMsg::BorrowerInfo { borrower } => {
            to_binary(&query_borrower_info(deps, _env, borrower)?)
        }
        QueryMsg::OraiBallance { user } => to_binary(&query_orai_ballance(deps, _env, user)?),
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
            contract_addr: config.overseer_contract,
            msg: to_binary(&OverseerExternalMsg::EpochState {})?,
        }))?;

    Ok(DepositRateResponse {
        deposit_rate: epochstate.deposit_rate,
    })
}

fn query_total_ballance_deposit(
    deps: Deps,
    env: Env,
    user: HumanAddr,
) -> StdResult<TotalBallanceDepositResponse> {
    let config = read_config(deps.storage)?;
    let ausdt_ballance: BalanceResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.aterra_contract,
            msg: to_binary(&Cw20QueryMsg::Balance {
                address: user.clone(),
            })?,
        }))?;

    let epochstate: MarketEpochStateResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.overseer_contract,
            msg: to_binary(&MarketExternalMsg::EpochState {
                block_height: env.block.height,
                distributed_interest: Uint256::zero(),
            })?,
        }))?;

    let mut total_ballance: Uint256 = ausdt_ballance.balance.into();
    total_ballance = total_ballance * epochstate.exchange_rate;
    Ok(TotalBallanceDepositResponse {
        total_ballance: total_ballance,
        ausdt_ballance: ausdt_ballance.balance.into(),
        exchange_rate: epochstate.exchange_rate,
    })
}

pub fn query_collateral_ballance(
    deps: Deps,
    env: Env,
    user: HumanAddr,
) -> StdResult<CollateralBallanceResponse> {
    let config = read_config(deps.storage)?;
    let collateral_ballance: BorrowerResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.custody_borai_contract,
            msg: to_binary(&CustodyExternalMsg::Borrower {
                address: user.to_string(),
            })?,
        }))?;

    Ok(CollateralBallanceResponse {
        borrower: collateral_ballance.borrower,
        balance: collateral_ballance.balance,
        spendable: collateral_ballance.spendable,
    })
}

pub fn query_borrower_info(
    deps: Deps,
    env: Env,
    borrower: HumanAddr,
) -> StdResult<BorrowerInfoResponse> {
    let config = read_config(deps.storage)?;
    let borrower_info: BorrowerInfoResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.market_contract,
            msg: to_binary(&MarketExternalMsg::BorrowerInfo {
                borrower: borrower,
                block_height: Some(env.block.height),
            })?,
        }))?;

    Ok(BorrowerInfoResponse {
        borrower: borrower_info.borrower.to_string(),
        interest_index: borrower_info.interest_index,
        reward_index: borrower_info.reward_index,
        loan_amount: borrower_info.loan_amount,
        pending_rewards: borrower_info.pending_rewards,
    })
}

pub fn query_orai_ballance(deps: Deps, env: Env, user: HumanAddr) -> StdResult<Uint256> {
    let config = read_config(deps.storage)?;
    let balance: OraiBalanceResponse =
        deps.querier.query(&QueryRequest::Bank(BankQuery::Balance {
            address: user,
            denom: config.denom_token,
        }))?;
    Ok(balance.amount.amount.into())
}
