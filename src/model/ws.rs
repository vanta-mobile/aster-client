//! WebSocket payloads for market streams and user-data streams.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::model::{OrderStatus, PositionSide, Side, StpMode};

/// Combined-stream envelope: `{"stream": "...", "data": {...}}`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CombinedStream<T = Event> {
    /// Stream name.
    pub stream: String,
    /// Event payload.
    pub data: T,
}

/// Every event this client knows how to decode, tagged by `e`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "e")]
pub enum Event {
    /// Aggregate trade.
    #[serde(rename = "aggTrade")]
    AggTrade(AggTradeEvent),
    /// Mark price / funding rate update.
    #[serde(rename = "markPriceUpdate")]
    MarkPrice(MarkPriceEvent),
    /// Kline update.
    #[serde(rename = "kline")]
    Kline(KlineEvent),
    /// 24h mini ticker.
    #[serde(rename = "24hrMiniTicker")]
    MiniTicker(MiniTickerEvent),
    /// 24h ticker.
    #[serde(rename = "24hrTicker")]
    Ticker(TickerEvent),
    /// Best bid/ask update.
    #[serde(rename = "bookTicker")]
    BookTicker(BookTickerEvent),
    /// Liquidation order.
    #[serde(rename = "forceOrder")]
    ForceOrder(ForceOrderEvent),
    /// Diff depth update.
    #[serde(rename = "depthUpdate")]
    DepthUpdate(DepthUpdateEvent),
    /// TradePro trade.
    #[serde(rename = "tradepro")]
    TradePro(TradeProEvent),
    /// Balance/position update.
    #[serde(rename = "ACCOUNT_UPDATE")]
    AccountUpdate(AccountUpdateEvent),
    /// Order update.
    #[serde(rename = "ORDER_TRADE_UPDATE")]
    OrderTradeUpdate(OrderTradeUpdateEvent),
    /// Margin call notification.
    #[serde(rename = "MARGIN_CALL")]
    MarginCall(MarginCallEvent),
    /// Account configuration update.
    #[serde(rename = "ACCOUNT_CONFIG_UPDATE")]
    AccountConfigUpdate(AccountConfigUpdateEvent),
    /// `listenKey` expired.
    #[serde(rename = "listenKeyExpired")]
    ListenKeyExpired(ListenKeyExpired),
    /// Announcement.
    #[serde(rename = "announcement")]
    Announcement(AnnouncementEvent),
    /// Unknown.
    /// Announcement.
    /// A stream type this version does not model yet.
    #[serde(other)]
    Unknown,
}

impl Event {
    /// Event time in milliseconds, where the payload carries one.
    pub fn event_time(&self) -> Option<i64> {
        match self {
            Event::AggTrade(e) => Some(e.event_time),
            Event::MarkPrice(e) => Some(e.event_time),
            Event::Kline(e) => Some(e.event_time),
            Event::MiniTicker(e) => Some(e.event_time),
            Event::Ticker(e) => Some(e.event_time),
            Event::BookTicker(e) => Some(e.event_time),
            Event::DepthUpdate(e) => Some(e.event_time),
            Event::TradePro(e) => Some(e.event_time),
            Event::AccountUpdate(e) => Some(e.event_time),
            Event::OrderTradeUpdate(e) => Some(e.event_time),
            Event::MarginCall(e) => Some(e.event_time),
            Event::AccountConfigUpdate(e) => Some(e.event_time),
            Event::ListenKeyExpired(e) => Some(e.event_time),
            Event::ForceOrder(_) | Event::Announcement(_) | Event::Unknown => None,
        }
    }
}

