use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, AllBalanceResponse, BalanceResponse, BankQuery, Coin,
    ContractResult, Decimal, Empty, HumanAddr, OwnedDeps, Querier, QuerierResult, QuerierWrapper,
    QueryRequest, StdResult, SystemError, SystemResult, Uint128, WasmQuery,
};
use std::collections::HashMap;

use crate::asset::PairInfo;
use crate::factory::QueryMsg as FactoryQueryMsg;
use crate::oracle::{OracleQuery, OracleTreasuryQuery, TaxCapResponse, TaxRateResponse};
use cw20::{BalanceResponse as Cw20BalanceResponse, Cw20QueryMsg, TokenInfoResponse};

pub const ATOM_DENOM: &str = "ibc/1777D03C5392415FE659F0E8ECB2CE553C6550542A68E4707D5D46949116790B";

pub fn query_balance(
    querier: &QuerierWrapper,
    account_addr: HumanAddr,
    denom: String,
) -> StdResult<Uint128> {
    // load price form the oracle
    let balance: BalanceResponse = querier.query(&QueryRequest::Bank(BankQuery::Balance {
        address: account_addr,
        denom,
    }))?;
    Ok(balance.amount.amount)
}

pub fn query_all_balances(
    querier: &QuerierWrapper,
    account_addr: HumanAddr,
) -> StdResult<Vec<Coin>> {
    // load price form the oracle
    let all_balances: AllBalanceResponse =
        querier.query(&QueryRequest::Bank(BankQuery::AllBalances {
            address: account_addr,
        }))?;
    Ok(all_balances.amount)
}

/// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
/// this uses our CustomQuerier.
pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier: WasmMockQuerier = WasmMockQuerier::new(MockQuerier::new(&[(
        &MOCK_CONTRACT_ADDR.into(),
        contract_balance,
    )]));

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier,
    token_querier: TokenQuerier,
    tax_querier: TaxQuerier,
    oraiswap_factory_querier: OraiswapFactoryQuerier,
}

#[derive(Clone, Default)]
pub struct TokenQuerier {
    // this lets us iterate over all pairs that match the first string
    balances: HashMap<String, HashMap<String, Uint128>>,
}

impl TokenQuerier {
    pub fn new(balances: &[(&String, &[(&String, &Uint128)])]) -> Self {
        TokenQuerier {
            balances: balances_to_map(balances),
        }
    }
}

pub(crate) fn balances_to_map(
    balances: &[(&String, &[(&String, &Uint128)])],
) -> HashMap<String, HashMap<String, Uint128>> {
    let mut balances_map: HashMap<String, HashMap<String, Uint128>> = HashMap::new();
    for (contract_addr, balances) in balances.iter() {
        let mut contract_balances_map: HashMap<String, Uint128> = HashMap::new();
        for (addr, balance) in balances.iter() {
            contract_balances_map.insert(addr.to_string(), **balance);
        }

        balances_map.insert(contract_addr.to_string(), contract_balances_map);
    }
    balances_map
}

#[derive(Clone, Default)]
pub struct TaxQuerier {
    rate: Decimal,
    // this lets us iterate over all pairs that match the first string
    caps: HashMap<String, Uint128>,
}

impl TaxQuerier {
    pub fn new(rate: Decimal, caps: &[(&String, &Uint128)]) -> Self {
        TaxQuerier {
            rate,
            caps: caps_to_map(caps),
        }
    }
}

pub(crate) fn caps_to_map(caps: &[(&String, &Uint128)]) -> HashMap<String, Uint128> {
    let mut owner_map: HashMap<String, Uint128> = HashMap::new();
    for (denom, cap) in caps.iter() {
        owner_map.insert(denom.to_string(), **cap);
    }
    owner_map
}

#[derive(Clone, Default)]
pub struct OraiswapFactoryQuerier {
    pairs: HashMap<String, PairInfo>,
}

impl OraiswapFactoryQuerier {
    pub fn new(pairs: &[(&String, &PairInfo)]) -> Self {
        OraiswapFactoryQuerier {
            pairs: pairs_to_map(pairs),
        }
    }
}

