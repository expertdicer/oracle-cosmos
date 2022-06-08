#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, from_binary, to_binary, HumanAddr, Binary, Deps, DepsMut, Env, MessageInfo, HandleResponse, 
    InitResponse, StdResult, CosmosMsg, BankMsg, Coin, Uint128, StdError, WasmMsg,
};
use crate::error::ContractError;
use moneymarket::dex::{InstantiateMsg, ExecuteMsg, QueryMsg, Cw20HookMsg, ConfigResponse};
use crate::state::{read_config, store_config, Config};
use cw20::{Cw20ReceiveMsg, Cw20HandleMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn init(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<InitResponse> {
    let config = Config {
        owner: deps.api.canonical_address(&msg.owner)?,
        input_token: deps.api.canonical_address(&msg.input_token)?,
        output_token: msg.output_token,
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
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::SwapForStable { recipient} => swap_for_stable(deps,env, info, recipient),
        ExecuteMsg::UpdateConfig {
            owner,
            input_token,
            output_token,
        } => {
            update_config(
                deps,
                info,
                owner,
                input_token,
                output_token,
            )
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
        Ok(Cw20HookMsg::SwapForDenom {}) => {
            // only input_token asset contract can execute this message
            let config: Config = read_config(deps.storage)?;
            if deps.api.canonical_address(&HumanAddr(contract_addr.to_string()))? != config.input_token {
                return Err(ContractError::Unauthorized {});
            }

            let cw20_sender_addr = cw20_msg.sender;
            send_denom(deps, env, cw20_sender_addr, cw20_msg.amount.into())
        }
        _ => Err(ContractError::MissingDepositCollateralHook {}),
    }
}

pub fn send_denom(
    deps: DepsMut,
    env: Env,
    receiver: HumanAddr,
    amount: Uint128,
) -> Result<HandleResponse, ContractError> {
    let config: Config = read_config(deps.storage)?;

    Ok( HandleResponse { 
        attributes: vec![
            attr("action", "deposit_collateral"),
            attr("borrower", receiver.as_str()),
            attr("amount", amount.to_string()),
        ],
        messages: vec![CosmosMsg::Bank(BankMsg::Send {
            from_address: env.contract.address,
            to_address: receiver.clone(),
            amount: vec![Coin{
                denom: config.output_token,
                amount,
            }],
        })],
        data:None,
    })
}

pub fn swap_for_stable(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    receiver: HumanAddr,
) -> Result<HandleResponse, ContractError> {
    let config: Config = read_config(deps.storage)?;

    let mut amount: Uint128;
    match info.sent_funds.iter().find(|x| x.denom.eq(&"orai".to_string())){
        Some(coin) => {
            amount = coin.amount;
        }
        None => { 
            return Err(ContractError::ZeroDepositUnallowed{})
        }
    };
    Ok( HandleResponse { 
        attributes: vec![
            attr("action", "swap_for_stable"),
            attr("receiver", receiver.as_str()),
            attr("amount", amount.to_string()),
        ],
        messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address,
            msg: to_binary(&Cw20HandleMsg::Transfer {
                recipient: receiver,
                amount: amount,
            })?,
            send: vec![],
        })],
        data:None,
    })
        
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<HumanAddr>,
    input_token: Option<HumanAddr>,
    output_token: Option<String>,
) -> Result<HandleResponse, ContractError> {
    let mut config: Config = read_config(deps.storage)?;
    if deps.api.canonical_address(&info.sender)? != config.owner {
        return Err(ContractError::Std(StdError::generic_err("unauthorized")));
    }

    if let Some(owner) = owner {
        config.owner = deps.api.canonical_address(&owner)?;
    }
    if let Some(input_token) = input_token {
        config.input_token = deps.api.canonical_address(&input_token)?;
    }
    if let Some(output_token) = output_token {
        config.output_token = output_token;
    }


    store_config(deps.storage, &config)?;
    Ok(HandleResponse::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = read_config(deps.storage)?;
    Ok(ConfigResponse {
        owner: deps.api.human_address(&config.owner)?.to_string(),
        input_token: deps
            .api
            .human_address(&config.input_token)?
            .to_string(),
        output_token: config.output_token.clone(),
    })
}

