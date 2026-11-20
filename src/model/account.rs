//! Account, position, wallet and builder models.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::model::{Order, PositionSide};

/// One asset line of `GET /fapi/v3/balance`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    /// Account alias.
    #[serde(default)]
    pub account_alias: Option<String>,
    /// Asset.
    pub asset: String,
    /// Wallet balance.
    pub balance: Decimal,
    /// Cross wallet balance.
    pub cross_wallet_balance: Decimal,
    /// Unrealized PnL of cross positions.
    pub cross_un_pnl: Decimal,
    /// Available balance.
    pub available_balance: Decimal,
    /// Maximum amount that can be transferred out.
    pub max_withdraw_amount: Decimal,
    /// Whether the asset can be used as margin.
    #[serde(default)]
    pub margin_available: Option<bool>,
    /// Last update time in milliseconds.
    #[serde(default)]
    pub update_time: Option<i64>,
}

/// `GET /fapi/v3/account` / `accountWithJoinMargin`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    /// Commission tier.
    pub fee_tier: i32,
    /// Whether the account can trade.
    pub can_trade: bool,
    /// Whether the account can deposit.
    pub can_deposit: bool,
    /// Whether the account can withdraw.
    pub can_withdraw: bool,
    /// Last update time in milliseconds.
    #[serde(default)]
    pub update_time: Option<i64>,
    /// Total initial margin (USDT).
    #[serde(default)]
    pub total_initial_margin: Option<Decimal>,
    /// Total maintenance margin (USDT).
    #[serde(default)]
    pub total_maint_margin: Option<Decimal>,
    /// Total wallet balance (USDT).
    #[serde(default)]
    pub total_wallet_balance: Option<Decimal>,
    /// Total unrealized profit (USDT).
    #[serde(default)]
    pub total_unrealized_profit: Option<Decimal>,
    /// Total margin balance (USDT).
    #[serde(default)]
    pub total_margin_balance: Option<Decimal>,
    /// Initial margin of all positions (USDT).
    #[serde(default)]
    pub total_position_initial_margin: Option<Decimal>,
    /// Initial margin of open orders (USDT).
    #[serde(default)]
    pub total_open_order_initial_margin: Option<Decimal>,
    /// Cross wallet balance (USDT).
    #[serde(default)]
    pub total_cross_wallet_balance: Option<Decimal>,
    /// Unrealized PnL of cross positions (USDT).
    #[serde(default)]
    pub total_cross_un_pnl: Option<Decimal>,
    /// Available balance.
    #[serde(default)]
    pub available_balance: Option<Decimal>,
    /// Maximum amount that can be transferred out.
    #[serde(default)]
    pub max_withdraw_amount: Option<Decimal>,
    /// Per-asset account entries.
    #[serde(default)]
    pub assets: Vec<AccountAsset>,
    /// Positions.
    #[serde(default)]
    pub positions: Vec<AccountPosition>,
}

/// Per-asset entry of the futures account.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountAsset {
    /// Asset.
    pub asset: String,
    /// Wallet balance.
    pub wallet_balance: Decimal,
    /// Unrealized profit.
    #[serde(default)]
    pub unrealized_profit: Option<Decimal>,
    /// Margin balance.
    #[serde(default)]
    pub margin_balance: Option<Decimal>,
    /// Maintenance margin.
    #[serde(default)]
    pub maint_margin: Option<Decimal>,
    /// Initial margin.
    #[serde(default)]
    pub initial_margin: Option<Decimal>,
    /// Initial margin of the position.
    #[serde(default)]
    pub position_initial_margin: Option<Decimal>,
    /// Initial margin reserved for open orders.
    #[serde(default)]
    pub open_order_initial_margin: Option<Decimal>,
    /// Cross wallet balance.
    #[serde(default)]
    pub cross_wallet_balance: Option<Decimal>,
    /// Unrealized PnL of cross positions.
    #[serde(default)]
    pub cross_un_pnl: Option<Decimal>,
    /// Available balance.
    #[serde(default)]
    pub available_balance: Option<Decimal>,
    /// Maximum amount that can be transferred out.
    #[serde(default)]
    pub max_withdraw_amount: Option<Decimal>,
    /// Whether the asset can be used as margin.
    #[serde(default)]
    pub margin_available: Option<bool>,
    /// Last update time in milliseconds.
    #[serde(default)]
    pub update_time: Option<i64>,
}