pub(crate) fn pairs_to_map(pairs: &[(&String, &PairInfo)]) -> HashMap<String, PairInfo> {
    let mut pairs_map: HashMap<String, PairInfo> = HashMap::new();
    for (key, pair) in pairs.iter() {
        pairs_map.insert(key.to_string(), (*pair).clone());
    }
    pairs_map
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely here
        let request: QueryRequest<Empty> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<Empty>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => match from_binary(msg) {
                // maybe querywrapper like custom query from smart contract
                Ok(OracleQuery::Treasury(query_data)) => match query_data {
                    OracleTreasuryQuery::TaxRate {} => {
                        let res = TaxRateResponse {
                            rate: self.tax_querier.rate,
                        };
                        SystemResult::Ok(ContractResult::Ok(to_binary(&res).unwrap()))
                    }
                    OracleTreasuryQuery::TaxCap { denom } => {
                        let cap = self
                            .tax_querier
                            .caps
                            .get(&denom)
                            .copied()
                            .unwrap_or_default();
                        let res = TaxCapResponse { cap };
                        SystemResult::Ok(ContractResult::Ok(to_binary(&res).unwrap()))
                    }
                },

                // try with FactoryQueryMsg
                _ => match from_binary(msg) {
                    Ok(FactoryQueryMsg::Pair { asset_infos }) => {
                        let key = asset_infos[0].to_string() + asset_infos[1].to_string().as_str();
                        match self.oraiswap_factory_querier.pairs.get(&key) {
                            Some(v) => SystemResult::Ok(ContractResult::Ok(to_binary(&v).unwrap())),
                            None => SystemResult::Err(SystemError::InvalidRequest {
                                error: "No pair info exists".to_string(),
                                request: msg.as_slice().into(),
                            }),
                        }
                    }

                    _ => match from_binary(msg).unwrap() {
                        Cw20QueryMsg::TokenInfo {} => {
                            let balances: &HashMap<String, Uint128> =
                                match self.token_querier.balances.get(contract_addr.as_str()) {
                                    Some(balances) => balances,
                                    None => {
                                        return SystemResult::Err(SystemError::InvalidRequest {
                                            error: format!(
                                                "No balance info exists for the contract {}",
                                                contract_addr
                                            ),
                                            request: msg.as_slice().into(),
                                        })
                                    }
                                };

                            let mut total_supply = Uint128::zero();

                            for balance in balances {
                                total_supply += *balance.1;
                            }

                            SystemResult::Ok(ContractResult::Ok(
                                to_binary(&TokenInfoResponse {
                                    name: "mAAPL".to_string(),
                                    symbol: "mAAPL".to_string(),
                                    decimals: 6,
                                    total_supply,
                                })
                                .unwrap(),
                            ))
                        }
                        Cw20QueryMsg::Balance { address } => {
                            let balances: &HashMap<String, Uint128> =
                                match self.token_querier.balances.get(contract_addr.as_str()) {
                                    Some(balances) => balances,
                                    None => {
                                        return SystemResult::Err(SystemError::InvalidRequest {
                                            error: format!(
                                                "No balance info exists for the contract {}",
                                                contract_addr
                                            ),
                                            request: msg.as_slice().into(),
                                        })
                                    }
                                };

                            let balance = match balances.get(address.as_str()) {
                                Some(v) => *v,
                                None => {
                                    return SystemResult::Ok(ContractResult::Ok(
                                        to_binary(&Cw20BalanceResponse {
                                            balance: Uint128::zero(),
                                        })
                                        .unwrap(),
                                    ));
                                }
                            };

                            SystemResult::Ok(ContractResult::Ok(
                                to_binary(&Cw20BalanceResponse { balance }).unwrap(),
                            ))
                        }
                        _ => panic!("DO NOT ENTER HERE"),
                    },
                },
            },
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<Empty>) -> Self {
        WasmMockQuerier {
            base,
            token_querier: TokenQuerier::default(),
            tax_querier: TaxQuerier::default(),
            oraiswap_factory_querier: OraiswapFactoryQuerier::default(),
        }
    }

    // configure the mint whitelist mock querier
    pub fn with_token_balances(&mut self, balances: &[(&String, &[(&String, &Uint128)])]) {
        self.token_querier = TokenQuerier::new(balances);
    }

    // configure the token owner mock querier
    pub fn with_tax(&mut self, rate: Decimal, caps: &[(&String, &Uint128)]) {
        self.tax_querier = TaxQuerier::new(rate, caps);
    }

    // configure the oraiswap pair
    pub fn with_oraiswap_pairs(&mut self, pairs: &[(&String, &PairInfo)]) {
        self.oraiswap_factory_querier = OraiswapFactoryQuerier::new(pairs);
    }
}