/// Aggregate trade event.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AggTradeEvent {
    /// Event time in milliseconds.
    #[serde(rename = "E")]
    pub event_time: i64,
    /// Trading pair, e.g. `BTCUSDT`.
    #[serde(rename = "s")]
    pub symbol: String,
    /// Aggregate trade id.
    #[serde(rename = "a")]
    pub id: i64,
    /// Price.
    #[serde(rename = "p")]
    pub price: Decimal,
    /// Quantity.
    #[serde(rename = "q")]
    pub qty: Decimal,
    /// First trade id in the aggregate.
    #[serde(rename = "f")]
    pub first_trade_id: i64,
    /// Last trade id in the aggregate.
    #[serde(rename = "l")]
    pub last_trade_id: i64,
    /// Trade time in milliseconds.
    #[serde(rename = "T")]
    pub trade_time: i64,
    /// Whether the buyer was the maker.
    #[serde(rename = "m")]
    pub buyer_is_maker: bool,
}

/// Mark price and funding-rate update.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkPriceEvent {
    /// Event time in milliseconds.
    #[serde(rename = "E")]
    pub event_time: i64,
    /// Trading pair, e.g. `BTCUSDT`.
    #[serde(rename = "s")]
    pub symbol: String,
    /// Mark price.
    #[serde(rename = "p")]
    pub mark_price: Decimal,
    /// Index price.
    #[serde(rename = "i")]
    pub index_price: Decimal,
    /// Estimated settle price (last hour before settlement).
    #[serde(rename = "P", default)]
    pub estimated_settle_price: Option<Decimal>,
    /// Funding Rate.
    #[serde(rename = "r")]
    pub funding_rate: Decimal,
    /// Next funding time in milliseconds.
    #[serde(rename = "T")]
    pub next_funding_time: i64,
}

/// Kline event.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KlineEvent {
    /// Event time in milliseconds.
    #[serde(rename = "E")]
    pub event_time: i64,
    /// Trading pair, e.g. `BTCUSDT`.
    #[serde(rename = "s")]
    pub symbol: String,
    /// Candle payload.
    #[serde(rename = "k")]
    pub kline: KlinePayload,
}

/// Candle payload.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KlinePayload {
    /// Kline start time.
    #[serde(rename = "t")]
    pub open_time: i64,
    /// Close time in milliseconds.
    /// Open time in milliseconds.
    /// Kline close time.
    #[serde(rename = "T")]
    pub close_time: i64,
    /// Trading pair, e.g. `BTCUSDT`.
    #[serde(rename = "s")]
    pub symbol: String,
    /// Kline interval.
    #[serde(rename = "i")]
    pub interval: String,
    /// Open price.
    #[serde(rename = "o")]
    pub open: Decimal,
    /// Close price.
    #[serde(rename = "c")]
    pub close: Decimal,
    /// High price.
    #[serde(rename = "h")]
    pub high: Decimal,
    /// Low price.
    #[serde(rename = "l")]
    pub low: Decimal,
    /// Base-asset volume.
    #[serde(rename = "v")]
    pub volume: Decimal,
    /// Trade count.
    #[serde(rename = "n")]
    pub trades: u32,
    /// Whether the candle is closed.
    /// Trade count.
    /// Is this kline closed?
    #[serde(rename = "x")]
    pub closed: bool,
    /// Quote-asset volume.
    #[serde(rename = "q")]
    pub quote_volume: Decimal,
    /// Taker buy volume in the base asset.
    #[serde(rename = "V")]
    pub taker_buy_base_volume: Decimal,
    /// Taker buy volume in the quote asset.
    #[serde(rename = "Q")]
    pub taker_buy_quote_volume: Decimal,
}

/// Mini ticker event.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MiniTickerEvent {
    /// Event time in milliseconds.
    #[serde(rename = "E")]
    pub event_time: i64,
    /// Trading pair, e.g. `BTCUSDT`.
    #[serde(rename = "s")]
    pub symbol: String,
    /// Close price.
    #[serde(rename = "c")]
    pub close: Decimal,
    /// Open price.
    #[serde(rename = "o")]
    pub open: Decimal,
    /// High price.
    #[serde(rename = "h")]
    pub high: Decimal,
    /// Low price.
    #[serde(rename = "l")]
    pub low: Decimal,
    /// Base-asset volume.
    #[serde(rename = "v")]
    pub volume: Decimal,
    /// Quote-asset volume.
    #[serde(rename = "q")]
    pub quote_volume: Decimal,
}

