use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::HumanAddr;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: HumanAddr,
    pub market_contract: HumanAddr,
    pub overseer_contract: HumanAddr,
    pub collateral_contract: HumanAddr,
    pub custody_borai_contract: HumanAddr,
    pub interest_contract: HumanAddr,
    pub orchai_contract: HumanAddr,
    pub stable_addr: HumanAddr,
    pub staking_contract: HumanAddr,
    pub denom_token: String,
    pub aterra_contract: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig {
        market_contract: Option<HumanAddr>,
        overseer_contract: Option<HumanAddr>,
        collateral_contract: Option<HumanAddr>,
        custody_borai_contract: Option<HumanAddr>,
        interest_contract: Option<HumanAddr>,
        orchai_contract: Option<HumanAddr>,
        stable_addr: Option<HumanAddr>,
        staking_contract: Option<HumanAddr>,
        denom_token: Option<String>,
        aterra_contract: Option<HumanAddr>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    DepositRate {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: HumanAddr,
    pub market_contract: HumanAddr,
    pub overseer_contract: HumanAddr,
    pub collateral_contract: HumanAddr,
    pub custody_borai_contract: HumanAddr,
    pub interest_contract: HumanAddr,
    pub orchai_contract: HumanAddr,
    pub stable_addr: HumanAddr,
    pub staking_contract: HumanAddr,
    pub denom_token: String,
    pub aterra_contract: HumanAddr,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositRateResponse {
    pub deposit_rate: Decimal256,
}
