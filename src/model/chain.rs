//! Models for the Aster-Chain API (`/aster-chain/v3`, host `chainapi.*`).
//!
//! Aster-Chain is a third documented surface next to futures (`/fapi/v3`) and
//! spot (`/api/v3`). It owns account privacy status, address transfers, staking
//! and the deposit/withdraw flows. Shapes follow
//! `V3(Recommended)/EN/aster-chain.md`.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// `GET /aster-chain/v3/account/status` and `POST …/account/modify-status`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainAccountStatus {
    /// `PUBLIC` or `PRIVATE`.
    pub status: String,
}

/// One destination of a batch transfer.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchTransferItem {
    /// Destination address.
    pub to_address: String,
    /// Asset.
    pub asset: String,
    /// Amount.
    pub amount: String,
    /// Optional destination chain id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transfer_chain_id: Option<i64>,
}

/// `POST /aster-chain/v3/transfer`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainTransferResult {
    /// Server-side transfer id.
    pub transfer_id: String,
    /// Asset.
    pub asset: String,
    /// Amount.
    pub amount: Decimal,
    /// Destination address.
    pub to_address: String,
    /// Creation timestamp in milliseconds.
    pub timestamp: i64,
    /// Processing status.
    pub status: String,
}

/// Per-item result inside [`BatchTransferResult`].
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchTransferItemResult {
    /// Destination address.
    pub to_address: String,
    /// Asset.
    pub asset: String,
    /// Server-side transfer id.
    pub transfer_id: String,
}

/// `POST /aster-chain/v3/batch-transfer`.
///
/// Processing is asynchronous: the response only confirms the batch was
/// accepted. `client_tran_id` makes the call idempotent.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchTransferResult {
    /// Caller-supplied idempotency key.
    pub client_tran_id: String,
    /// Batch status.
    pub status: String,
    /// Number of items accepted.
    pub total_count: i32,
    /// Per-item identifiers.
    #[serde(default)]
    pub items: Vec<BatchTransferItemResult>,
}

/// A failed item in a batch transfer history row.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchTransferFailedItem {
    /// Server-side transfer id, when one was assigned.
    #[serde(default)]
    pub transfer_id: Option<String>,
    /// Destination address.
    pub to_address: String,
    /// Asset.
    pub asset: String,
    /// Amount.
    pub amount: String,
}

/// One row of `GET /aster-chain/v3/batch-transfer/history`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchTransferHistoryRow {
    /// Caller-supplied idempotency key.
    pub client_tran_id: String,
    /// `I`/`P`/`S`/`F`/`M`.
    pub status: String,
    /// Creation timestamp in milliseconds.
    pub create_time: i64,
    /// Last update timestamp in milliseconds.
    pub update_time: i64,
    /// Total item count.
    pub total_count: i32,
    /// Item counts per outcome.
    #[serde(default)]
    pub success_count: i32,
    /// Failed item count.
    #[serde(default)]
    pub fail_count: i32,
    /// Partially processed item count.
    #[serde(default)]
    pub partial_count: i32,
    /// Still-processing item count.
    #[serde(default)]
    pub processing_count: i32,
    /// Details for failed items.
    #[serde(default)]
    pub failed_items: Vec<BatchTransferFailedItem>,
}

/// Paginated envelope of `GET /aster-chain/v3/batch-transfer/history`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchTransferHistory {
    /// Total number of matching batches.
    pub total: i64,
    /// Page rows.
    #[serde(default)]
    pub rows: Vec<BatchTransferHistoryRow>,
}

/// `GET /aster-chain/v3/staking/stakeAccountStatus`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StakingAccountStatus {
    /// Raw status flag reported by the staking module.
    pub stake_account_status: String,
}

/// `GET /aster-chain/v3/staking/myStaking`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MyStaking {
    /// Validator the stake is delegated to.
    pub validator_address: String,
    /// Staked amount.
    pub stake_amount: Decimal,
    /// Lock period code.
    pub period_code: String,
    /// Lock start timestamp in milliseconds.
    pub start_time: i64,
    /// Lock end timestamp in milliseconds.
    pub end_time: i64,
    /// Stake status.
    pub status: String,
}