/// 24h ticker event.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TickerEvent {
    /// Event time in milliseconds.
    #[serde(rename = "E")]
    pub event_time: i64,
    /// Trading pair, e.g. `BTCUSDT`.
    #[serde(rename = "s")]
    pub symbol: String,
    /// Absolute price change over the window.
    #[serde(rename = "p")]
    pub price_change: Decimal,
    /// Price change percentage over the window.
    #[serde(rename = "P")]
    pub price_change_percent: Decimal,
    /// Last traded price.
    #[serde(rename = "c")]
    pub last_price: Decimal,
    /// Last trade quantity.
    #[serde(rename = "Q", default)]
    pub last_qty: Option<Decimal>,
    /// Open price.
    #[serde(rename = "o")]
    pub open: Decimal,
    /// High price.
    #[serde(rename = "h")]
    pub high: Decimal,
    /// Low price.
    #[serde(rename = "l")]
    pub low: Decimal,
    /// Base-asset volume.
    #[serde(rename = "v")]
    pub volume: Decimal,
    /// Quote-asset volume.
    #[serde(rename = "q")]
    pub quote_volume: Decimal,
}

/// Best bid/ask event.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookTickerEvent {
    /// Event time in milliseconds.
    #[serde(rename = "E", default)]
    pub event_time: i64,
    /// Trading pair, e.g. `BTCUSDT`.
    #[serde(rename = "s")]
    pub symbol: String,
    /// Best bid price.
    #[serde(rename = "b")]
    pub bid_price: Decimal,
    /// Quantity at the best bid.
    #[serde(rename = "B")]
    pub bid_qty: Decimal,
    /// Best ask price.
    #[serde(rename = "a")]
    pub ask_price: Decimal,
    /// Quantity at the best ask.
    #[serde(rename = "A")]
    pub ask_qty: Decimal,
}

/// Liquidation order event.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ForceOrderEvent {
    /// Event time in milliseconds.
    #[serde(rename = "E")]
    pub event_time: i64,
    /// Order payload.
    #[serde(rename = "o")]
    pub order: ForceOrderPayload,
}

/// Liquidation order payload.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ForceOrderPayload {
    /// Trading pair, e.g. `BTCUSDT`.
    #[serde(rename = "s")]
    pub symbol: String,
    /// Order side.
    #[serde(rename = "S")]
    pub side: Side,
    /// Order type.
    #[serde(rename = "o")]
    pub order_type: String,
    /// Original quantity.
    #[serde(rename = "q")]
    pub original_qty: Decimal,
    /// Price.
    #[serde(rename = "p")]
    pub price: Decimal,
    /// Average fill price.
    #[serde(rename = "ap")]
    pub average_price: Decimal,
    /// Status reported by the server.
    #[serde(rename = "X")]
    pub status: OrderStatus,
    /// Trade time in milliseconds.
    #[serde(rename = "T")]
    pub trade_time: i64,
}

/// Diff book depth. Align with a REST snapshot using `U`/`u`/`pu`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DepthUpdateEvent {
    /// Event time in milliseconds.
    #[serde(rename = "E")]
    pub event_time: i64,
    /// Transaction time in milliseconds.
    /// Event time in milliseconds.
    #[serde(rename = "T")]
    pub transaction_time: i64,
    /// Trading pair, e.g. `BTCUSDT`.
    #[serde(rename = "s")]
    pub symbol: String,
    /// Trading pair, e.g. `BTCUSDT`.
    /// First update id in this event.
    #[serde(rename = "U")]
    pub first_update_id: u64,
    /// First update id covered by this event.
    /// Final update id in this event.
    #[serde(rename = "u")]
    pub final_update_id: u64,
    /// Last update id of the previous event.
    /// Last update id covered by this event.
    /// Last update id of the previous event.
    #[serde(rename = "pu")]
    pub previous_final_update_id: u64,
    /// Bid levels as `(price, quantity)` pairs.
    #[serde(rename = "b")]
    pub bids: Vec<(Decimal, Decimal)>,
    /// Ask levels as `(price, quantity)` pairs.
    #[serde(rename = "a")]
    pub asks: Vec<(Decimal, Decimal)>,
}

