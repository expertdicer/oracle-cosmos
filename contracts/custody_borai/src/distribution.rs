use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    attr, to_binary, BalanceResponse, BankMsg, BankQuery, Coin, CosmosMsg, Deps, DepsMut, Env,
    HandleResponse, HumanAddr, MessageInfo, QueryRequest, StdResult, Uint128, WasmMsg, WasmQuery,
};

use crate::error::ContractError;
use crate::external::handle::{RewardContractExecuteMsg, RewardContractQueryMsg};
use crate::state::{read_config, BLunaAccruedRewardsResponse, Config};
use cw20::Cw20HandleMsg;
use moneymarket::custody::ExecuteMsg;
use moneymarket::querier::{deduct_tax, query_all_balances, query_balance};
use oraiswap::asset::AssetInfo;
use oraiswap::router::SwapOperation;

// REWARD_THRESHOLD
// This value is used as the minimum reward claim amount
// thus if a user's reward is less than 1 ust do not send the ClaimRewards msg
// const REWARDS_THRESHOLD: Uint128 = Uint128::from(1000000u128); fixme

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

    let reward_contract = deps.api.human_address(&config.reward_contract)?;

    // let accrued_rewards =
    //     get_accrued_rewards(deps.as_ref(), reward_contract.clone(), contract_addr)?;
    // if accrued_rewards < REWARDS_THRESHOLD {
    //     return Ok(HandleResponse::default());
    // }
    // fixme

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
pub fn distribute_hook(deps: DepsMut, env: Env) -> Result<HandleResponse, ContractError> {
    let config: Config = read_config(deps.storage)?;
    let overseer_contract = deps.api.human_address(&config.overseer_contract)?;

    // reward_amount = (prev_balance + reward_amount) - prev_balance
    // = (0 + reward_amount) - 0 = reward_amount = balance
    let reward_amount: Uint256 = query_balance(
        deps.as_ref(),
        env.contract.address.clone(),
        deps.api.human_address(&config.stable_addr)?,
    )?;
    let mut messages: Vec<CosmosMsg> = vec![]; // fixme
    if !reward_amount.is_zero() {
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.human_address(&config.stable_addr)?,
            msg: to_binary(&Cw20HandleMsg::Transfer {
                recipient: overseer_contract,
                amount: reward_amount.into(),
            })?,
            send: vec![],
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
pub fn swap_to_stable_denom(deps: DepsMut, env: Env) -> Result<HandleResponse, ContractError> {
    let config: Config = read_config(deps.storage)?;
    let swap_contract = deps.api.human_address(&config.swap_contract)?;
    let contract_addr = env.contract.address.clone();
    let balances: Vec<Coin> = query_all_balances(deps.as_ref(), contract_addr)?;
    let balance_res: BalanceResponse =
        deps.querier.query(&QueryRequest::Bank(BankQuery::Balance {
            address: env.contract.address.clone(),
            denom: "orai".to_string(),
        }))?;
    let orai_balance: Uint128 = balance_res.amount.amount;
    // let mut messages: Vec<CosmosMsg> = balances
    //     .iter()
    //     .filter(|x| x.denom != config.stable_addr.clone())
    //     .map(|coin: &Coin| { create_swap_msg(&coin, config.stable_denom.as_str(), swap_contract.clone())}
    //     )// fixme
    //     .collect::<StdResult<Vec<CosmosMsg>>>()?;

    let mut messages: Vec<CosmosMsg> = vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: swap_contract,
        msg: to_binary(&moneymarket::dex::ExecuteMsg::SwapForStable {
            recipient: env.contract.address.clone(),
        })?,
        send: vec![Coin {
            denom: "orai".to_string(),
            amount: orai_balance,
        }],
    })];

    // neu xong thi a comment doan duoi nay di nha

    let stable_ballance: Uint128 = orai_balance + orai_balance + orai_balance;
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.human_address(&config.stable_addr)?,
        send: vec![],
        msg: to_binary(&Cw20HandleMsg::Transfer {
            recipient: env.contract.address,
            amount: stable_ballance.into(),
        })?,
    }));

    let res = HandleResponse {
        attributes: vec![],
        messages: messages,
        data: None,
    };
    Ok(res)
}

pub fn create_swap_msg(
    offer_coin: &Coin,
    ask_denom: &str,
    swap_contract: HumanAddr,
) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: swap_contract,
        send: vec![], // fixme
        msg: to_binary(&SwapOperation::OraiSwap {
            offer_asset_info: AssetInfo::NativeToken {
                denom: offer_coin.denom.clone(),
            },
            ask_asset_info: AssetInfo::NativeToken {
                denom: ask_denom.to_string(),
            },
        })?,
    }))
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