/// `GET /aster-chain/v3/staking/claimableRewards`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimableRewards {
    /// Amount currently claimable.
    pub claimable_amount: Decimal,
    /// Reward asset.
    pub asset: String,
    /// Rewards accrued since the last claim.
    pub accrued_rewards: Decimal,
    /// Timestamp of the last claim in milliseconds.
    pub last_claim_time: i64,
}

/// Result of a staking write (`create`/`deposit`/`updateLockPeriod`/`claimRewards`).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StakingActionResult {
    /// Raw result flag reported by the staking module.
    pub result: StakingResult,
}

/// Success flag of a staking write.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum StakingResult {
    /// Boolean form.
    Bool(bool),
    /// Numeric form (`1` for success).
    Number(i64),
    /// String form.
    Text(String),
}

/// One element of `GET /aster-chain/v3/staking/getLockedAster`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LockedAsterPeriod {
    /// Lock period code.
    pub period_code: String,
    /// Total volume locked under this period.
    pub total_volume: Decimal,
}

/// `GET /aster-chain/v3/spot/user-deposit-address`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DepositAddress {
    /// Network label.
    pub network: String,
    /// Deposit address.
    pub address: String,
}

/// Per-chain withdraw limits inside [`UserWithdrawInfo`].
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainWithdrawLimit {
    /// Chain id of the settlement network.
    pub chain_id: i64,
    /// Spot-side maximum withdrawable amount.
    #[serde(default)]
    pub spot_max_withdraw_amount: Decimal,
    /// Perp-side maximum withdrawable amount.
    #[serde(default)]
    pub perp_max_withdraw_amount: Decimal,
    /// Chain-level limit.
    #[serde(default)]
    pub chain_limit: Decimal,
}

/// Per-asset withdraw limits inside [`UserWithdrawInfo`].
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetWithdrawLimit {
    /// Currency label.
    pub currency: String,
    /// Total withdrawn from the spot account today.
    #[serde(default)]
    pub spot_total_withdraw_amount: Decimal,
    /// Total withdrawn from the perp account today.
    #[serde(default)]
    pub perp_total_withdraw_amount: Decimal,
    /// Daily limit for this asset.
    #[serde(default)]
    pub daily_limit: Decimal,
    /// Per-chain limits, keyed by chain id.
    #[serde(default)]
    pub chain_balances: std::collections::BTreeMap<String, ChainWithdrawLimit>,
}

/// `GET /aster-chain/v3/perp/user-withdraw-info`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserWithdrawInfo {
    /// User-level daily limit.
    pub user_daily_limit: Decimal,
    /// Remaining user-level daily allowance.
    pub user_remaining_daily_limit: Decimal,
    /// Platform-wide daily limit.
    pub total_daily_limit: Decimal,
    /// Remaining platform-wide allowance.
    pub total_remaining_daily_limit: Decimal,
    /// Per-asset limits, keyed by asset symbol.
    #[serde(default)]
    pub balances: std::collections::BTreeMap<String, AssetWithdrawLimit>,
}

/// `GET /aster-chain/v3/withdraw/estimateFee`.
///
/// Superset of the spot-side estimate: it also reports gas price/limit and the
/// native-token price used for the conversion.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainWithdrawEstimateFee {
    /// Gas price on the destination chain.
    #[serde(default)]
    pub gas_price: Decimal,
    /// Gas limit required by the withdrawal.
    #[serde(default)]
    pub gas_limit: Decimal,
    /// Native-token price.
    #[serde(default)]
    pub native_price: Decimal,
    /// Token price used for fee conversion.
    pub token_price: Decimal,
    /// Minimum fee required for the withdrawal.
    pub gas_cost: Decimal,
    /// Fee value in USD.
    pub gas_usd_value: Decimal,
}

/// `POST /aster-chain/v3/{perp,spot}/wallet/transfer`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletTransferResult {
    /// Server-side transfer id.
    pub tran_id: String,
    /// Processing status.
    pub status: String,
}