/// Position entry of the futures account.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountPosition {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Initial margin.
    #[serde(default)]
    pub initial_margin: Option<Decimal>,
    /// Maintenance margin.
    #[serde(default)]
    pub maint_margin: Option<Decimal>,
    /// Unrealized profit.
    #[serde(default)]
    pub unrealized_profit: Option<Decimal>,
    /// Initial margin of the position.
    #[serde(default)]
    pub position_initial_margin: Option<Decimal>,
    /// Initial margin reserved for open orders.
    #[serde(default)]
    pub open_order_initial_margin: Option<Decimal>,
    /// Leverage.
    #[serde(default)]
    pub leverage: Option<Decimal>,
    /// Whether the position is isolated.
    #[serde(default)]
    pub isolated: Option<bool>,
    /// Average entry price.
    #[serde(default)]
    pub entry_price: Option<Decimal>,
    /// Maximum notional value.
    #[serde(default)]
    pub max_notional: Option<Decimal>,
    /// Position side (`BOTH` unless hedge mode).
    pub position_side: PositionSide,
    /// Position amount; negative for shorts.
    pub position_amt: Decimal,
    /// Notional value.
    #[serde(default)]
    pub notional: Option<Decimal>,
    /// Isolated wallet balance.
    #[serde(default)]
    pub isolated_wallet: Option<Decimal>,
    /// Last update time in milliseconds.
    #[serde(default)]
    pub update_time: Option<i64>,
}

/// Element of `GET /fapi/v3/positionRisk`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionRisk {
    /// Average entry price.
    pub entry_price: Decimal,
    /// Margin mode.
    #[serde(default)]
    pub margin_type: Option<String>,
    /// Whether auto-add-margin is enabled (string flag).
    #[serde(default)]
    pub is_auto_add_margin: Option<String>,
    /// Isolated margin.
    #[serde(default)]
    pub isolated_margin: Option<Decimal>,
    /// Leverage.
    pub leverage: Decimal,
    /// Liquidation price.
    pub liquidation_price: Decimal,
    /// Mark price.
    pub mark_price: Decimal,
    /// Maximum notional value.
    #[serde(default)]
    pub max_notional_value: Option<Decimal>,
    /// Position amount; negative for shorts.
    pub position_amt: Decimal,
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Unrealized profit.
    pub un_realized_profit: Decimal,
    /// Position side (`BOTH` unless hedge mode).
    pub position_side: PositionSide,
    /// Notional value.
    #[serde(default)]
    pub notional: Option<Decimal>,
    /// Isolated wallet balance.
    #[serde(default)]
    pub isolated_wallet: Option<Decimal>,
    /// Last update time in milliseconds.
    #[serde(default)]
    pub update_time: Option<i64>,
}

/// `GET /fapi/v3/leverageBrackets`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeverageBracket {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Leverage brackets.
    pub brackets: Vec<Bracket>,
}

/// One leverage bracket. Mainnet returns every field as a string
/// (`"initialLeverage":"200"`), so integers accept either representation.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bracket {
    /// Bracket number.
    #[serde(deserialize_with = "super::u32_or_string")]
    pub bracket: u32,
    /// Maximum leverage for the bracket.
    #[serde(deserialize_with = "super::u32_or_string")]
    pub initial_leverage: u32,
    /// Upper notional bound of the bracket.
    pub notional_cap: Decimal,
    /// Lower notional bound of the bracket.
    pub notional_floor: Decimal,
    /// Maintenance margin ratio.
    #[serde(default)]
    pub maint_margin_ratio: Option<Decimal>,
    /// Auxiliary value for maintenance-margin calculation.
    #[serde(default)]
    pub cum: Option<Decimal>,
}

