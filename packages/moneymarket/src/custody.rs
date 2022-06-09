use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::Uint256;
use cosmwasm_std::HumanAddr;
use cw20::Cw20ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    /// owner address
    pub owner: HumanAddr,
    /// bAsset token address
    pub collateral_token: HumanAddr,
    /// overseer contract address
    pub overseer_contract: HumanAddr,
    /// market contract address
    pub market_contract: HumanAddr,
    /// bAsset rewrad contract
    pub reward_contract: HumanAddr,
    /// liquidation contract address
    pub liquidation_contract: HumanAddr,
    /// oraiswap contract address
    pub swap_contract: HumanAddr, // fixmed
    /// Expected reward denom. If bAsset reward is not same with
    /// it, we try to convert the reward to the `stable_denom`.
    pub stable_addr: HumanAddr,
    pub basset_info: BAssetInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// CW20 token receiver
    Receive(Cw20ReceiveMsg),

    ////////////////////
    /// Overseer operations
    ////////////////////

    /// Update config
    UpdateConfig {
        owner: Option<HumanAddr>,
        liquidation_contract: Option<HumanAddr>,
    },
    /// Make specified amount of tokens unspendable
    LockCollateral {
        borrower: HumanAddr,
        amount: Uint256,
    },
    /// Make specified amount of collateral tokens spendable
    UnlockCollateral {
        borrower: HumanAddr,
        amount: Uint256,
    },
    /// Claim bAsset rewards and distribute claimed rewards
    /// to market and overseer contracts
    DistributeRewards {},

    /// Liquidate collateral and send liquidated collateral to `to` address
    LiquidateCollateral {
        liquidator: HumanAddr,
        borrower: HumanAddr,
        amount: Uint256,
    },

    ////////////////////
    /// User operations
    ////////////////////

    /// Withdraw spendable collateral token.
    /// If the amount is not given,
    /// return all spendable collateral
    WithdrawCollateral {
        amount: Option<Uint256>,
    },

    SwapToStableDenom {},

    DistributeHook {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    /// Deposit collateral token
    DepositCollateral {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Borrower {
        address: HumanAddr,
    },
    Borrowers {
        start_after: Option<HumanAddr>,
        limit: Option<u32>,
    },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub collateral_token: String,
    pub overseer_contract: String,
    pub market_contract: String,
    pub reward_contract: String,
    pub liquidation_contract: String,
    pub stable_addr: String,
    pub basset_info: BAssetInfo,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BorrowerResponse {
    pub borrower: String,
    pub balance: Uint256,
    pub spendable: Uint256,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BorrowersResponse {
    pub borrowers: Vec<BorrowerResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BAssetInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}
