use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{CanonicalAddr, HumanAddr, StdResult, Storage};
use cosmwasm_storage::{singleton, singleton_read};

static KEY_CONFIG: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
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

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    singleton(storage, KEY_CONFIG).save(config)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    singleton_read(storage, KEY_CONFIG).load()
}