/// `GET /fapi/v3/adlQuantile`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdlQuantile {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// ADL quantile per position side.
    pub adl_quantile: AdlQuantileLevels,
}

/// ADL quantiles per position side.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdlQuantileLevels {
    /// Long positions.
    #[serde(rename = "LONG", default)]
    pub long: Option<u32>,
    /// Short positions.
    #[serde(rename = "SHORT", default)]
    pub short: Option<u32>,
    /// One-way mode positions.
    #[serde(rename = "BOTH", default)]
    pub both: Option<u32>,
}

/// Element of `GET /fapi/v3/forceOrders`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ForceOrder {
    /// Exchange order id.
    pub order_id: i64,
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Order status.
    pub status: String,
    /// Client order id; the idempotency key for orders.
    pub client_order_id: String,
    /// Price.
    pub price: Decimal,
    /// Average fill price.
    pub avg_price: Decimal,
    /// Cumulative filled quote amount.
    pub cum_quote: Decimal,
    /// Cumulative filled quantity.
    pub executed_qty: Decimal,
    /// Timestamp in milliseconds.
    pub time: i64,
    /// Last update time in milliseconds.
    pub update_time: i64,
    /// Original order quantity.
    pub orig_qty: Decimal,
    /// Order type.
    #[serde(rename = "type")]
    pub order_type: String,
    /// Order side.
    pub side: String,
    /// Position side (`BOTH` unless hedge mode).
    pub position_side: String,
    /// How long the order stays active.
    pub time_in_force: String,
    /// Order type before any trigger.
    pub orig_type: String,
}

/// `GET /fapi/v3/commissionRate`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommissionRate {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Maker commission rate.
    pub maker_commission_rate: Decimal,
    /// Taker commission rate.
    pub taker_commission_rate: Decimal,
}

/// `GET /fapi/v3/income`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Income {
    /// Trading pair, e.g. `BTCUSDT`.
    #[serde(default)]
    pub symbol: String,
    /// Income type.
    pub income_type: String,
    /// Income amount.
    pub income: Decimal,
    /// Asset.
    pub asset: String,
    /// Extra information.
    #[serde(default)]
    pub info: String,
    /// Timestamp in milliseconds.
    pub time: i64,
    /// Transaction id.
    #[serde(default)]
    pub tran_id: Option<serde_json::Value>,
    /// Trade id.
    #[serde(default)]
    pub trade_id: String,
}

/// `GET /fapi/v3/incomePage`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomePage {
    /// Total number of matching records.
    pub total: u64,
    /// Page number, starting at 1.
    pub page: u64,
    /// Page size.
    pub limit: u64,
    /// Whether more pages exist.
    pub has_more: bool,
    /// Records on this page.
    pub rows: Vec<Income>,
}

/// Element of `GET /fapi/v3/positionMargin/history`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionMarginChange {
    /// Amount.
    pub amount: Decimal,
    /// Asset.
    pub asset: String,
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Timestamp in milliseconds.
    pub time: i64,
    /// `1` add margin, `2` reduce margin.
    #[serde(rename = "type")]
    pub change_type: i32,
    /// Position side (`BOTH` unless hedge mode).
    pub position_side: PositionSide,
    /// Change source (`TRADE` or `USER_ADJUST`).
    pub delta_type: String,
    /// Client transaction id; idempotency key for transfers.
    #[serde(default)]
    pub client_tran_id: String,
}