/// TradePro trade event.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeProEvent {
    /// Event time in milliseconds.
    #[serde(rename = "E")]
    pub event_time: i64,
    /// Transaction time in milliseconds.
    #[serde(rename = "T")]
    pub transaction_time: i64,
    /// Trading pair, e.g. `BTCUSDT`.
    #[serde(rename = "s")]
    pub symbol: String,
    /// Trade id.
    #[serde(rename = "t")]
    pub trade_id: i64,
    /// Price.
    #[serde(rename = "p")]
    pub price: Decimal,
    /// Quantity.
    #[serde(rename = "q")]
    pub qty: Decimal,
    /// Quantity.
    /// Transaction hash.
    #[serde(rename = "h")]
    pub tx_hash: String,
    /// Taker and maker addresses (hidden in privacy mode).
    /// Transaction hash.
    /// `[taker_address, maker_address]`, hidden when privacy mode is on.
    #[serde(rename = "m", default)]
    pub participants: Vec<String>,
}

/// `listenKey` expired.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListenKeyExpired {
    /// Event time in milliseconds.
    #[serde(rename = "E")]
    pub event_time: i64,
}

/// Margin call notification.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarginCallEvent {
    /// Event time in milliseconds.
    #[serde(rename = "E")]
    pub event_time: i64,
    /// Cross wallet balance.
    #[serde(rename = "cw", default)]
    pub cross_wallet_balance: Option<Decimal>,
    /// Positions.
    #[serde(rename = "p", default)]
    pub positions: Vec<MarginCallPosition>,
}

/// Position that triggered the margin call.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarginCallPosition {
    /// Trading pair, e.g. `BTCUSDT`.
    #[serde(rename = "s")]
    pub symbol: String,
    /// Position side (`BOTH` unless hedge mode).
    #[serde(rename = "ps")]
    pub position_side: PositionSide,
    /// Position amount.
    #[serde(rename = "pa")]
    pub position_amount: Decimal,
    /// Margin mode.
    #[serde(rename = "mt")]
    pub margin_type: String,
    /// Isolated wallet balance.
    #[serde(rename = "iw")]
    pub isolated_wallet: Decimal,
    /// Mark price.
    #[serde(rename = "mp")]
    pub mark_price: Decimal,
    /// Unrealized PnL.
    #[serde(rename = "up")]
    pub unrealized_pnl: Decimal,
    /// Maintenance margin required.
    #[serde(rename = "mm")]
    pub maintenance_margin: Decimal,
}

/// Balance or position changed (`ACCOUNT_UPDATE`).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountUpdateEvent {
    /// Event time in milliseconds.
    #[serde(rename = "E")]
    pub event_time: i64,
    /// Transaction time in milliseconds.
    #[serde(rename = "T")]
    pub transaction_time: i64,
    /// Balance/position update payload.
    #[serde(rename = "a")]
    pub update: AccountUpdate,
}

/// Balance/position update.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountUpdate {
    /// Event reason type.
    /// `DEPOSIT`, `WITHDRAW`, `ORDER`, `FUNDING_FEE`, `ASSET_TRANSFER`, ...
    #[serde(rename = "m")]
    pub reason: String,
    /// Asset balances.
    #[serde(rename = "B", default)]
    pub balances: Vec<BalanceUpdate>,
    /// Positions.
    #[serde(rename = "P", default)]
    pub positions: Vec<PositionUpdate>,
}

