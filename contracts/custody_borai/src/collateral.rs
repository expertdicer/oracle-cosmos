use crate::error::ContractError;
use crate::state::{
    read_borrower_info, read_borrowers, read_config, remove_borrower_info, store_borrower_info,
    BorrowerInfo, Config,
};

use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    attr, to_binary, HumanAddr, CanonicalAddr, CosmosMsg, Deps, DepsMut, MessageInfo, HandleResponse,
    StdResult, WasmMsg,
};
use cw20::Cw20HandleMsg;
use moneymarket::custody::{BorrowerResponse, BorrowersResponse};
use moneymarket::liquidation::Cw20HookMsg as LiquidationCw20HookMsg;

/// Deposit new collateral
/// Executor: bAsset token contract
pub fn deposit_collateral(
    deps: DepsMut,
    borrower: HumanAddr,
    amount: Uint256,
) -> Result<HandleResponse, ContractError> {
    let borrower_raw = deps.api.canonical_address(&borrower)?;
    let mut borrower_info: BorrowerInfo = read_borrower_info(deps.storage, &borrower_raw);

    // increase borrower collateral
    borrower_info.balance += amount;
    borrower_info.spendable += amount;

    store_borrower_info(deps.storage, &borrower_raw, &borrower_info)?;

    Ok( HandleResponse { 
        attributes: vec![
            attr("action", "deposit_collateral"),
            attr("borrower", borrower.as_str()),
            attr("amount", amount.to_string()),
        ],
        messages: vec![],
        data:None,
    })
}

/// Withdraw spendable collateral or a specified amount of collateral
/// Executor: borrower
pub fn withdraw_collateral(
    deps: DepsMut,
    info: MessageInfo,
    amount: Option<Uint256>,
) -> Result<HandleResponse, ContractError> {
    let config: Config = read_config(deps.storage)?;

    let borrower = info.sender;
    let borrower_raw = deps.api.canonical_address(&borrower)?;
    let mut borrower_info: BorrowerInfo = read_borrower_info(deps.storage, &borrower_raw);

    // Check spendable balance
    let amount = amount.unwrap_or(borrower_info.spendable);
    if borrower_info.spendable < amount {
        return Err(ContractError::WithdrawAmountExceedsSpendable(
            borrower_info.spendable.into(),
        ));
    }

    // decrease borrower collateral
    borrower_info.balance = borrower_info.balance - amount;
    borrower_info.spendable = borrower_info.spendable - amount;

    if borrower_info.balance == Uint256::zero() {
        remove_borrower_info(deps.storage, &borrower_raw);
    } else {
        store_borrower_info(deps.storage, &borrower_raw, &borrower_info)?;
    }

    let res = HandleResponse {
        attributes: vec![
            attr("action", "withdraw_collateral"),
            attr("borrower", borrower.as_str()),
            attr("amount", amount.to_string()),
        ],
        messages: vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps
                    .api
                    .human_address(&config.collateral_token)?,
                send: vec![],
                msg: to_binary(&Cw20HandleMsg::Transfer {
                    recipient: borrower,
                    amount: amount.into(),
                })?,
            }),
        ],
        data: None,
    };
    Ok(res)
}

/// Decrease spendable collateral to lock
/// specified amount of collateral token
/// Executor: overseer
pub fn lock_collateral(
    deps: DepsMut,
    info: MessageInfo,
    borrower: HumanAddr,
    amount: Uint256,
) -> Result<HandleResponse, ContractError> {
    let config: Config = read_config(deps.storage)?;
    if deps.api.canonical_address(&info.sender)? != config.overseer_contract {
        return Err(ContractError::Unauthorized {});
    }

    let borrower_raw: CanonicalAddr = deps.api.canonical_address(&borrower)?;
    let mut borrower_info: BorrowerInfo = read_borrower_info(deps.storage, &borrower_raw);
    if amount > borrower_info.spendable {
        return Err(ContractError::LockAmountExceedsSpendable(
            borrower_info.spendable.into(),
        ));
    }

    borrower_info.spendable = borrower_info.spendable - amount;
    store_borrower_info(deps.storage, &borrower_raw, &borrower_info)?;

    let res = HandleResponse {
        attributes: vec![
            attr("action", "lock_collateral"),
            attr("borrower", borrower),
            attr("amount", amount),
        ],
        messages: vec![],
        data: None,
    };
    Ok(res)
}