/// `POST /fapi/v3/positionMargin`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifyPositionMarginResult {
    /// Amount.
    pub amount: Decimal,
    /// Error or acknowledgement code.
    pub code: i32,
    /// Message.
    pub msg: String,
    /// `1` add margin, `2` reduce margin.
    #[serde(rename = "type")]
    pub change_type: i32,
}

/// `GET /fapi/v3/positionSide/dual`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DualSidePosition {
    /// `true` for hedge mode, `false` for one-way mode.
    pub dual_side_position: bool,
}

/// `GET /fapi/v3/stpMode`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StpModeResponse {
    /// Account-level self-trade prevention mode.
    pub stp_mode: String,
}

/// `GET /fapi/v3/multiAssetsMargin`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiAssetsMargin {
    /// Multi-assets margin mode.
    pub multi_assets_margin: bool,
}

/// `POST /fapi/v3/assetExchange`; an empty body means no exchange happened.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetExchangeResult {
    /// Account id.
    pub account_id: i64,
    /// Asset.
    pub asset: String,
    /// Wallet balance.
    pub balance: Decimal,
    /// Last update time in milliseconds.
    pub update_time: i64,
}

/// Spot `GET /api/v3/account`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotAccount {
    /// Commission tier.
    pub fee_tier: i32,
    /// Whether the account can trade.
    pub can_trade: bool,
    /// Whether the account can deposit.
    pub can_deposit: bool,
    /// Whether the account can withdraw.
    pub can_withdraw: bool,
    /// Whether the account can burn assets for fees.
    #[serde(default)]
    pub can_burn_asset: Option<bool>,
    /// Last update time in milliseconds.
    #[serde(default)]
    pub update_time: Option<i64>,
    /// Asset balances.
    pub balances: Vec<SpotBalance>,
}

/// Spot asset balance.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotBalance {
    /// Asset.
    pub asset: String,
    /// Free balance.
    pub free: Decimal,
    /// Balance locked by open orders.
    pub locked: Decimal,
}

/// `GET /api/v3/aster/withdraw/estimateFee`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawEstimateFee {
    /// Token price used for fee conversion.
    pub token_price: Decimal,
    /// Minimum fee required for the withdrawal.
    pub gas_cost: Decimal,
    /// Fee value in USD.
    pub gas_usd_value: Decimal,
}

/// `POST .../aster/user-withdraw`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawResult {
    /// Withdrawal id.
    pub withdraw_id: String,
    /// Transaction hash of the withdrawal.
    pub hash: String,
}

/// A deposit/withdrawal history record.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DepositWithdrawRecord {
    /// Record id.
    pub id: String,
    /// `DEPOSIT` or `WITHDRAW`.
    #[serde(rename = "type")]
    pub record_type: String,
    /// Asset.
    pub asset: String,
    /// Amount.
    pub amount: Decimal,
    /// Processing state.
    pub state: String,
    /// Transaction hash.
    #[serde(default)]
    pub tx_hash: Option<String>,
    /// Timestamp in milliseconds.
    pub time: i64,
    /// Chain id of the settlement network.
    pub chain_id: i64,
    /// Account type.
    pub account_type: String,
}

/// Agent wallet entry returned by `GET /fapi/v3/agent`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentInfo {
    /// Agent wallet address.
    #[serde(default)]
    pub agent_address: String,
    /// Agent display name.
    #[serde(default)]
    pub agent_name: String,
    /// Expiry timestamp in milliseconds.
    #[serde(default)]
    pub expired: Option<i64>,
    /// Whether the agent may place spot orders.
    #[serde(default)]
    pub can_spot_trade: Option<bool>,
    /// Whether the agent may place perpetual orders.
    #[serde(default)]
    pub can_perp_trade: Option<bool>,
    /// Whether the account can withdraw.
    #[serde(default)]
    pub can_withdraw: Option<bool>,
    /// How the agent was registered.
    #[serde(default)]
    pub source: Option<String>,
    /// Space-separated IPs/CIDRs; required when `canWithdraw` is true.
    #[serde(default)]
    pub ip_whitelist: Option<String>,
    /// Fields not modelled by this version.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Generic pagination envelope used by every `builder/*` query.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Paginated<T> {
    /// Total number of matching records.
    #[serde(default)]
    pub total: u64,
    /// Current page number.
    #[serde(default)]
    pub current_page: u64,
    /// Total number of pages.
    #[serde(default)]
    pub total_pages: u64,
    /// Effective page size.
    #[serde(default)]
    pub page_size: u64,
    /// Whether more pages exist.
    #[serde(default)]
    pub has_more: bool,
    /// Records on this page.
    pub rows: Vec<T>,
    /// Addresses that could not be returned, with the reason.
    #[serde(default)]
    pub errors: Vec<BuilderError>,
}

