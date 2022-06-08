use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cw20::Cw20ReceiveMsg;
use cw20::{Cw20Coin, MinterResponse};
use cosmwasm_std::{HumanAddr, Attribute, Binary, Uint128};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    /// Owner address for config update
    pub owner_addr: HumanAddr,
    /// stable coin denom used to borrow & repay
    pub stable_addr: HumanAddr,
    /// Anchor token code ID used to instantiate
    pub orchai_code_id: u64,
    /// Anchor token distribution speed
    pub anc_emission_rate: Decimal256,
    /// Maximum allowed borrow rate over deposited stable balance
    pub max_borrow_factor: Decimal256, 

    // pub hook_msg: HookMsg,
}

/// InstantiateMsg Hook
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HookMsg {
    pub contract_addr: HumanAddr,
    pub amount: Uint256,
    pub recipient: HumanAddr,
}


/// TokenContract InstantiateMsg
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct TokenInstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub initial_balances: Vec<Cw20Coin>,
    pub mint: Option<MinterResponse>,
    pub init_hook: Option<InitHook>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),

    ////////////////////
    /// Owner operations
    ////////////////////
    /// Register Contracts contract address
    RegisterContracts {
        overseer_contract: HumanAddr,
        /// The contract has the logics for
        /// Anchor borrow interest rate
        interest_model: HumanAddr,
        /// The contract has the logics for
        /// ANC distribution speed
        distribution_model: HumanAddr,
        /// Collector contract to send all the reserve
        collector_contract: HumanAddr,
        /// Faucet contract to drip ANC token to users
        distributor_contract: HumanAddr,
    },

    /// Update config values
    UpdateConfig {
        owner_addr: Option<HumanAddr>,
        max_borrow_factor: Option<Decimal256>,
        interest_model: Option<HumanAddr>,
        distribution_model: Option<HumanAddr>,
    },

    ////////////////////
    /// Overseer operations
    ////////////////////
    /// Repay stable with liquidated collaterals
    RepayStableFromLiquidation {
        borrower: String,
        prev_balance: Uint256,
    },

    /// Execute epoch operations
    /// 1. send reserve to collector contract
    /// 2. update anc_emission_rate state
    ExecuteEpochOperations {
        deposit_rate: Decimal256,
        target_deposit_rate: Decimal256,
        threshold_deposit_rate: Decimal256,
        distributed_interest: Uint256,
    },

    ////////////////////
    /// User operations
    ////////////////////
    /// Deposit stable asset to get interest
    // DepositStable {},

    /// Borrow stable asset with collaterals in overseer contract
    BorrowStable {
        borrow_amount: Uint256,
        to: Option<HumanAddr>,
    },

    /// Repay stable asset to decrease liability
    // RepayStable {},

    /// Claim distributed ANC rewards
    ClaimRewards {
        to: Option<HumanAddr>,
    },

    RegisterATerra {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]  // huhu
#[serde(rename_all = "snake_case")]
pub struct InitHook {
    pub msg: Binary,
    pub contract_addr: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]  // fixme
#[serde(rename_all = "snake_case")]
pub enum SubMsgResult {
    Ok(SubMsgResponse),
    #[serde(rename = "error")]
    Err(String),
}

// Implementations here mimic the Result API and should be implemented via a conversion to Result
// to ensure API consistency
impl SubMsgResult {
    /// Converts a `SubMsgResult<S>` to a `Result<S, String>` as a convenient way
    /// to access the full Result API.
    pub fn into_result(self) -> Result<SubMsgResponse, String> {
        Result::<SubMsgResponse, String>::from(self)
    }

    pub fn unwrap(self) -> SubMsgResponse {
        self.into_result().unwrap()
    }

    pub fn unwrap_err(self) -> String {
        self.into_result().unwrap_err()
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, SubMsgResult::Ok(_))
    }

    pub fn is_err(&self) -> bool {
        matches!(self, SubMsgResult::Err(_))
    }
}

impl<E: ToString> From<Result<SubMsgResponse, E>> for SubMsgResult {
    fn from(original: Result<SubMsgResponse, E>) -> SubMsgResult {
        match original {
            Ok(value) => SubMsgResult::Ok(value),
            Err(err) => SubMsgResult::Err(err.to_string()),
        }
    }
}

impl From<SubMsgResult> for Result<SubMsgResponse, String> {
    fn from(original: SubMsgResult) -> Result<SubMsgResponse, String> {
        match original {
            SubMsgResult::Ok(value) => Ok(value),
            SubMsgResult::Err(err) => Err(err),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]  // fixme
#[serde(rename_all = "snake_case")]
pub struct SubMsgResponse {
    pub events: Vec<Event>,
    pub data: Option<Binary>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]  // fixme
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct Event {
    pub ty: String,
    pub attributes: Vec<Attribute>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    /// Return stable coins to a user
    /// according to exchange rate
    RedeemStable {},
    DepositStabe {},
    RepayStable {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    State {
        block_height: Option<u64>,
    },
    EpochState {
        block_height: Option<u64>,
        distributed_interest: Option<Uint256>,
    },
    BorrowerInfo {
        borrower: String,
        block_height: Option<u64>,
    },
    BorrowerInfos {
        start_after: Option<HumanAddr>,
        limit: Option<u32>,
    },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner_addr: String,
    pub aterra_contract: String,
    pub interest_model: String,
    pub distribution_model: String,
    pub overseer_contract: String,
    pub collector_contract: String,
    pub distributor_contract: String,
    pub stable_addr: String,
    pub max_borrow_factor: Decimal256,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub total_liabilities: Decimal256,
    pub total_reserves: Decimal256,
    pub last_interest_updated: u64,
    pub last_reward_updated: u64,
    pub global_interest_index: Decimal256,
    pub global_reward_index: Decimal256,
    pub anc_emission_rate: Decimal256,
    pub prev_aterra_supply: Uint256,
    pub prev_exchange_rate: Decimal256,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EpochStateResponse {
    pub exchange_rate: Decimal256,
    pub aterra_supply: Uint256,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BorrowerInfoResponse {
    pub borrower: String,
    pub interest_index: Decimal256,
    pub reward_index: Decimal256,
    pub loan_amount: Uint256,
    pub pending_rewards: Decimal256,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BorrowerInfosResponse {
    pub borrower_infos: Vec<BorrowerInfoResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}
