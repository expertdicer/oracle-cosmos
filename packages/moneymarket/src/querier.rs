use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    to_binary, AllBalanceResponse, BalanceResponse, BankQuery, Coin, Deps, HumanAddr, QueryRequest,
    StdError, StdResult, Uint128, WasmQuery,
};
use cw20::{Cw20QueryMsg, TokenInfoResponse};
use oraiswap::oracle::OracleContract;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// use terra_cosmwasm::TerraQuerier;

use crate::oracle::{PriceResponse, QueryMsg as OracleQueryMsg};

pub fn query_all_balances(deps: Deps, account_addr: HumanAddr) -> StdResult<Vec<Coin>> {
    // load price form the oracle
    let all_balances: AllBalanceResponse =
        deps.querier
            .query(&QueryRequest::Bank(BankQuery::AllBalances {
                address: account_addr,
            }))?;
    Ok(all_balances.amount)
}

pub fn query_balance(
    deps: Deps,
    account_addr: HumanAddr,
    stable_addr: HumanAddr,
) -> StdResult<Uint256> {
    // load price form the oracle
    let balance: BalanceResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: stable_addr,
        msg: to_binary(&Cw20QueryMsg::Balance {
            address: account_addr,
        })?,
    }))?;
    Ok(balance.amount.amount.into())
}

pub fn query_token_balance(
    deps: Deps,
    contract_addr: HumanAddr,
    account_addr: HumanAddr,
) -> StdResult<Uint256> {
    // load balance form the token contract
    let balance: Uint128 = deps
        .querier
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract_addr,
            msg: to_binary(&Cw20QueryMsg::Balance {
                address: account_addr,
            })?,
        }))
        .unwrap_or_else(|_| Uint128::zero());

    Ok(balance.into())
}

pub fn query_supply(deps: Deps, contract_addr: HumanAddr) -> StdResult<Uint256> {
    // load price form the oracle
    let token_info: TokenInfoResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract_addr,
            msg: to_binary(&Cw20QueryMsg::TokenInfo {})?,
        }))?;

    Ok(Uint256::from(token_info.total_supply))
}

pub fn query_tax_rate_and_cap(
    deps: Deps,
    denom: String,
    orai_oracle: HumanAddr,
) -> StdResult<(Decimal256, Uint256)> {
    let orai_querier = OracleContract(orai_oracle);
    let rate = orai_querier.query_tax_rate(&deps.querier)?.rate;
    let cap = orai_querier.query_tax_cap(&deps.querier, denom)?.cap;
    Ok((rate.into(), cap.into()))
}

pub fn query_tax_rate(deps: Deps, orai_oracle: HumanAddr) -> StdResult<Decimal256> {
    let orai_querier = OracleContract(orai_oracle);
    Ok(orai_querier.query_tax_rate(&deps.querier)?.rate.into())
}

pub fn compute_tax(
    deps: Deps,
    coin: &Coin,
    denom: String,
    orai_oracle: HumanAddr,
) -> StdResult<Uint256> {
    let orai_querier = OracleContract(orai_oracle);
    let tax_rate = Decimal256::from(orai_querier.query_tax_rate(&deps.querier)?.rate);
    let tax_cap = Uint256::from(orai_querier.query_tax_cap(&deps.querier, denom)?.cap);
    let amount = Uint256::from(coin.amount);
    Ok(std::cmp::min(
        amount * Decimal256::one() - amount / (Decimal256::one() + tax_rate),
        tax_cap,
    ))
}

// pub fn query_tax_rate_and_cap(deps: Deps, denom: String) -> StdResult<(Decimal256, Uint256)> {
//     let terra_querier = TerraQuerier::new(&deps.querier);
//     let rate = terra_querier.query_tax_rate()?.rate;
//     let cap = terra_querier.query_tax_cap(denom)?.cap;
//     Ok((rate.into(), cap.into()))
// }

// pub fn query_tax_rate(deps: Deps) -> StdResult<Decimal256> {
//     let terra_querier = TerraQuerier::new(&deps.querier);
//     Ok(terra_querier.query_tax_rate()?.rate.into())
// }

// pub fn compute_tax(deps: Deps, coin: &Coin) -> StdResult<Uint256> {
//     let terra_querier = TerraQuerier::new(&deps.querier);
//     let tax_rate = Decimal256::from((terra_querier.query_tax_rate()?).rate);
//     let tax_cap = Uint256::from((terra_querier.query_tax_cap(coin.denom.to_string())?).cap);
//     let amount = Uint256::from(coin.amount);
//     Ok(std::cmp::min(
//         amount * Decimal256::one() - amount / (Decimal256::one() + tax_rate),
//         tax_cap,
//     ))
// }

pub fn deduct_tax(_deps: Deps, amount: Uint128) -> StdResult<Uint128> {
    // let tax_amount = compute_tax(deps, &coin)?;
    let tax_amount: Uint256 = Uint256::zero();
    Ok((Uint256::from(amount) - tax_amount).into())
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TimeConstraints {
    pub block_time: u64,
    pub valid_timeframe: u64,
}

pub fn query_price(
    deps: Deps,
    oracle_addr: HumanAddr,
    base: String,
    quote: String,
    time_contraints: Option<TimeConstraints>,
) -> StdResult<PriceResponse> {
    let oracle_price: PriceResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: oracle_addr,
            msg: to_binary(&OracleQueryMsg::Price { base, quote })?,
        }))?;

    if let Some(time_contraints) = time_contraints {
        let valid_update_time = time_contraints.block_time - time_contraints.valid_timeframe;
        if oracle_price.last_updated_base < valid_update_time
            || oracle_price.last_updated_quote < valid_update_time
        {
            return Err(StdError::generic_err("Price is too old"));
        }
    }

    Ok(oracle_price)
}
