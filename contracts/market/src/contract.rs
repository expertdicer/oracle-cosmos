#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::borrow::{
    borrow_stable, claim_rewards, compute_interest, compute_interest_raw, compute_reward,
    query_borrower_info, query_borrower_infos, repay_stable, repay_stable_from_liquidation,
};
use crate::deposit::{compute_exchange_rate_raw, deposit_stable, redeem_stable};
use crate::error::ContractError;
use crate::querier::{query_borrow_rate, query_target_deposit_rate};
use crate::state::{read_config, read_state, store_config, store_state, Config, State};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    attr, from_binary, to_binary, HumanAddr, BankMsg, Binary, CanonicalAddr, Coin, CosmosMsg, Deps,
    DepsMut, Env, MessageInfo, StdError, StdResult, Uint128, WasmMsg, InitResponse, HandleResponse, MigrateResponse
};
use cw20::{Cw20Coin, Cw20ReceiveMsg, MinterResponse};
use moneymarket::interest_model::BorrowRateResponse;
use moneymarket::market::{
    ConfigResponse, Cw20HookMsg, EpochStateResponse, ExecuteMsg, InstantiateMsg, MigrateMsg,
    QueryMsg, StateResponse,TokenInstantiateMsg, InitHook,
};
use moneymarket::querier::{deduct_tax, query_balance, query_supply};

