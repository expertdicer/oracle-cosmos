use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Coin, HumanAddr};

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
    TotalBallanceDeposit { user: HumanAddr },
    CollateralBalance { user: HumanAddr },
    BorrowerInfo { borrower: HumanAddr },
    OraiBalance { user: HumanAddr },
    SOraiBalance { user: HumanAddr },
    Reward { user: HumanAddr },
    Apr {},
    BorrowRate {},
    TotalDepositAndBorrow {},
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TotalBallanceDepositResponse {
    pub total_ballance: Uint256,
    pub ausdt_ballance: Uint256,
    pub exchange_rate: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollateralBallanceResponse {
    pub borrower: String,
    pub balance: Uint256,
    pub spendable: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BorrowerInfoResponse {
    pub borrower: String,
    pub interest_index: Decimal256,
    pub reward_index: Decimal256,
    pub loan_amount: Uint256,
    pub pending_rewards: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct OraiBalanceResponse {
    /// Always returns a Coin with the requested denom.
    /// This may be of 0 amount if no such funds.
    pub amount: Coin,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SOraiBalanceResponse {
    /// Always returns a Coin with the requested denom.
    /// This may be of 0 amount if no such funds.
    pub amount: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ClaimableResponse {
    pub reward: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositAndBorrowResponse {
    pub deposit: Uint256,
    pub borrow: Decimal256,
}
