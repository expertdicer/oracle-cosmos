use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{HumanAddr, Uint128};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OverseerExternalMsg {
    /// Request bAsset reward amount
    EpochState {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MarketExternalMsg {
    /// Request bAsset reward amount
    EpochState {
        block_height: u64,
        distributed_interest: Uint256,
    },
    BorrowerInfo {
        borrower: HumanAddr,
        block_height: Option<u64>,
    },
    State {
        block_height: Option<u64>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CustodyExternalMsg {
    Borrower { address: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EpochStateResponse {
    pub deposit_rate: Decimal256,
    pub prev_aterra_supply: Uint256,
    pub prev_exchange_rate: Decimal256,
    pub prev_interest_buffer: Uint256,
    pub last_executed_height: u64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct BalanceResponse {
    pub balance: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MarketEpochStateResponse {
    pub exchange_rate: Decimal256,
    pub aterra_supply: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BorrowerResponse {
    pub borrower: String,
    pub balance: Uint256,
    pub spendable: Uint256,
}