/// Balance change within an account update.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceUpdate {
    /// Asset.
    #[serde(rename = "a")]
    pub asset: String,
    /// Wallet balance.
    #[serde(rename = "wb")]
    pub wallet_balance: Decimal,
    /// Cross wallet balance.
    #[serde(rename = "cw", default)]
    pub cross_wallet_balance: Option<Decimal>,
    /// Balance change excluding PnL and commission.
    /// Cross wallet balance.
    /// Balance change excluding PnL and commission.
    #[serde(rename = "bc", default)]
    pub balance_change: Option<Decimal>,
}

/// Position change within an account update.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionUpdate {
    /// Trading pair, e.g. `BTCUSDT`.
    #[serde(rename = "s")]
    pub symbol: String,
    /// Position amount.
    #[serde(rename = "pa")]
    pub position_amount: Decimal,
    /// Average entry price.
    #[serde(rename = "ep")]
    pub entry_price: Decimal,
    /// Accumulated realized PnL before fees.
    #[serde(rename = "cr", default)]
    pub accumulated_realized: Option<Decimal>,
    /// Unrealized PnL.
    #[serde(rename = "up")]
    pub unrealized_pnl: Decimal,
    /// Margin mode.
    #[serde(rename = "mt")]
    pub margin_type: String,
    /// Isolated wallet balance.
    #[serde(rename = "iw")]
    pub isolated_wallet: Decimal,
    /// Position side (`BOTH` unless hedge mode).
    #[serde(rename = "ps")]
    pub position_side: PositionSide,
}

/// Order update event.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderTradeUpdateEvent {
    /// Event time in milliseconds.
    #[serde(rename = "E")]
    pub event_time: i64,
    /// Transaction time in milliseconds.
    #[serde(rename = "T")]
    pub transaction_time: i64,
    /// Order payload.
    #[serde(rename = "o")]
    pub order: OrderUpdate,
}

/// Order payload of `ORDER_TRADE_UPDATE`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderUpdate {
    /// Trading pair, e.g. `BTCUSDT`.
    #[serde(rename = "s")]
    pub symbol: String,
    /// Client order id; the idempotency key for orders.
    #[serde(rename = "c")]
    pub client_order_id: String,
    /// Order side.
    #[serde(rename = "S")]
    pub side: Side,
    /// Order type.
    #[serde(rename = "o")]
    pub order_type: String,
    /// How long the order stays active.
    #[serde(rename = "f")]
    pub time_in_force: String,
    /// Original quantity.
    #[serde(rename = "q")]
    pub original_qty: Decimal,
    /// Original price.
    #[serde(rename = "p")]
    pub original_price: Decimal,
    /// Average fill price.
    #[serde(rename = "ap")]
    pub average_price: Decimal,
    /// Trigger price for conditional orders.
    #[serde(rename = "sp")]
    pub stop_price: Decimal,
    /// Execution type of the order update.
    /// Trigger price for conditional orders.
    /// Execution type: `NEW`, `CANCELED`, `CALCULATED`, `EXPIRED`, `TRADE`.
    #[serde(rename = "x")]
    pub execution_type: String,
    /// Order status.
    #[serde(rename = "X")]
    pub status: OrderStatus,
    /// Exchange order id.
    #[serde(rename = "i")]
    pub order_id: i64,
    /// Quantity filled by this event.
    #[serde(rename = "l")]
    pub last_filled_qty: Decimal,
    /// Cumulative filled quantity.
    #[serde(rename = "z")]
    pub filled_accumulated_qty: Decimal,
    /// Price of this fill.
    #[serde(rename = "L")]
    pub last_filled_price: Decimal,
    /// Asset the commission was charged in.
    #[serde(rename = "N", default)]
    pub commission_asset: Option<String>,
    /// Commission charged.
    #[serde(rename = "n", default)]
    pub commission: Option<Decimal>,
    /// Trade time in milliseconds.
    #[serde(rename = "T")]
    pub trade_time: i64,
    /// Trade id.
    #[serde(rename = "t", default)]
    pub trade_id: Option<i64>,
    /// Whether the buyer was the maker.
    #[serde(rename = "m", default)]
    pub buyer_is_maker: Option<bool>,
    /// Whether the order may only reduce a position.
    #[serde(rename = "R", default)]
    pub reduce_only: Option<bool>,
    /// Price used to trigger the order.
    #[serde(rename = "wt", default)]
    pub working_type: Option<String>,
    /// Original Order Type.
    #[serde(rename = "ot", default)]
    pub original_order_type: Option<String>,
    /// Position side (`BOTH` unless hedge mode).
    #[serde(rename = "ps", default)]
    pub position_side: Option<PositionSide>,
    /// Whether this is a close-all order.
    #[serde(rename = "cp", default)]
    pub close_position: Option<bool>,
    /// Activation price of a trailing-stop order.
    #[serde(rename = "AP", default)]
    pub activation_price: Option<Decimal>,
    /// Trailing-stop callback rate, 1 = 1%.
    #[serde(rename = "cr", default)]
    pub callback_rate: Option<Decimal>,
    /// Realized profit of the trade.
    #[serde(rename = "rp", default)]
    pub realized_profit: Option<Decimal>,
    /// Account-level self-trade prevention mode.
    #[serde(rename = "stp", default)]
    pub stp_mode: Option<StpMode>,
}

