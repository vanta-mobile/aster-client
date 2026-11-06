//! Market-data models: books, trades, tickers, klines, exchange rules.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// `GET .../depth`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Depth {
    /// Order-book update id.
    pub last_update_id: u64,
    /// Order-book update id.
    /// Message output time.
    #[serde(default, rename = "E")]
    pub event_time: Option<i64>,
    /// Transaction time in milliseconds.
    /// Event time in milliseconds.
    #[serde(default, rename = "T")]
    pub transaction_time: Option<i64>,
    /// Bid levels as `(price, quantity)` pairs.
    pub bids: Vec<(Decimal, Decimal)>,
    /// Ask levels as `(price, quantity)` pairs.
    pub asks: Vec<(Decimal, Decimal)>,
}

/// A public trade (`GET .../trades`, `GET .../historicalTrades`).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    /// Identifier.
    pub id: i64,
    /// Price.
    pub price: Decimal,
    /// Quantity.
    pub qty: Decimal,
    /// Quote-asset quantity.
    #[serde(default)]
    pub quote_qty: Option<Decimal>,
    /// Timestamp in milliseconds.
    pub time: i64,
    /// Whether the buyer was the maker.
    pub is_buyer_maker: bool,
}

/// `GET .../ticker/24hr` (single symbol or array).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ticker24h {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Absolute price change over the window.
    pub price_change: Decimal,
    /// Price change percentage over the window.
    pub price_change_percent: Decimal,
    /// Volume-weighted average price.
    #[serde(default)]
    pub weighted_avg_price: Option<Decimal>,
    /// Previous close price.
    #[serde(default)]
    pub prev_close_price: Option<Decimal>,
    /// Last traded price.
    pub last_price: Decimal,
    /// Last trade quantity.
    #[serde(default)]
    pub last_qty: Option<Decimal>,
    /// Price at the start of the window.
    #[serde(default)]
    pub open_price: Option<Decimal>,
    /// Highest price in the window.
    pub high_price: Decimal,
    /// Lowest price in the window.
    pub low_price: Decimal,
    /// Base-asset volume.
    pub volume: Decimal,
    /// Quote-asset volume.
    pub quote_volume: Decimal,
    /// Open time in milliseconds.
    pub open_time: i64,
    /// Close time in milliseconds.
    pub close_time: i64,
    /// First trade id in the window.
    #[serde(default)]
    pub first_id: Option<i64>,
    /// Last trade id in the window.
    #[serde(default)]
    pub last_id: Option<i64>,
    /// Number of trades.
    #[serde(default)]
    pub count: Option<u32>,
}

/// `GET .../ticker/price`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceTicker {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Price.
    pub price: Decimal,
    /// Timestamp in milliseconds.
    #[serde(default)]
    pub time: Option<i64>,
}

/// `GET .../ticker/bookTicker`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookTicker {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Best bid price.
    pub bid_price: Decimal,
    /// Quantity at the best bid.
    pub bid_qty: Decimal,
    /// Best ask price.
    pub ask_price: Decimal,
    /// Quantity at the best ask.
    pub ask_qty: Decimal,
    /// Timestamp in milliseconds.
    #[serde(default)]
    pub time: Option<i64>,
}

/// `GET /fapi/v3/premiumIndex`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PremiumIndex {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Mark price.
    pub mark_price: Decimal,
    /// Index price.
    pub index_price: Decimal,
    /// Estimated settle price (last hour before settlement).
    #[serde(default)]
    pub estimated_settle_price: Option<Decimal>,
    /// Last funding rate.
    #[serde(default)]
    pub last_funding_rate: Option<Decimal>,
    /// Next funding time in milliseconds.
    #[serde(default)]
    pub next_funding_time: Option<i64>,
    /// Interest-rate component of the funding rate.
    #[serde(default)]
    pub interest_rate: Option<Decimal>,
    /// Timestamp in milliseconds.
    #[serde(default)]
    pub time: Option<i64>,
}

/// Element of `GET /fapi/v3/fundingRate`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FundingRateRecord {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Funding Rate.
    pub funding_rate: Decimal,
    /// Funding Time.
    pub funding_time: i64,
}

/// Element of `GET /fapi/v3/fundingInfo`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FundingInfo {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Interest-rate component of the funding rate.
    pub interest_rate: Decimal,
    /// Timestamp in milliseconds.
    pub time: i64,
    /// Funding interval in hours.
    pub funding_interval_hours: u32,
    /// Funding fee cap.
    pub funding_fee_cap: Decimal,
    /// Funding fee floor.
    pub funding_fee_floor: Decimal,
}