/// Address that could not be returned by a builder query.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuilderError {
    /// Wallet address.
    pub address: String,
    /// Why the address was left out of `rows`.
    pub error_msg: String,
}

/// Row of `GET /fapi/v3/builder/userAccounts`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuilderUserAccount {
    /// Wallet address.
    pub address: String,
    /// Commission tier.
    #[serde(default)]
    pub fee_tier: Option<i32>,
    /// Whether the account can trade.
    #[serde(default)]
    pub can_trade: Option<bool>,
    /// Whether the account can deposit.
    #[serde(default)]
    pub can_deposit: Option<bool>,
    /// Whether the account can withdraw.
    #[serde(default)]
    pub can_withdraw: Option<bool>,
    /// Last update time in milliseconds.
    #[serde(default)]
    pub update_time: Option<i64>,
    /// Account type.
    #[serde(default)]
    pub account_type: Option<i32>,
    /// `true` for hedge mode, `false` for one-way mode.
    #[serde(default)]
    pub dual_side_position: Option<bool>,
    /// Whether multi-assets margin is enabled.
    #[serde(default)]
    pub joint_margin: Option<bool>,
    /// Whether fee burn is enabled.
    #[serde(default)]
    pub fee_burn: Option<bool>,
    /// Fee-burn asset id.
    #[serde(default)]
    pub fee_burn_asset_id: Option<i64>,
    /// Per-symbol configuration.
    #[serde(default)]
    pub symbol_config: Vec<BuilderSymbolConfig>,
}

/// Per-symbol leverage in a builder account row.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuilderSymbolConfig {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Leverage.
    #[serde(default)]
    pub leverage: Option<u32>,
    /// Notional limit coefficient.
    #[serde(default)]
    pub notional_limit_coef: Option<Decimal>,
}

/// Row of `GET /fapi/v3/builder/userBalances`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuilderUserBalance {
    /// Wallet address.
    pub address: String,
    /// Asset.
    pub asset: String,
    /// Wallet balance.
    pub wallet_balance: Decimal,
    /// Price.
    #[serde(default)]
    pub price: Option<Decimal>,
    /// Balance converted to USD.
    #[serde(default)]
    pub balance_in_usd: Option<Decimal>,
}

/// Row of `GET /fapi/v3/builder/userPositionRisk`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuilderUserPositionRisk {
    /// Wallet address.
    pub address: String,
    /// Position payload.
    #[serde(flatten)]
    pub position: PositionRisk,
}

/// Row of `GET /fapi/v3/builder/userCommissionRates`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuilderUserCommissionRate {
    /// Wallet address.
    #[serde(default)]
    pub address: Option<String>,
    /// Trading pair, e.g. `BTCUSDT`.
    #[serde(default)]
    pub symbol: Option<String>,
    /// Maker commission rate.
    #[serde(default)]
    pub maker_commission_rate: Option<Decimal>,
    /// Taker commission rate.
    #[serde(default)]
    pub taker_commission_rate: Option<Decimal>,
}

/// Row of `GET /fapi/v3/builder/userOpenOrders` and `userAllOrders`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuilderOrder {
    /// Wallet address.
    pub address: String,
    /// Order payload.
    #[serde(flatten)]
    pub order: Order,
}

