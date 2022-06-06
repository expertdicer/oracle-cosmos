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
    pub base_apr: Uint256,
    pub orchai_token: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig {
        owner: Option<HumanAddr>,
        base_apr: Option<Uint256>,
        asset_token: Option<HumanAddr>,
    },
    StakingOrai {
        amount: Uint256,
    },
}