/// `GET /fapi/v3/indexreferences`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexReferences {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Timestamp in milliseconds.
    pub time: i64,
    /// Index price sources.
    pub references: Vec<IndexReference>,
}

/// One index price source and its weight.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexReference {
    /// Source exchange.
    pub exchange: String,
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Documented request weight.
    pub weight: Decimal,
}

/// `GET /fapi/v3/remainingOpenableNotionalValue`; `"-1"` means "no cap".
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemainingOpenableNotional {
    /// Remaining Openable Notional Value.
    pub remaining_openable_notional_value: Decimal,
}

/// Rate-limit entry from `exchangeInfo.rateLimits`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimit {
    /// Rate-limit family.
    pub rate_limit_type: RateLimitType,
    /// Kline interval.
    pub interval: String,
    /// Interval multiplier.
    pub interval_num: u32,
    /// Maximum number of requests in the interval.
    pub limit: u32,
}

/// Rate-limit family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RateLimitType {
    /// Request-weight limiter.
    RequestWeight,
    /// Order-count limiter.
    Orders,
    /// Raw-request limiter.
    RawRequest,
    /// Value not modelled by this version.
    #[serde(other)]
    Unknown,
}

/// Asset entry of `exchangeInfo`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeAsset {
    /// Asset.
    pub asset: String,
    /// Whether the asset can be used as margin.
    #[serde(default)]
    pub margin_available: Option<bool>,
    /// Auto-exchange threshold in multi-assets mode.
    #[serde(default)]
    pub auto_asset_exchange: Option<Decimal>,
}

/// Trading rules for one symbol (shared shape of futures and spot
/// `exchangeInfo`).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolInfo {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Index pair.
    #[serde(default)]
    pub pair: Option<String>,
    /// Symbol status.
    pub status: SymbolStatus,
    /// Contract type.
    #[serde(default)]
    pub contract_type: Option<String>,
    /// Base asset.
    #[serde(default)]
    pub base_asset: String,
    /// Quote asset.
    #[serde(default)]
    pub quote_asset: String,
    /// Settlement asset.
    #[serde(default)]
    pub margin_asset: Option<String>,
    /// Advertised price precision (not `tickSize`).
    #[serde(default)]
    pub price_precision: Option<u32>,
    /// Advertised quantity precision (not `stepSize`).
    #[serde(default)]
    pub quantity_precision: Option<u32>,
    /// Advertised base-asset precision (not a tick/step).
    #[serde(default)]
    pub base_asset_precision: Option<u32>,
    /// Advertised quote precision (not a tick/step).
    #[serde(default)]
    pub quote_precision: Option<u32>,
    /// Underlying type; the API reports `COIN` for every symbol.
    #[serde(default)]
    pub underlying_type: Option<String>,
    /// Underlying sub-types.
    #[serde(default)]
    pub underlying_sub_type: Vec<String>,
    /// Price-protection threshold for conditional orders.
    #[serde(default)]
    pub trigger_protect: Option<Decimal>,
    /// Liquidation fee rate.
    #[serde(default)]
    pub liquidation_fee: Option<Decimal>,
    /// Maximum market-order deviation from mark price.
    #[serde(default)]
    pub market_take_bound: Option<Decimal>,
    /// Listing time in milliseconds.
    #[serde(default)]
    pub onboard_date: Option<i64>,
    /// Delivery time in milliseconds.
    #[serde(default)]
    pub delivery_date: Option<i64>,
    /// Whether OCO orders are allowed.
    #[serde(default)]
    pub oco_allowed: Option<bool>,
    /// Filters.
    pub filters: Vec<Filter>,
    /// Order types accepted by the symbol.
    pub order_types: Vec<OrderTypeName>,
    /// How long the order stays active.
    pub time_in_force: Vec<TimeInForceName>,
}

/// Trading status of a symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SymbolStatus {
    /// Awaiting listing.
    PendingTrading,
    /// Trading.
    Trading,
    /// Pre-settlement.
    PreSettle,
    /// Settling.
    Settling,
    /// Closed.
    Close,
    /// Value not modelled by this version.
    #[serde(other)]
    Unknown,
}

