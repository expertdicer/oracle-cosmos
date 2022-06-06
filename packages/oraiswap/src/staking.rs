use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::asset::{Asset, AssetInfo};
use cosmwasm_std::{Decimal, HumanAddr, Uint128};
use cw20::Cw20ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    // default is sender
    pub owner: Option<HumanAddr>,
    pub rewarder: HumanAddr,
    pub minter: Option<HumanAddr>,
    pub oracle_addr: HumanAddr,
    pub factory_addr: HumanAddr,
    pub base_denom: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Receive(Cw20ReceiveMsg),

    ////////////////////////
    /// Owner operations ///
    ////////////////////////
    UpdateConfig {
        rewarder: Option<HumanAddr>,
        owner: Option<HumanAddr>,
    },
    RegisterAsset {
        asset_info: AssetInfo, // can be ow20 token or native token
        staking_token: HumanAddr,
    },
    DeprecateStakingToken {
        asset_info: AssetInfo,
        new_staking_token: HumanAddr,
    },
    // update rewards per second for an asset
    UpdateRewardsPerSec {
        asset_info: AssetInfo,
        assets: Vec<Asset>,
    },
    // reward tokens are in amount proportionaly, and used by minter contract to update amounts after checking the balance, which
    // will be used as rewards for the specified asset's staking pool.
    DepositReward {
        rewards: Vec<Asset>,
    },

    ////////////////////////
    /// User operations ///
    ////////////////////////
    Unbond {
        asset_info: AssetInfo,
        amount: Uint128,
    },
    /// Withdraw pending rewards
    Withdraw {
        // If the asset token is not given, then all rewards are withdrawn
        asset_info: Option<AssetInfo>,
    },
    // Withdraw for others in this pool, such as when rewards per second are changed for the pool
    WithdrawOthers {
        asset_info: Option<AssetInfo>,
        staker_addrs: Vec<HumanAddr>,
    },

    /// Provides liquidity and automatically stakes the LP tokens
    AutoStake {
        assets: [Asset; 2],
        slippage_tolerance: Option<Decimal>,
    },
    /// Hook to stake the minted LP tokens
    AutoStakeHook {
        asset_info: AssetInfo,
        staking_token: HumanAddr,
        staker_addr: HumanAddr,
        prev_staking_token_amount: Uint128,
    },
    UpdateListStakers {
        asset_info: AssetInfo,
        stakers: Vec<HumanAddr>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    // this call from LP token contract
    Bond { asset_info: AssetInfo },
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {
    pub staker_addrs: Vec<HumanAddr>,
    // pub amount_infos: Vec<AmountInfo>,
    // pub new_staking_token: HumanAddr,
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AmountInfo {
    pub asset_info: AssetInfo,
    pub amount: Uint128,
    // pub new_staking_token: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    PoolInfo {
        asset_info: AssetInfo,
    },
    RewardsPerSec {
        asset_info: AssetInfo,
    },
    RewardInfo {
        staker_addr: HumanAddr,
        asset_info: Option<AssetInfo>,
    },
    // Query all staker belong to the pool
    RewardInfos {
        asset_info: AssetInfo,
        start_after: Option<HumanAddr>,
        limit: Option<u32>,
        // so can convert or throw error
        order: Option<i32>,
    },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: HumanAddr,
    pub rewarder: HumanAddr,
    pub oracle_addr: HumanAddr,
    pub factory_addr: HumanAddr,
    pub base_denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RewardsPerSecResponse {
    pub assets: Vec<Asset>,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PoolInfoResponse {
    pub asset_info: AssetInfo,
    pub staking_token: HumanAddr,
    pub total_bond_amount: Uint128,
    pub reward_index: Decimal,
    pub pending_reward: Uint128,
    pub migration_index_snapshot: Option<Decimal>,
    pub migration_deprecated_staking_token: Option<HumanAddr>,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RewardInfoResponse {
    pub staker_addr: HumanAddr,
    pub reward_infos: Vec<RewardInfoResponseItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RewardInfoResponseItem {
    pub asset_info: AssetInfo,
    pub bond_amount: Uint128,
    pub pending_reward: Uint128,
    pub pending_withdraw: Vec<Asset>,
    // returns true if the position should be closed to keep receiving rewards
    // with the new lp token
    pub should_migrate: Option<bool>,
}
