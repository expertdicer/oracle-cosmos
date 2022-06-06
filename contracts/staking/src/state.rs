use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{CanonicalAddr, StdResult, Storage, HumanAddr, StdError};
use cosmwasm_storage::{singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton, Singleton};

static KEY_CONFIG: &[u8] = b"config";
const PREFIX_USER_REWARD: &[u8] = b"user_reward";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: HumanAddr,
    pub native_token_denom: String, // "ORAI"
    pub native_token: HumanAddr,
    pub asset_token: HumanAddr,
    pub base_apr: Uint256,
    pub orchai_token: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserReward {
    pub last_reward: Uint256,
    pub last_time: u64,
    pub amount: Uint256,
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    singleton(storage, KEY_CONFIG).save(config)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    singleton_read(storage, KEY_CONFIG).load()
}

pub fn store_user_reward_elem(
    storage: &mut dyn Storage,
    user: &CanonicalAddr,
    user_reward_elem: &UserReward,
) -> StdResult<()> {
    let mut user_reward_bucket: Bucket<UserReward> = Bucket::new(storage, PREFIX_USER_REWARD);
    user_reward_bucket.save(user.as_slice(), user_reward_elem)?;

    Ok(())
}

pub fn read_user_reward_elem(
    storage: &dyn Storage,
    user: &CanonicalAddr,
) -> StdResult<UserReward> {
    let user_reward_bucket: ReadonlyBucket<UserReward> =
        ReadonlyBucket::new(storage, PREFIX_USER_REWARD);
    match user_reward_bucket.load(user.as_slice()) {
        Ok(v) => Ok(v),
        _ => Err(StdError::generic_err(
            "Token is not registered as collateral",
        )),
    }
}