/// Increase spendable collateral to unlock
/// specified amount of collateral token
/// Executor: overseer
pub fn unlock_collateral(
    deps: DepsMut,
    info: MessageInfo,
    borrower: HumanAddr,
    amount: Uint256,
) -> Result<HandleResponse, ContractError> {
    let config: Config = read_config(deps.storage)?;
    if deps.api.canonical_address(&info.sender)? != config.overseer_contract {
        return Err(ContractError::Unauthorized {});
    }

    let borrower_raw: CanonicalAddr = deps.api.canonical_address(&borrower)?;
    let mut borrower_info: BorrowerInfo = read_borrower_info(deps.storage, &borrower_raw);
    let borrowed_amt = borrower_info.balance - borrower_info.spendable;
    if amount > borrowed_amt {
        return Err(ContractError::UnlockAmountExceedsLocked(
            borrowed_amt.into(),
        ));
    }

    borrower_info.spendable += amount;
    store_borrower_info(deps.storage, &borrower_raw, &borrower_info)?;

    let res = HandleResponse {
        attributes: vec![
            attr("action", "unlock_collateral"),
            attr("borrower", borrower),
            attr("amount", amount),
        ],
        messages: vec![],
        data: None,
    };
    Ok(res)
}

pub fn liquidate_collateral(
    deps: DepsMut,
    info: MessageInfo,
    liquidator: HumanAddr,
    borrower: HumanAddr,
    amount: Uint256,
) -> Result<HandleResponse, ContractError> {
    let config: Config = read_config(deps.storage)?;
    if deps.api.canonical_address(&info.sender)? != config.overseer_contract {
        return Err(ContractError::Unauthorized {});
    }

    let borrower_raw: CanonicalAddr = deps.api.canonical_address(&borrower)?;
    let mut borrower_info: BorrowerInfo = read_borrower_info(deps.storage, &borrower_raw);
    let borrowed_amt = borrower_info.balance - borrower_info.spendable;
    if amount > borrowed_amt {
        return Err(ContractError::LiquidationAmountExceedsLocked(
            borrowed_amt.into(),
        ));
    }

    borrower_info.balance = borrower_info.balance - amount;
    store_borrower_info(deps.storage, &borrower_raw, &borrower_info)?;

    let res = HandleResponse {
        attributes: vec![
            attr("action", "liquidate_collateral"),
            attr("liquidator", liquidator.clone()),
            attr("borrower", borrower),
            attr("amount", amount),
        ],
        messages: vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps
                    .api
                    .human_address(&config.collateral_token)?,
                send: vec![],
                msg: to_binary(&Cw20HandleMsg::Send {
                    contract: deps
                        .api
                        .human_address(&config.liquidation_contract)?,
                    amount: amount.into(),
                    msg: Some(to_binary(&LiquidationCw20HookMsg::ExecuteBid {
                        liquidator: liquidator.to_string(),
                        fee_address: Some(
                            deps.api
                                .human_address(&config.overseer_contract)?.to_string(),
                        ),
                        repay_address: Some(
                            deps.api.human_address(&config.market_contract)?.to_string(),
                        ),
                    })?),
                })?,
            }),
        ],
        data: None,
    };
    Ok(res)
}

pub fn query_borrower(deps: Deps, borrower: HumanAddr) -> StdResult<BorrowerResponse> {
    let borrower_raw = deps.api.canonical_address(&borrower)?;
    let borrower_info: BorrowerInfo = read_borrower_info(deps.storage, &borrower_raw);
    Ok(BorrowerResponse {
        borrower: borrower.to_string(),
        balance: borrower_info.balance,
        spendable: borrower_info.spendable,
    })
}

pub fn query_borrowers(
    deps: Deps,
    start_after: Option<HumanAddr>,
    limit: Option<u32>,
) -> StdResult<BorrowersResponse> {
    let start_after = if let Some(start_after) = start_after {
        Some(deps.api.canonical_address(&start_after)?)
    } else {
        None
    };

    let borrowers = read_borrowers(deps, start_after, limit)?;
    Ok(BorrowersResponse { borrowers })
}