pub const INITIAL_DEPOSIT_AMOUNT: u128 = 1000000;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn init(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<InitResponse, ContractError> {
    let initial_deposit = info
        .sent_funds
        .iter()
        .find(|c| c.denom == msg.stable_denom)
        .map(|c| c.amount)
        .unwrap_or_else(Uint128::zero);

    if initial_deposit != Uint128::from(INITIAL_DEPOSIT_AMOUNT) {
        return Err(ContractError::InitialFundsNotDeposited(
            INITIAL_DEPOSIT_AMOUNT,
            msg.stable_denom,
        ));
    }

    store_config(
        deps.storage,
        &Config {
            contract_addr: deps.api.canonical_address(&HumanAddr(env.contract.address.to_string()))?,
            owner_addr: deps.api.canonical_address(&msg.owner_addr)?,
            aterra_contract: CanonicalAddr::from(vec![]),
            overseer_contract: CanonicalAddr::from(vec![]),
            interest_model: CanonicalAddr::from(vec![]),
            distribution_model: CanonicalAddr::from(vec![]),
            collector_contract: CanonicalAddr::from(vec![]),
            distributor_contract: CanonicalAddr::from(vec![]),
            stable_denom: msg.stable_denom.clone(),
            max_borrow_factor: msg.max_borrow_factor,
        },
    )?;

    store_state(
        deps.storage,
        &State {
            total_liabilities: Decimal256::zero(),
            total_reserves: Decimal256::zero(),
            last_interest_updated: env.block.height,
            last_reward_updated: env.block.height,
            global_interest_index: Decimal256::one(),
            global_reward_index: Decimal256::zero(),
            anc_emission_rate: msg.anc_emission_rate,
            prev_aterra_supply: Uint256::zero(),
            prev_exchange_rate: Decimal256::one(),
        },
    )?;

    let res = InitResponse {
        attributes: vec![],
        messages: vec![
            CosmosMsg::Wasm(WasmMsg::Instantiate {
                code_id: msg.orchai_code_id,
                send: vec![],
                label: Some("".to_string()),
                msg: to_binary(&TokenInstantiateMsg {
                    name: format!("Orchai {}", msg.stable_denom[1..].to_uppercase()),
                    symbol: format!(
                        "o{}T",
                        msg.stable_denom[1..(msg.stable_denom.len() - 1)].to_uppercase()
                    ),
                    decimals: 6u8,
                    initial_balances: vec![Cw20Coin {
                        address: CanonicalAddr(to_binary(&env.contract.address)?),
                        amount: Uint128::from(INITIAL_DEPOSIT_AMOUNT),
                    }],
                    mint: Some(MinterResponse {
                        minter: HumanAddr(env.contract.address.to_string()),
                        cap: None,
                    }),
                    init_hook: Some(InitHook {
                        contract_addr: env.contract.address,
                        msg: to_binary(&ExecuteMsg::RegisterATerra {})?,
                    }),
                })?,
            }),
        ],
    };
    Ok(res) 
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn handle(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<HandleResponse, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::RegisterATerra{} => register_aterra(deps, env, info),
        ExecuteMsg::RegisterContracts {
            overseer_contract,
            interest_model,
            distribution_model,
            collector_contract,
            distributor_contract,
        } => {
            register_contracts(
                deps,
                overseer_contract,
                interest_model,
                distribution_model,
                collector_contract,
                distributor_contract,
            )
        }
        ExecuteMsg::UpdateConfig {
            owner_addr,
            interest_model,
            distribution_model,
            max_borrow_factor,
        } => {
            update_config(
                deps,
                env,
                info,
                owner_addr,
                interest_model,
                distribution_model,
                max_borrow_factor,
            )
        }
        ExecuteMsg::ExecuteEpochOperations {
            deposit_rate,
            target_deposit_rate,
            threshold_deposit_rate,
            distributed_interest,
        } => execute_epoch_operations(
            deps,
            env,
            info,
            deposit_rate,
            target_deposit_rate,
            threshold_deposit_rate,
            distributed_interest,
        ),
        ExecuteMsg::DepositStable {} => deposit_stable(deps, env, info),
        ExecuteMsg::BorrowStable { borrow_amount, to } => {
            borrow_stable(
                deps,
                env,
                info,
                borrow_amount,
                to,
            )
        }
        ExecuteMsg::RepayStable {} => repay_stable(deps, env, info),
        ExecuteMsg::RepayStableFromLiquidation {
            borrower,
            prev_balance,
        } => {
            let api = deps.api;
            repay_stable_from_liquidation(
                deps,
                env,
                info,
                api.human_address(&CanonicalAddr(to_binary(&borrower)?))?,
                prev_balance,
            )
        }
        ExecuteMsg::ClaimRewards { to } => {
            claim_rewards(deps, env, info, to)
        }
    }
}


pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<HandleResponse, ContractError> {
    let contract_addr = info.sender;
    match from_binary(&cw20_msg.msg.unwrap()) {
        Ok(Cw20HookMsg::RedeemStable {}) => {
            // only asset contract can execute this message
            let config: Config = read_config(deps.storage)?;
            if deps.api.canonical_address(&HumanAddr(contract_addr.to_string()))? != config.aterra_contract {
                return Err(ContractError::Unauthorized {});
            }

            redeem_stable(deps, env, cw20_msg.sender, cw20_msg.amount)
        }
        _ => Err(ContractError::MissingRedeemStableHook {}),
    }
}

pub fn register_aterra(deps: DepsMut, _env: Env, info: MessageInfo,) -> Result<HandleResponse, ContractError> {
    let mut config: Config = read_config(deps.storage)?;
    if config.aterra_contract != CanonicalAddr::from(vec![]) {
        return Err(ContractError::Unauthorized {});
    }

    config.aterra_contract = deps.api.canonical_address(&info.sender)?;
    store_config(deps.storage, &config)?;

    let res = HandleResponse {
        attributes: vec![
            attr("action", "register_aterra"),
            attr("aterra_contract", info.sender),
        ],
        messages: vec![],
        data: None,
    };
    Ok(res)
}

pub fn register_contracts(
    deps: DepsMut,
    overseer_contract: HumanAddr,
    interest_model: HumanAddr,
    distribution_model: HumanAddr,
    collector_contract: HumanAddr,
    distributor_contract: HumanAddr,
) -> Result<HandleResponse, ContractError> {
    let mut config: Config = read_config(deps.storage)?;
    if config.overseer_contract != CanonicalAddr::from(vec![])
        || config.interest_model != CanonicalAddr::from(vec![])
        || config.distribution_model != CanonicalAddr::from(vec![])
        || config.collector_contract != CanonicalAddr::from(vec![])
        || config.distributor_contract != CanonicalAddr::from(vec![])
    {
        return Err(ContractError::Unauthorized {});
    }

    config.overseer_contract = deps.api.canonical_address(&overseer_contract)?;
    config.interest_model = deps.api.canonical_address(&interest_model)?;
    config.distribution_model = deps.api.canonical_address(&distribution_model)?;
    config.collector_contract = deps.api.canonical_address(&collector_contract)?;
    config.distributor_contract = deps.api.canonical_address(&distributor_contract)?;
    store_config(deps.storage, &config)?;

    let res = HandleResponse {
        attributes: vec![
            attr("action", "register_contracts"),
            attr("overseer_contract", overseer_contract),
            attr("interest_model", interest_model),
            attr("distribution_model", distribution_model),
            attr("collector_contract", collector_contract),
            attr("distributor_contract", distributor_contract),
        ],
        messages: vec![],
        data: None,
    };
    Ok(res)
}

pub fn update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner_addr: Option<HumanAddr>,
    interest_model: Option<HumanAddr>,
    distribution_model: Option<HumanAddr>,
    max_borrow_factor: Option<Decimal256>,
) -> Result<HandleResponse, ContractError> {
    let mut config: Config = read_config(deps.storage)?;

    // permission check
    if deps.api.canonical_address(&HumanAddr(info.sender.to_string()))? != config.owner_addr {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(owner_addr) = owner_addr {
        config.owner_addr = deps.api.canonical_address(&owner_addr)?;
    }

    if interest_model.is_some() {
        let mut state: State = read_state(deps.storage)?;
        compute_interest(deps.as_ref(), &config, &mut state, env.block.height, None)?;
        store_state(deps.storage, &state)?;

        if let Some(interest_model) = interest_model {
            config.interest_model = deps.api.canonical_address(&interest_model)?;
        }
    }

    if let Some(distribution_model) = distribution_model {
        config.distribution_model = deps.api.canonical_address(&distribution_model)?;
    }

    if let Some(max_borrow_factor) = max_borrow_factor {
        config.max_borrow_factor = max_borrow_factor;
    }

    store_config(deps.storage, &config)?;
    Ok(HandleResponse {
        attributes: (vec![attr("action", "update_config")]),
        messages: vec![],
        data: None,
    })
}

pub fn execute_epoch_operations(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _deposit_rate: Decimal256,
    target_deposit_rate: Decimal256,
    _threshold_deposit_rate: Decimal256,
    distributed_interest: Uint256,
) -> Result<HandleResponse, ContractError> {
    let config: Config = read_config(deps.storage)?;
    if config.overseer_contract != deps.api.canonical_address(&HumanAddr(info.sender.to_string()))? {
        return Err(ContractError::Unauthorized {});
    }

    let mut state: State = read_state(deps.storage)?;

    // Compute interest and reward before updating anc_emission_rate
    let aterra_supply = query_supply(
        deps.as_ref(),
        deps.api.human_address(&config.aterra_contract)?,
    )?;
    let balance: Uint256 = query_balance(
        deps.as_ref(),
        deps.api.human_address(&config.contract_addr)?,
        config.stable_denom.to_string(),
    )? - distributed_interest;

    let borrow_rate_res: BorrowRateResponse = query_borrow_rate(
        deps.as_ref(),
        deps.api.human_address(&config.interest_model)?,
        balance,
        state.total_liabilities,
        state.total_reserves,
    )?;

    compute_interest_raw(
        &mut state,
        env.block.height,
        balance,
        aterra_supply,
        borrow_rate_res.rate,
        target_deposit_rate,
    );

    // recompute prev_exchange_rate with distributed_interest
    state.prev_exchange_rate =
        compute_exchange_rate_raw(&state, aterra_supply, balance + distributed_interest);

    compute_reward(&mut state, env.block.height);

    // Compute total_reserves to fund collector contract
    // Update total_reserves and send it to collector contract
    // only when there is enough balance
    let total_reserves = state.total_reserves * Uint256::one();
    let messages: Vec<CosmosMsg> = if !total_reserves.is_zero() && balance > total_reserves {
        state.total_reserves = state.total_reserves - Decimal256::from_uint256(total_reserves);

        vec![CosmosMsg::Bank(BankMsg::Send {
            from_address: env.contract.address,  // fixme
            to_address: deps
                .api
                .human_address(&config.collector_contract)?,
            amount: vec![deduct_tax(
                deps.as_ref(),
                Coin {
                    denom: config.stable_denom,
                    amount: total_reserves.into(),
                },
            )?],
        })]
    } else {
        vec![]
    };

    store_state(deps.storage, &state)?;
    let res = HandleResponse {
        attributes: vec![
            attr("action", "execute_epoch_operations"),
            attr("total_reserves", total_reserves),
            attr("anc_emission_rate", state.anc_emission_rate.to_string()),
        ],
        messages: messages,
        data: None,
    };
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::State { block_height } => to_binary(&query_state(deps, env, block_height)?),
        QueryMsg::EpochState {
            block_height,
            distributed_interest,
        } => to_binary(&query_epoch_state(
            deps,
            block_height,
            distributed_interest,
        )?),
        QueryMsg::BorrowerInfo {
            borrower,
            block_height,
        } => to_binary(&query_borrower_info(
            deps,
            env,
            deps.api.human_address(&CanonicalAddr(to_binary(&borrower)?))?,
            block_height,
        )?),
        QueryMsg::BorrowerInfos { start_after, limit } => to_binary(&query_borrower_infos(
            deps,
            start_after,
            limit,
        )?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = read_config(deps.storage)?;
    Ok(ConfigResponse {
        owner_addr: deps.api.human_address(&config.owner_addr)?.to_string(),
        aterra_contract: deps.api.human_address(&config.aterra_contract)?.to_string(),
        interest_model: deps.api.human_address(&config.interest_model)?.to_string(),
        distribution_model: deps
            .api
            .human_address(&config.distribution_model)?
            .to_string(),
        overseer_contract: deps
            .api
            .human_address(&config.overseer_contract)?
            .to_string(),
        collector_contract: deps
            .api
            .human_address(&config.collector_contract)?
            .to_string(),
        distributor_contract: deps
            .api
            .human_address(&config.distributor_contract)?
            .to_string(),
        stable_denom: config.stable_denom,
        max_borrow_factor: config.max_borrow_factor,
    })
}

pub fn query_state(deps: Deps, env: Env, block_height: Option<u64>) -> StdResult<StateResponse> {
    let mut state: State = read_state(deps.storage)?;

    let block_height = if let Some(block_height) = block_height {
        block_height
    } else {
        env.block.height
    };

    if block_height < state.last_interest_updated {
        return Err(StdError::generic_err(
            "block_height must bigger than last_interest_updated",
        ));
    }

    if block_height < state.last_reward_updated {
        return Err(StdError::generic_err(
            "block_height must bigger than last_reward_updated",
        ));
    }

    let config: Config = read_config(deps.storage)?;

    // Compute interest rate with given block height
    compute_interest(deps, &config, &mut state, block_height, None)?;

    // Compute reward rate with given block height
    compute_reward(&mut state, block_height);

    Ok(StateResponse {
        total_liabilities: state.total_liabilities,
        total_reserves: state.total_reserves,
        last_interest_updated: state.last_interest_updated,
        last_reward_updated: state.last_reward_updated,
        global_interest_index: state.global_interest_index,
        global_reward_index: state.global_reward_index,
        anc_emission_rate: state.anc_emission_rate,
        prev_aterra_supply: state.prev_aterra_supply,
        prev_exchange_rate: state.prev_exchange_rate,
    })
}

pub fn query_epoch_state(
    deps: Deps,
    block_height: Option<u64>,
    distributed_interest: Option<Uint256>,
) -> StdResult<EpochStateResponse> {
    let config: Config = read_config(deps.storage)?;
    let mut state: State = read_state(deps.storage)?;

    let distributed_interest = distributed_interest.unwrap_or_else(Uint256::zero);
    let aterra_supply = query_supply(deps, deps.api.human_address(&config.aterra_contract)?)?;
    let balance = query_balance(
        deps,
        deps.api.human_address(&config.contract_addr)?,
        config.stable_denom.to_string(),
    )? - distributed_interest;

    if let Some(block_height) = block_height {
        if block_height < state.last_interest_updated {
            return Err(StdError::generic_err(
                "block_height must bigger than last_interest_updated",
            ));
        }

        let borrow_rate_res: BorrowRateResponse = query_borrow_rate(
            deps,
            deps.api.human_address(&config.interest_model)?,
            balance,
            state.total_liabilities,
            state.total_reserves,
        )?;

        let target_deposit_rate: Decimal256 =
            query_target_deposit_rate(deps, deps.api.human_address(&config.overseer_contract)?)?;

        // Compute interest rate to return latest epoch state
        compute_interest_raw(
            &mut state,
            block_height,
            balance,
            aterra_supply,
            borrow_rate_res.rate,
            target_deposit_rate,
        );
    }

    // compute_interest_raw store current exchange rate
    // as prev_exchange_rate, so just return prev_exchange_rate
    let exchange_rate =
        compute_exchange_rate_raw(&state, aterra_supply, balance + distributed_interest);

    Ok(EpochStateResponse {
        exchange_rate,
        aterra_supply,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _info: MessageInfo, _msg: MigrateMsg) -> StdResult<MigrateResponse> {
    Ok(MigrateResponse::default())
}