/// A parsed symbol filter. Unknown filter types are preserved as
/// [`Filter::Unknown`] so new server-side filters never break decoding.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(
    tag = "filterType",
    rename_all = "SCREAMING_SNAKE_CASE",
    rename_all_fields = "camelCase"
)]
/// A parsed symbol filter.
pub enum Filter {
    /// `PRICE_FILTER`: price step.
    PriceFilter {
        #[serde(default)]
        /// Price step.
        tick_size: Decimal,
        #[serde(default)]
        /// Minimum price.
        min_price: Option<Decimal>,
        #[serde(default)]
        /// Maximum price.
        max_price: Option<Decimal>,
    },
    /// `LOT_SIZE`: limit quantity step.
    LotSize {
        #[serde(default)]
        /// Quantity step.
        step_size: Decimal,
        #[serde(default)]
        /// Minimum quantity.
        min_qty: Option<Decimal>,
        #[serde(default)]
        /// Maximum quantity.
        max_qty: Option<Decimal>,
    },
    /// `MARKET_LOT_SIZE`: market quantity step.
    MarketLotSize {
        #[serde(default)]
        /// Quantity step.
        step_size: Decimal,
        #[serde(default)]
        /// Minimum quantity.
        min_qty: Option<Decimal>,
        #[serde(default)]
        /// Maximum quantity.
        max_qty: Option<Decimal>,
    },
    /// `MAX_NUM_ORDERS`: open-order cap.
    MaxNumOrders {
        #[serde(default)]
        /// Maximum number of open orders.
        limit: Option<u32>,
    },
    /// `MAX_NUM_ALGO_ORDERS`: conditional-order cap.
    MaxNumAlgoOrders {
        #[serde(default)]
        /// Maximum number of open orders.
        limit: Option<u32>,
    },
    /// `MIN_NOTIONAL`: minimum notional.
    MinNotional {
        #[serde(default)]
        /// Minimum notional value.
        notional: Option<Decimal>,
        #[serde(default)]
        /// Minimum notional value.
        min_notional: Option<Decimal>,
    },
    /// `MAX_NOTIONAL`: maximum notional.
    MaxNotional {
        #[serde(default)]
        /// Maximum notional value.
        max_notional: Option<Decimal>,
    },
    /// `NOTIONAL`: notional bounds and averaging window.
    Notional {
        #[serde(default)]
        /// Minimum notional value.
        min_notional: Option<Decimal>,
        #[serde(default)]
        /// Maximum notional value.
        max_notional: Option<Decimal>,
        #[serde(default)]
        /// Averaging window in minutes.
        avg_price_mins: Option<u32>,
        #[serde(default)]
        /// Whether the minimum applies to market orders.
        apply_min_to_market: Option<bool>,
        #[serde(default)]
        /// Whether the maximum applies to market orders.
        apply_max_to_market: Option<bool>,
    },
    /// `PERCENT_PRICE`: price deviation limits.
    PercentPrice {
        #[serde(default)]
        /// Maximum upward price multiplier.
        multiplier_up: Option<Decimal>,
        #[serde(default)]
        /// Maximum downward price multiplier.
        multiplier_down: Option<Decimal>,
        #[serde(default)]
        /// Decimal places used by the multiplier.
        multiplier_decimal: Option<Decimal>,
        #[serde(default)]
        /// Secondary upward multiplier.
        lt_multiplier_up: Option<Decimal>,
        #[serde(default)]
        /// Secondary downward multiplier.
        lt_multiplier_down: Option<Decimal>,
    },
    /// `PERCENT_PRICE_BY_SIDE`: side-specific deviation limits.
    PercentPriceBySide {
        #[serde(default)]
        /// Bid-side upward multiplier.
        bid_multiplier_up: Option<Decimal>,
        #[serde(default)]
        /// Ask-side upward multiplier.
        ask_multiplier_up: Option<Decimal>,
        #[serde(default)]
        /// Bid-side downward multiplier.
        bid_multiplier_down: Option<Decimal>,
        #[serde(default)]
        /// Ask-side downward multiplier.
        ask_multiplier_down: Option<Decimal>,
        #[serde(default)]
        /// Averaging window in minutes.
        avg_price_mins: Option<u32>,
        #[serde(default)]
        /// Decimal places used by the multiplier.
        multiplier_decimal: Option<Decimal>,
    },
    /// Value not modelled by this version.
    #[serde(other)]
    Unknown,
}

/// Order-type name as advertised in `exchangeInfo` (may include types the
/// typed order builder does not expose).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderTypeName {
    /// Limit order.
    Limit,
    /// Market order.
    Market,
    /// Stop-limit order.
    Stop,
    /// Stop-market order.
    StopMarket,
    /// Take-profit limit order.
    TakeProfit,
    /// Take-profit market order.
    TakeProfitMarket,
    /// Trailing stop-market order.
    TrailingStopMarket,
    /// Value not modelled by this version.
    #[serde(other)]
    Other,
}

