use cosmwasm_std::{to_binary, QuerierWrapper, QueryRequest, StdResult, WasmQuery, HumanAddr, Deps};
use moneymarket::overseer::{
    QueryMsg as OverseerQueryMsg, WhitelistResponse, WhitelistResponseElem,
};
use oraiswap::oracle::OracleContract;
use cosmwasm_bignumber::{Decimal256, Uint256};

pub fn query_collateral_whitelist_info(
    querier: &QuerierWrapper,
    overseer: HumanAddr,
    collateral_token: HumanAddr,
) -> StdResult<WhitelistResponseElem> {
    let whitelist_res: WhitelistResponse =
        querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: overseer,
            msg: to_binary(&OverseerQueryMsg::Whitelist {
                collateral_token: Some(collateral_token),
                start_after: None,
                limit: None,
            })?,
        }))?;

    Ok(whitelist_res.elems[0].clone())
}

pub fn query_tax_rate_and_cap(deps: Deps, denom: String, orai_oracle: HumanAddr) -> StdResult<(Decimal256, Uint256)> {
    let orai_querier = OracleContract(orai_oracle);
    let rate = orai_querier.query_tax_rate(&deps.querier)?.rate;
    let cap = orai_querier.query_tax_cap(&deps.querier,denom)?.cap;
    Ok((rate.into(), cap.into()))
}
