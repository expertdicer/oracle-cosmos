use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::HumanAddr;
use cw20::Cw20ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: HumanAddr,
    pub native_token_denom: String, // "ORAI"
    pub asset_token: HumanAddr,
    pub base_apr: Decimal256,
    pub orchai_token: HumanAddr,
    pub validator_to_delegate: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    QueryConfig {},
    Claimable { user: HumanAddr },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    UpdateConfig {
        owner: Option<HumanAddr>,
        base_apr: Option<Decimal256>,
        asset_token: Option<HumanAddr>,
        validator_to_delegate: Option<HumanAddr>,
        orchai_token: Option<HumanAddr>,
    },
    StakingOrai {},
    ClaimRewards {
        recipient: Option<HumanAddr>,
    },
    UpdateUserReward {
        user: HumanAddr,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    /// Deposit collateral token
    WithdrawCollateral {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: HumanAddr,
    pub native_token_denom: String, // "ORAI"
    pub asset_token: HumanAddr,
    pub base_apr: Decimal256,
    pub orchai_token: HumanAddr,
    pub validator_to_delegate: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ClaimableResponse {
    pub reward: Uint256,
}