/// Row of `GET /fapi/v3/builder/userTrades`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuilderTrade {
    /// User wallet address.
    pub user_address: String,
    /// Trade id.
    pub trade_id: i64,
    /// Insertion time in milliseconds.
    pub insert_time: i64,
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Position side (`BOTH` unless hedge mode).
    pub position_side: Option<String>,
    /// Price.
    pub price: Decimal,
    /// Quantity.
    pub qty: Decimal,
    /// Base asset.
    #[serde(default)]
    pub base_asset: Option<String>,
    /// Symbol class, `CRYPTO` or `STOCK`.
    #[serde(default)]
    pub base_type: Option<String>,
    /// Quote asset.
    #[serde(default)]
    pub quote_asset: Option<String>,
    /// Order side.
    #[serde(default)]
    pub side: Option<String>,
    /// Whether the trade was an active buy.
    #[serde(default)]
    pub active_buy: Option<bool>,
    /// Commission asset.
    #[serde(default)]
    pub fee_asset: Option<String>,
    /// Notional value of the trade.
    #[serde(default)]
    pub total_quota: Option<Decimal>,
    /// Fee amount.
    #[serde(default)]
    pub fee: Option<Decimal>,
    /// Exchange order id.
    #[serde(default)]
    pub order_id: Option<i64>,
    /// Realized profit of the trade.
    #[serde(default)]
    pub realized_profit: Option<Decimal>,
    /// Settlement asset.
    #[serde(default)]
    pub margin_asset: Option<String>,
    /// Builder fee charged on the trade.
    #[serde(default)]
    pub builder_fee: Option<Decimal>,
}

/// Row of `GET /fapi/v3/builder/userCommissions`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuilderCommission {
    /// Trade id.
    pub trade_id: i64,
    /// Insertion time in milliseconds.
    pub insert_time: i64,
    /// Exchange order id.
    pub order_id: i64,
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Commission asset.
    pub fee_asset: String,
    /// Settlement asset.
    pub margin_asset: String,
    /// Builder fee charged on the trade.
    pub builder_fee: Decimal,
    /// User wallet address.
    pub user_address: String,
}

/// Row of `GET /fapi/v3/builder/approvedUserList`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovedUser {
    /// User wallet address.
    pub user_address: String,
    /// Builder wallet address.
    pub builder_address: String,
    /// Maximum fee rate granted to the builder.
    pub max_fee_rate: Decimal,
    /// Builder display name.
    pub builder_name: String,
    /// Approval time in milliseconds.
    pub approve_time: i64,
}

/// `POST /fapi/v3/leverage`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeverageChange {
    /// Leverage.
    pub leverage: u32,
    /// Maximum notional value.
    pub max_notional_value: Decimal,
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
}

/// Direct announcement (`GET /fapi/v3/announcement/direct`).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnouncementPage {
    /// Total number of matching records.
    #[serde(default)]
    pub total: u64,
    /// Records on this page.
    #[serde(default)]
    pub rows: Vec<Announcement>,
}

/// Announcement.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Announcement {
    /// Announcement id.
    pub id: i64,
    /// Localized announcement contents.
    #[serde(default)]
    pub contents: Vec<AnnouncementTranslation>,
    /// Announcement category.
    #[serde(default)]
    pub category: String,
    /// Publish time in milliseconds.
    #[serde(default)]
    pub publish_time: i64,
    /// Link to the full announcement.
    #[serde(default)]
    pub jump_link: String,
}

/// Localized announcement text.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnouncementTranslation {
    /// Language code.
    #[serde(default)]
    pub language: String,
    /// Title.
    #[serde(default)]
    pub title: String,
    /// Subtitle.
    #[serde(default)]
    pub subtitle: String,
    /// Announcement body.
    #[serde(default)]
    pub content: String,
}
