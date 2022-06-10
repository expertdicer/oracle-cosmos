use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::HumanAddr;

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// #[serde(rename_all = "snake_case")]
// pub enum OverseerQueryAndResponse {
//     /// Request bAsset reward amount
//     AccruedRewards { address: String },
//     EpochState {
//         deposit_rate: Decimal256,
//         prev_aterra_supply: Uint256,
//         prev_exchange_rate: Decimal256,
//         prev_interest_buffer: Uint256,
//         last_executed_height: u64,
//     },
// }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EpochStateResponse {
    pub exchange_rate: Decimal256,
    pub aterra_supply: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryEpochState {
    pub block_height: u64,
    pub distributed_interest: Uint256,
}
