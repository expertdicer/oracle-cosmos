use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{to_binary, HumanAddr, Deps, QueryRequest, StdResult, WasmQuery};

use moneymarket::distribution_model::{AncEmissionRateResponse, QueryMsg as DistributionQueryMsg};
use moneymarket::interest_model::{BorrowRateResponse, QueryMsg as InterestQueryMsg};
use moneymarket::overseer::{BorrowLimitResponse, ConfigResponse, QueryMsg as OverseerQueryMsg};

pub fn query_borrow_rate(
    deps: Deps,
    interest_addr: HumanAddr,
    market_balance: Uint256,
    total_liabilities: Decimal256,
    total_reserves: Decimal256,
) -> StdResult<BorrowRateResponse> {
    let borrow_rate: BorrowRateResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: interest_addr,
            msg: to_binary(&InterestQueryMsg::BorrowRate {
                market_balance,
                total_liabilities,
                total_reserves,
            })?,
        }))?;

    Ok(borrow_rate)
}

pub fn query_borrow_limit(
    deps: Deps,
    overseer_addr: HumanAddr,
    borrower: HumanAddr,
    block_time: Option<u64>,
) -> StdResult<BorrowLimitResponse> {
    let borrow_limit: BorrowLimitResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: overseer_addr,
            msg: to_binary(&OverseerQueryMsg::BorrowLimit {
                borrower: borrower,
                block_time,
            })?,
        }))?;

    Ok(borrow_limit)
}

pub fn query_orchai_epb_rate(
    deps: Deps,
    distribution_model: HumanAddr,
    deposit_rate: Decimal256,
    target_deposit_rate: Decimal256,
    threshold_deposit_rate: Decimal256,
    current_emission_rate: Decimal256,
) -> StdResult<AncEmissionRateResponse> {
    let orchai_epb_rate: AncEmissionRateResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: distribution_model,
            msg: to_binary(&DistributionQueryMsg::AncEmissionRate {
                deposit_rate,
                target_deposit_rate,
                threshold_deposit_rate,
                current_emission_rate,
            })?,
        }))?;

    Ok(orchai_epb_rate)
}

pub fn query_target_deposit_rate(deps: Deps, overseer_contract: HumanAddr) -> StdResult<Decimal256> {
    let overseer_config: ConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: overseer_contract,
            msg: to_binary(&OverseerQueryMsg::Config {})?,
        }))?;

    Ok(overseer_config.target_deposit_rate)
}