/// Time-in-force name advertised by `exchangeInfo`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimeInForceName {
    /// Good till cancel.
    Gtc,
    /// Immediate or cancel.
    Ioc,
    /// Fill or kill (rejected by Aster order placement).
    Fok,
    /// Good till crossing (post only).
    Gtx,
    /// Hidden order.
    Hidden,
    /// Value not modelled by this version.
    #[serde(other)]
    Other,
}

/// Kline intervals accepted by `klines`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KlineInterval {
    /// 1 minute.
    #[serde(rename = "1m")]
    Min1,
    /// 3 minutes.
    #[serde(rename = "3m")]
    Min3,
    /// 5 minutes.
    #[serde(rename = "5m")]
    Min5,
    /// 15 minutes.
    #[serde(rename = "15m")]
    Min15,
    /// 30 minutes.
    #[serde(rename = "30m")]
    Min30,
    /// 1 hour.
    #[serde(rename = "1h")]
    Hour1,
    /// 2 hours.
    #[serde(rename = "2h")]
    Hour2,
    /// 4 hours.
    #[serde(rename = "4h")]
    Hour4,
    /// 6 hours.
    #[serde(rename = "6h")]
    Hour6,
    /// 8 hours.
    #[serde(rename = "8h")]
    Hour8,
    /// 12 hours.
    #[serde(rename = "12h")]
    Hour12,
    /// 1 day.
    #[serde(rename = "1d")]
    Day1,
    /// 3 days.
    #[serde(rename = "3d")]
    Day3,
    /// 1 week.
    #[serde(rename = "1w")]
    Week1,
    /// 1 month.
    #[serde(rename = "1M")]
    Month1,
}

impl KlineInterval {
    /// Wire representation of this value.
    pub fn as_str(self) -> &'static str {
        match self {
            KlineInterval::Min1 => "1m",
            KlineInterval::Min3 => "3m",
            KlineInterval::Min5 => "5m",
            KlineInterval::Min15 => "15m",
            KlineInterval::Min30 => "30m",
            KlineInterval::Hour1 => "1h",
            KlineInterval::Hour2 => "2h",
            KlineInterval::Hour4 => "4h",
            KlineInterval::Hour6 => "6h",
            KlineInterval::Hour8 => "8h",
            KlineInterval::Hour12 => "12h",
            KlineInterval::Day1 => "1d",
            KlineInterval::Day3 => "3d",
            KlineInterval::Week1 => "1w",
            KlineInterval::Month1 => "1M",
        }
    }
}

impl SymbolInfo {
    /// Look up a filter by kind.
    pub fn filter(&self, want: &FilterKind) -> Option<&Filter> {
        self.filters.iter().find(|f| f.kind() == *want)
    }

    /// `PRICE_FILTER.tickSize` — the only valid price step (never
    /// `pricePrecision`).
    pub fn tick_size(&self) -> Option<Decimal> {
        match self.filter(&FilterKind::PriceFilter) {
            Some(Filter::PriceFilter { tick_size, .. }) => Some(*tick_size),
            _ => None,
        }
    }

    /// `LOT_SIZE.stepSize` — limit-order quantity step.
    pub fn step_size(&self) -> Option<Decimal> {
        match self.filter(&FilterKind::LotSize) {
            Some(Filter::LotSize { step_size, .. }) => Some(*step_size),
            _ => None,
        }
    }

    /// `MARKET_LOT_SIZE.stepSize` — market-order quantity step.
    pub fn market_step_size(&self) -> Option<Decimal> {
        match self.filter(&FilterKind::MarketLotSize) {
            Some(Filter::MarketLotSize { step_size, .. }) => Some(*step_size),
            _ => None,
        }
    }

    /// Minimum notional, from `MIN_NOTIONAL.notional` or `NOTIONAL.minNotional`.
    pub fn min_notional(&self) -> Option<Decimal> {
        self.filters.iter().find_map(|filter| match filter {
            Filter::MinNotional {
                notional,
                min_notional,
            } => notional.or(*min_notional),
            Filter::Notional { min_notional, .. } => *min_notional,
            _ => None,
        })
    }

    /// maximum quantity.
    pub fn max_qty(&self) -> Option<Decimal> {
        match self.filter(&FilterKind::LotSize) {
            Some(Filter::LotSize { max_qty, .. }) => *max_qty,
            _ => None,
        }
    }

    /// Round a price **down** to the symbol's tick size.
    pub fn quantize_price(&self, price: Decimal) -> Option<Decimal> {
        self.tick_size().map(|tick| quantize_down(price, tick))
    }

