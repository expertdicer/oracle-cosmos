use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::HumanAddr;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: HumanAddr,
    pub native_token_denom: String, // "ORAI"
    pub native_token: HumanAddr,
    pub asset_token: HumanAddr,
    pub base_apr: Decimal256,
    pub orchai_token: HumanAddr,
    pub validator_to_delegate: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    QueryConfig {},
    Claimable {
        user: HumanAddr,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig {
        owner: Option<HumanAddr>,
        base_apr: Option<Decimal256>,
        asset_token: Option<HumanAddr>,
        validator_to_delegate: Option<HumanAddr>,
    },
    StakingOrai {
        amount: Uint256,
    },
    ClaimReward {},
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: HumanAddr,
    pub native_token_denom: String, // "ORAI"
    pub native_token: HumanAddr,
    pub asset_token: HumanAddr,
    pub base_apr: Decimal256,
    pub orchai_token: HumanAddr,
    pub validator_to_delegate: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ClaimableResponse {
    pub reward: Uint256,
}