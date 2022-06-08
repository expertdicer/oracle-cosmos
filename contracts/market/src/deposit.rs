use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    attr, to_binary, HumanAddr, BankMsg, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    StdResult, Uint128, WasmMsg, HandleResponse,
};

use crate::borrow::{compute_interest, compute_reward};
use crate::error::ContractError;
use crate::state::{read_config, read_state, store_state, Config, State};
use moneymarket::querier::{deduct_tax, query_balance, query_supply};

use cw20::Cw20HandleMsg;

pub fn deposit_stable(
    deps: DepsMut,
    env: Env,
    sender: HumanAddr,
    amount: Uint128,
) -> Result<HandleResponse, ContractError> {
    let config: Config = read_config(deps.storage)?;

    // Check base denom deposit
    let deposit_amount: Uint256 = Uint256::from(amount);

    // Cannot deposit zero amount
    if deposit_amount.is_zero() {
        return Err(ContractError::ZeroDeposit {});
    }

    // Update interest related state
    let mut state: State = read_state(deps.storage)?;
    compute_interest(
        deps.as_ref(),
        &config,
        &mut state,
        env.block.height,
        Some(deposit_amount),
    )?;
    compute_reward(&mut state, env.block.height);

    // Load anchor token exchange rate with updated state
    let exchange_rate =
        compute_exchange_rate(deps.as_ref(), &config, &state, Some(deposit_amount))?;
    let mint_amount = deposit_amount / exchange_rate;

    state.prev_aterra_supply += mint_amount;
    store_state(deps.storage, &state)?;
    let res = HandleResponse {
        attributes: vec![
            attr("action", "deposit_stable"),
            attr("depositor", sender.to_string()),
            attr("mint_amount", mint_amount),
            attr("deposit_amount", deposit_amount),
        ],
        messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.human_address(&config.aterra_contract)?,
            send: vec![],
            msg: to_binary(&Cw20HandleMsg::Mint {
                recipient: sender,
                amount: mint_amount.into(),
            })?,
        }),
        ],
        data: None,
    };
    Ok(res)
}

pub fn redeem_stable(
    deps: DepsMut,
    env: Env,
    sender: HumanAddr,
    burn_amount: Uint128,
) -> Result<HandleResponse, ContractError> {
    let config: Config = read_config(deps.storage)?;

    // Update interest related state
    let mut state: State = read_state(deps.storage)?;
    compute_interest(deps.as_ref(), &config, &mut state, env.block.height, None)?;
    compute_reward(&mut state, env.block.height);

    // Load anchor token exchange rate with updated state
    let exchange_rate = compute_exchange_rate(deps.as_ref(), &config, &state, None)?;
    let redeem_amount = Uint256::from(burn_amount) * exchange_rate;

    let query_target = HumanAddr(env.contract.address.to_string());
    let current_balance = query_balance(
        deps.as_ref(),
        query_target,
        HumanAddr(config.stable_addr.to_string()),
    )?;

    // Assert redeem amount
    assert_redeem_amount(&config, &state, current_balance, redeem_amount)?;

    state.prev_aterra_supply = state.prev_aterra_supply - Uint256::from(burn_amount);
    store_state(deps.storage, &state)?;
    let res = HandleResponse {
        attributes: vec![
            attr("action", "redeem_stable"),
            attr("burn_amount", burn_amount),
            attr("redeem_amount", redeem_amount),
        ],
        messages: vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.human_address(&config.aterra_contract)?,
                send: vec![],
                msg: to_binary(&Cw20HandleMsg::Burn {
                    amount: burn_amount,
                })?,
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: HumanAddr(config.stable_addr.to_string()),
                msg: to_binary(&Cw20HandleMsg::Transfer {
                    recipient: sender,
                    amount: redeem_amount.into(),
                })?,
                send: vec![],
            }),
        ],
        data: None,
    };
    Ok(res)
}

fn assert_redeem_amount(
    config: &Config,
    state: &State,
    current_balance: Uint256,
    redeem_amount: Uint256,
) -> Result<HandleResponse, ContractError> {
    let current_balance = Decimal256::from_uint256(current_balance);
    let redeem_amount = Decimal256::from_uint256(redeem_amount);
    if redeem_amount + state.total_reserves > current_balance {
        return Err(ContractError::NoStableAvailable{});
    }

    Ok(HandleResponse::default())
}

pub(crate) fn compute_exchange_rate(
    deps: Deps,
    config: &Config,
    state: &State,
    deposit_amount: Option<Uint256>,
) -> StdResult<Decimal256> {
    let aterra_supply = query_supply(deps, deps.api.human_address(&config.aterra_contract)?)?;
    let balance = query_balance(
        deps,
        deps.api.human_address(&config.contract_addr)?,
        HumanAddr(config.stable_addr.to_string()),
    )? - deposit_amount.unwrap_or_else(Uint256::zero);

    Ok(compute_exchange_rate_raw(state, aterra_supply, balance))
}

pub fn compute_exchange_rate_raw(
    state: &State,
    aterra_supply: Uint256,
    contract_balance: Uint256,
) -> Decimal256 {
    if aterra_supply.is_zero() {
        return Decimal256::one();
    }

    // (aterra / stable_denom)
    // exchange_rate = (balance + total_liabilities - total_reserves) / aterra_supply
    (Decimal256::from_uint256(contract_balance) + state.total_liabilities - state.total_reserves)
        / Decimal256::from_uint256(aterra_supply)
}