    /// Round a quantity **down** to the symbol's step size.
    pub fn quantize_qty(&self, qty: Decimal) -> Option<Decimal> {
        self.step_size().map(|step| quantize_down(qty, step))
    }

    /// Whether the symbol is currently trading.
    pub fn is_trading(&self) -> bool {
        self.status == SymbolStatus::Trading
    }
}

/// Round `value` down to a multiple of `step` (used for tick/step alignment).
pub fn quantize_down(value: Decimal, step: Decimal) -> Decimal {
    if step.is_zero() {
        return value;
    }
    (value / step).trunc() * step
}

/// Filter kinds, used to look one up in [`SymbolInfo::filters`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterKind {
    /// `PRICE_FILTER`: price step.
    PriceFilter,
    /// `LOT_SIZE`: limit quantity step.
    LotSize,
    /// `MARKET_LOT_SIZE`: market quantity step.
    MarketLotSize,
    /// `MAX_NUM_ORDERS`: open-order cap.
    MaxNumOrders,
    /// `MAX_NUM_ALGO_ORDERS`: conditional-order cap.
    MaxNumAlgoOrders,
    /// `MIN_NOTIONAL`: minimum notional.
    MinNotional,
    /// `MAX_NOTIONAL`: maximum notional.
    MaxNotional,
    /// `NOTIONAL`: notional bounds and averaging window.
    Notional,
    /// `PERCENT_PRICE`: price deviation limits.
    PercentPrice,
    /// `PERCENT_PRICE_BY_SIDE`: side-specific deviation limits.
    PercentPriceBySide,
    /// Value not modelled by this version.
    Unknown,
}

impl Filter {
    /// transfer direction.
    pub fn kind(&self) -> FilterKind {
        match self {
            Filter::PriceFilter { .. } => FilterKind::PriceFilter,
            Filter::LotSize { .. } => FilterKind::LotSize,
            Filter::MarketLotSize { .. } => FilterKind::MarketLotSize,
            Filter::MaxNumOrders { .. } => FilterKind::MaxNumOrders,
            Filter::MaxNumAlgoOrders { .. } => FilterKind::MaxNumAlgoOrders,
            Filter::MinNotional { .. } => FilterKind::MinNotional,
            Filter::MaxNotional { .. } => FilterKind::MaxNotional,
            Filter::Notional { .. } => FilterKind::Notional,
            Filter::PercentPrice { .. } => FilterKind::PercentPrice,
            Filter::PercentPriceBySide { .. } => FilterKind::PercentPriceBySide,
            Filter::Unknown => FilterKind::Unknown,
        }
    }
}

/// Futures `GET /fapi/v3/exchangeInfo`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FuturesExchangeInfo {
    /// Futures margin type.
    #[serde(default)]
    pub futures_type: String,
    /// Exchange-wide filters.
    #[serde(default)]
    pub exchange_filters: Vec<serde_json::Value>,
    /// Rate limits applicable to this endpoint set.
    pub rate_limits: Vec<RateLimit>,
    /// Server time in milliseconds.
    #[serde(default)]
    pub server_time: Option<i64>,
    /// Per-asset account entries.
    pub assets: Vec<ExchangeAsset>,
    /// Trading rules per symbol.
    pub symbols: Vec<SymbolInfo>,
    /// Exchange timezone.
    #[serde(default)]
    pub timezone: String,
}

impl FuturesExchangeInfo {
    /// trading pair, e.g. `BTCUSDT`.
    pub fn symbol(&self, symbol: &str) -> Option<&SymbolInfo> {
        self.symbols.iter().find(|s| s.symbol == symbol)
    }
}

/// Spot `GET /api/v3/exchangeInfo`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotExchangeInfo {
    /// Exchange timezone.
    #[serde(default)]
    pub timezone: String,
    /// Server time in milliseconds.
    #[serde(default)]
    pub server_time: Option<i64>,
    /// Rate limits applicable to this endpoint set.
    pub rate_limits: Vec<RateLimit>,
    /// Exchange-wide filters.
    #[serde(default)]
    pub exchange_filters: Vec<serde_json::Value>,
    /// Per-asset account entries.
    pub assets: Vec<ExchangeAsset>,
    /// Trading rules per symbol.
    pub symbols: Vec<SymbolInfo>,
}

impl SpotExchangeInfo {
    /// trading pair, e.g. `BTCUSDT`.
    pub fn symbol(&self, symbol: &str) -> Option<&SymbolInfo> {
        self.symbols.iter().find(|s| s.symbol == symbol)
    }
}
