use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    attr, to_binary, HumanAddr, BankMsg, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, QueryRequest,
    HandleResponse, StdResult, Uint128, WasmMsg, WasmQuery,
};

use crate::error::ContractError;
use crate::external::handle::{RewardContractExecuteMsg, RewardContractQueryMsg};
use crate::state::{read_config, BLunaAccruedRewardsResponse, Config};
use moneymarket::querier::{query_all_balances, deduct_tax, query_balance};
use moneymarket::custody::ExecuteMsg;

// REWARD_THRESHOLD
// This value is used as the minimum reward claim amount
// thus if a user's reward is less than 1 ust do not send the ClaimRewards msg
const REWARDS_THRESHOLD: Uint128 = Uint128::from(1000000u128);

/// Request withdraw reward operation to
/// reward contract and execute `distribute_hook`
/// Executor: overseer
pub fn distribute_rewards(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<HandleResponse, ContractError> {
    let config: Config = read_config(deps.storage)?;
    if config.overseer_contract != deps.api.canonical_address(&info.sender)? {
        return Err(ContractError::Unauthorized {});
    }

    let contract_addr = env.contract.address;
    let reward_contract = deps.api.human_address(&config.reward_contract)?;

    let accrued_rewards =
        get_accrued_rewards(deps.as_ref(), reward_contract.clone(), contract_addr)?;
    if accrued_rewards < REWARDS_THRESHOLD {
        return Ok(HandleResponse::default());
    }

    let contract_addr = env.contract.address;
    
    let res = HandleResponse {
        attributes: vec![],
        messages: vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: reward_contract,
                send: vec![],
                msg: to_binary(&RewardContractExecuteMsg::ClaimRewards { recipient: None })?,
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.clone(),
                send: vec![],
                msg: to_binary(&ExecuteMsg::SwapToStableDenom {})?,
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr,
                send: vec![],
                msg: to_binary(&ExecuteMsg::DistributeHook {})?,
            }),
        ],
        data: None,
    };
    // Do not emit the event logs here
    Ok(res)
}

/// Apply swapped reward to global index
/// Executor: itself
pub fn distribute_hook(
    deps: DepsMut,
    env: Env,
) -> Result<HandleResponse, ContractError> {
    let contract_addr = env.contract.address;
    let config: Config = read_config(deps.storage)?;
    let overseer_contract = deps.api.human_address(&config.overseer_contract)?;

    // reward_amount = (prev_balance + reward_amount) - prev_balance
    // = (0 + reward_amount) - 0 = reward_amount = balance
    let reward_amount: Uint256 = query_balance(
        deps.as_ref(),
        contract_addr,
        config.stable_denom.to_string(),
    )?;
    let mut messages: Vec<CosmosMsg<TerraMsgWrapper>> = vec![];      // fixme
    if !reward_amount.is_zero() {
        messages.push(CosmosMsg::Bank(BankMsg::Send {
            from_address: env.contract.address , // fixme            
            to_address: overseer_contract,
            amount: vec![deduct_tax(
                deps.as_ref(),
                Coin {
                    denom: config.stable_denom,
                    amount: reward_amount.into(),
                },
            )?],
        }));
    }

    let res = HandleResponse {
        attributes: vec![
            attr("action", "distribute_rewards"),
            attr("buffer_rewards", reward_amount),
        ],
        messages: messages,
        data: None,
    };
    Ok(res)
}

/// Swap all coins to stable_denom
/// and execute `swap_hook`
/// Executor: itself
pub fn swap_to_stable_denom(
    deps: DepsMut,
    env: Env,
) -> Result<Response<TerraMsgWrapper>, ContractError> {         
    let config: Config = read_config(deps.storage)?;

    let contract_addr = env.contract.address.clone();
    let balances: Vec<Coin> = query_all_balances(deps.as_ref(), contract_addr)?;
    let mut messages: Vec<CosmosMsg<TerraMsgWrapper>> = balances
        .iter()
        .filter(|x| x.denom != config.stable_denom)
        .map(|coin: &Coin| SubMsg::new(create_swap_msg(coin.clone(), config.stable_denom.clone()))) // fixme
        .collect();

    let res = HandleResponse {
        attributes: vec![],
        messages: messages,
        data: None,
    };
    Ok(res)
}

pub(crate) fn get_accrued_rewards(
    deps: Deps,
    reward_contract_addr: HumanAddr,
    contract_addr: HumanAddr,
) -> StdResult<Uint128> {
    let rewards: BLunaAccruedRewardsResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: reward_contract_addr,
            msg: to_binary(&RewardContractQueryMsg::AccruedRewards {
                address: contract_addr.to_string(),
            })?,
        }))?;

    Ok(rewards.rewards)
}