/// Account configuration changed (`ACCOUNT_CONFIG_UPDATE`).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountConfigUpdateEvent {
    /// Event time in milliseconds.
    #[serde(rename = "E")]
    pub event_time: i64,
    /// Transaction time in milliseconds.
    #[serde(rename = "T")]
    pub transaction_time: i64,
    /// Transaction time in milliseconds.
    /// Per-symbol configuration (leverage change).
    #[serde(rename = "ac", default)]
    pub symbol_config: Option<SymbolConfigUpdate>,
    /// Account Config.
    /// Per-symbol configuration.
    /// Account-wide configuration (multi-assets / fee burn / position mode).
    #[serde(rename = "ai", default)]
    pub account_config: Option<AccountConfig>,
}

/// Per-symbol configuration change (leverage).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolConfigUpdate {
    /// Trading pair, e.g. `BTCUSDT`.
    #[serde(rename = "s")]
    pub symbol: String,
    /// Leverage.
    #[serde(rename = "l")]
    pub leverage: u32,
}

/// Account-wide configuration update.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountConfig {
    /// Multi-assets margin mode.
    #[serde(rename = "j", default)]
    pub multi_assets_margin: Option<bool>,
    /// Whether fee burn is enabled.
    #[serde(rename = "f", default)]
    pub fee_burn: Option<bool>,
    /// `true` for hedge mode, `false` for one-way mode.
    /// Whether fee burn is enabled.
    /// `true` for hedge mode, `false` for one-way mode.
    #[serde(rename = "d", default)]
    pub dual_side_position: Option<bool>,
}

/// Announcement event (public or direct).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnouncementEvent {
    /// English content.
    #[serde(default)]
    pub en: Option<AnnouncementContent>,
    /// Chinese content.
    #[serde(default)]
    pub zh: Option<AnnouncementContent>,
}

/// Localized announcement payload.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnouncementContent {
    /// Announcement id.
    #[serde(rename = "i")]
    pub id: i64,
    /// Title.
    #[serde(rename = "t")]
    pub title: String,
    /// Subtitle.
    #[serde(rename = "s", default)]
    pub subtitle: String,
    /// Announcement category.
    #[serde(rename = "c")]
    pub category: String,
    /// Timestamp in milliseconds.
    #[serde(rename = "d")]
    pub time: i64,
    /// Whether the announcement was read.
    #[serde(rename = "S", default)]
    pub read: bool,
    /// Link to the full announcement.
    #[serde(rename = "j", default)]
    pub link: String,
    /// Announcement body.
    #[serde(rename = "C", default)]
    pub content: String,
}
