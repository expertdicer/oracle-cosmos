#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    attr, to_binary, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, HandleResponse,
    HumanAddr, InitResponse, MessageInfo, MigrateResponse, StdError, StdResult, WasmMsg,
};

use crate::state::{read_config, store_config, Config};

use crate::migration::migrate_config;
use crate::queurier::{query_balance, query_token_balance};
use anchor_token::collector::{ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use cosmwasm_std::Uint128;
use cw20::Cw20HandleMsg;
use oraiswap::asset::{Asset, AssetInfo, PairInfo};
use oraiswap::oracle::OracleContract;
use oraiswap::pair::HandleMsg as AstroportExecuteMsg;
use oraiswap::querier::query_pair_info;

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
            gov_contract: deps.api.canonical_address(&msg.gov_contract)?,
            astroport_factory: deps.api.canonical_address(&msg.astroport_factory)?,
            anchor_token: deps.api.canonical_address(&msg.anchor_token)?,
            oraiswap_oracle: deps.api.canonical_address(&msg.oraiswap_oracle)?,
            reward_factor: msg.reward_factor,
            max_spread: msg.max_spread,
        },
    )?;

    Ok(InitResponse::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn handle(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<HandleResponse> {
    match msg {
        ExecuteMsg::UpdateConfig {
            reward_factor,
            gov_contract,
            astroport_factory,
            oraiswap_oracle,
            max_spread,
        } => update_config(
            deps,
            info,
            reward_factor,
            gov_contract,
            astroport_factory,
            oraiswap_oracle,
            max_spread,
        ),
        ExecuteMsg::Sweep { denom } => sweep(deps, env, denom),
        ExecuteMsg::Distribute {} => distribute(deps, env),
    }
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    reward_factor: Option<Decimal>,
    gov_contract: Option<HumanAddr>,
    astroport_factory: Option<HumanAddr>,
    oraiswap_oracle: Option<HumanAddr>,
    max_spread: (bool, Option<Decimal>),
) -> StdResult<HandleResponse> {
    let mut config: Config = read_config(deps.storage)?;
    if deps.api.canonical_address(&info.sender)? != config.gov_contract {
        return Err(StdError::generic_err("unauthorized"));
    }

    if let Some(reward_factor) = reward_factor {
        config.reward_factor = reward_factor;
    }

    if let Some(gov_contract) = gov_contract {
        config.gov_contract = deps.api.canonical_address(&gov_contract)?;
    }
    if let Some(astroport_factory) = astroport_factory {
        config.astroport_factory = deps.api.canonical_address(&astroport_factory)?;
    }

    if let Some(oraiswap_oracle) = oraiswap_oracle {
        config.oraiswap_oracle = deps.api.canonical_address(&oraiswap_oracle)?;
    }

    if max_spread.0 {
        config.max_spread = max_spread.1
    }

    store_config(deps.storage, &config)?;
    Ok(HandleResponse::default())
}

/// Sweep
/// Anyone can execute sweep function to swap
/// asset token => ANC token and distribute
/// result ANC token to gov contract
pub fn sweep(deps: DepsMut, env: Env, denom: String) -> StdResult<HandleResponse> {
    let config: Config = read_config(deps.storage)?;
    let anchor_token = deps.api.human_address(&config.anchor_token)?;
    let astroport_factory_addr = deps.api.human_address(&config.astroport_factory)?;

    let pair_info: PairInfo = query_pair_info(
        &deps.querier,
        astroport_factory_addr,
        &[
            AssetInfo::NativeToken {
                denom: denom.to_string(),
            },
            AssetInfo::Token {
                contract_addr: anchor_token,
            },
        ],
    )?;

    let amount = query_balance(
        deps.as_ref(),
        env.contract.address.clone(),
        denom.to_string(),
    )?;

    let swap_asset = Asset {
        info: AssetInfo::NativeToken {
            denom: denom.to_string(),
        },
        amount,
    };

    // deduct tax first
    let amount = (swap_asset.deduct_tax(
        &OracleContract(HumanAddr(config.oraiswap_oracle.to_string())),
        &deps.querier,
    )?)
    .amount;
    let res = HandleResponse {
        attributes: vec![
            attr("action", "sweep"),
            attr(
                "collected_rewards",
                format!("{:?}{:?}", amount.to_string(), denom),
            ),
        ],
        messages: vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: pair_info.contract_addr,
                msg: to_binary(&AstroportExecuteMsg::Swap {
                    offer_asset: Asset {
                        amount,
                        ..swap_asset
                    },
                    max_spread: config.max_spread,
                    belief_price: None,
                    to: None,
                })?,
                send: vec![Coin {
                    denom: denom.to_string(),
                    amount,
                }],
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address,
                msg: to_binary(&ExecuteMsg::Distribute {})?,
                send: vec![],
            }),
        ],
        data: None,
    };
    Ok(res)
}

// Only contract itself can execute distribute function
pub fn distribute(deps: DepsMut, env: Env) -> StdResult<HandleResponse> {
    let config: Config = read_config(deps.storage)?;
    let amount = query_token_balance(
        deps.as_ref(),
        deps.api.human_address(&config.anchor_token)?,
        env.contract.address,
    )?;

    let distribute_amount = amount * config.reward_factor;
    let left_amount = checked_sub(amount, distribute_amount)?;

    let mut messages: Vec<CosmosMsg> = vec![];

    if !distribute_amount.is_zero() {
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.human_address(&config.anchor_token)?,
            msg: to_binary(&Cw20HandleMsg::Transfer {
                recipient: deps.api.human_address(&config.gov_contract)?,
                amount: distribute_amount,
            })?,
            send: vec![],
        }));
    }

    // burn the left amount
    if !left_amount.is_zero() {
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.human_address(&config.anchor_token)?,
            msg: to_binary(&Cw20HandleMsg::Burn {
                amount: left_amount,
            })?,
            send: vec![],
        }));
    }

    let res = HandleResponse {
        attributes: vec![
            attr("action", "distribute"),
            attr("distribute_amount", &distribute_amount.to_string()),
            attr("distributor_payback_amount", &left_amount.to_string()),
        ],
        messages: messages,
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
        astroport_factory: deps
            .api
            .human_address(&state.astroport_factory)?
            .to_string(),
        anchor_token: deps.api.human_address(&state.anchor_token)?.to_string(),
        oraiswap_oracle: deps.api.human_address(&state.oraiswap_oracle)?.to_string(),
        reward_factor: state.reward_factor,
        max_spread: state.max_spread,
    };

    Ok(resp)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: MigrateMsg,
) -> StdResult<MigrateResponse> {
    //migrate config
    migrate_config(
        deps.storage,
        deps.api.canonical_address(&msg.astroport_factory)?,
        msg.max_spread,
    )?;

    Ok(MigrateResponse::default())
}

fn checked_sub(left: Uint128, right: Uint128) -> StdResult<Uint128> {
    left.0
        .checked_sub(right.0)
        .map(Uint128)
        .ok_or_else(|| StdError::generic_err("OverFlow"))
}
