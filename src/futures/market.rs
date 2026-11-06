//! Futures market-data endpoints (`/fapi/v3/*`, unauthenticated).

use crate::client::{Endpoint, Method, Venue};
use crate::error::{Error, Result};
use crate::model::*;
use crate::params::Params;

macro_rules! public_get {
    ($(#[$meta:meta])* $name:ident, $path:literal, $response:ty) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, Default)]
        pub struct $name;

        impl Endpoint for $name {
            type Response = $response;
            const METHOD: Method = Method::Get;
            const PATH: &'static str = $path;
            const VENUE: Venue = Venue::Futures;
            const SECURITY: crate::client::Security = crate::client::Security::None;
        }
    };
}

public_get!(
    /// Connectivity probe.
    Ping,
    "/fapi/v3/ping",
    Empty
);
public_get!(
    /// Server time in milliseconds.
    ServerTime,
    "/fapi/v3/time",
    ServerTimePayload
);
public_get!(
    /// Trading rules, symbols, filters and rate limits.
    ExchangeInfo,
    "/fapi/v3/exchangeInfo",
    FuturesExchangeInfo
);

/// Order book snapshot. Valid limits: 5, 10, 20, 50, 100, 500, 1000.
#[derive(Debug, Clone)]
pub struct Depth {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Depth level count (5/10/20/50/100/500/1000).
    pub limit: Option<u32>,
}

impl Depth {
    /// Create a new value.
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            limit: None,
        }
    }
}

impl Endpoint for Depth {
    type Response = DepthPayload;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/depth";
    const VENUE: Venue = Venue::Futures;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        p.set("limit", &self.limit);
        Ok(p)
    }

    fn weight(&self) -> u32 {
        match self.limit.unwrap_or(500) {
            0..=50 => 2,
            100 => 5,
            500 => 10,
            _ => 20,
        }
    }
}

/// Recent market trades.
#[derive(Debug, Clone)]
pub struct Trades {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Maximum number of results.
    pub limit: Option<u32>,
}

impl Trades {
    /// Create a new value.
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            limit: None,
        }
    }
}

impl Endpoint for Trades {
    type Response = Vec<Trade>;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/trades";
    const VENUE: Venue = Venue::Futures;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        p.set("limit", &self.limit);
        Ok(p)
    }
}

/// Older market trades.
#[derive(Debug, Clone)]
pub struct HistoricalTrades {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Maximum number of results.
    pub limit: Option<u32>,
    /// Trade id to fetch from (defaults to the most recent).
    pub from_id: Option<i64>,
}

impl HistoricalTrades {
    /// Create a new value.
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            limit: None,
            from_id: None,
        }
    }
}

impl Endpoint for HistoricalTrades {
    type Response = Vec<Trade>;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/historicalTrades";
    const VENUE: Venue = Venue::Futures;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        p.set("limit", &self.limit);
        p.set("fromId", &self.from_id);
        Ok(p)
    }

    fn weight(&self) -> u32 {
        20
    }
}

/// Aggregate trades.
#[derive(Debug, Clone)]
pub struct AggTrades {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Trade id to fetch from (defaults to the most recent).
    pub from_id: Option<i64>,
    /// Start of the query window (inclusive, ms).
    pub start_time: Option<i64>,
    /// End of the query window (inclusive, ms).
    pub end_time: Option<i64>,
    /// Maximum number of results.
    pub limit: Option<u32>,
}

impl AggTrades {
    /// Create a new value.
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            from_id: None,
            start_time: None,
            end_time: None,
            limit: None,
        }
    }
}

impl Endpoint for AggTrades {
    type Response = Vec<AggTrade>;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/aggTrades";
    const VENUE: Venue = Venue::Futures;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        p.set("fromId", &self.from_id);
        p.set("startTime", &self.start_time);
        p.set("endTime", &self.end_time);
        p.set("limit", &self.limit);
        Ok(p)
    }

    fn weight(&self) -> u32 {
        20
    }
}

/// Kline/candlestick data.
#[derive(Debug, Clone)]
pub struct Klines {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Kline interval.
    pub interval: KlineInterval,
    /// Start of the query window (inclusive, ms).
    pub start_time: Option<i64>,
    /// End of the query window (inclusive, ms).
    pub end_time: Option<i64>,
    /// Maximum number of results.
    pub limit: Option<u32>,
}

impl Klines {
    /// Create a new value.
    pub fn new(symbol: impl Into<String>, interval: KlineInterval) -> Self {
        Self {
            symbol: symbol.into(),
            interval,
            start_time: None,
            end_time: None,
            limit: None,
        }
    }
}

impl Endpoint for Klines {
    type Response = Vec<Kline>;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/klines";
    const VENUE: Venue = Venue::Futures;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        p.set("interval", &self.interval.as_str());
        p.set("startTime", &self.start_time);
        p.set("endTime", &self.end_time);
        p.set("limit", &self.limit);
        Ok(p)
    }

    fn weight(&self) -> u32 {
        match self.limit.unwrap_or(500) {
            0..=99 => 1,
            100..=499 => 2,
            500..=999 => 5,
            _ => 10,
        }
    }
}

/// Index-price klines. Note: the vendor notes this path may not be live —
/// prefer [`MarketKlines`] until confirmed.
#[derive(Debug, Clone)]
pub struct IndexPriceKlines {
    /// Index pair.
    /// Index pair (e.g. `BTCUSDT`), sent as `pair`.
    pub pair: String,
    /// Kline interval.
    pub interval: KlineInterval,
    /// Start of the query window (inclusive, ms).
    pub start_time: Option<i64>,
    /// End of the query window (inclusive, ms).
    pub end_time: Option<i64>,
    /// Maximum number of results.
    pub limit: Option<u32>,
}

impl IndexPriceKlines {
    /// Create a new value.
    pub fn new(pair: impl Into<String>, interval: KlineInterval) -> Self {
        Self {
            pair: pair.into(),
            interval,
            start_time: None,
            end_time: None,
            limit: None,
        }
    }
}

impl Endpoint for IndexPriceKlines {
    type Response = Vec<Kline>;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/indexPriceKlines";
    const VENUE: Venue = Venue::Futures;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("pair", &self.pair);
        p.set("interval", &self.interval.as_str());
        p.set("startTime", &self.start_time);
        p.set("endTime", &self.end_time);
        p.set("limit", &self.limit);
        Ok(p)
    }
}

/// Mark-price klines. Note: the vendor notes this path may not be live —
/// prefer [`MarketKlines`] until confirmed.
#[derive(Debug, Clone)]
pub struct MarkPriceKlines {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Kline interval.
    pub interval: KlineInterval,
    /// Start of the query window (inclusive, ms).
    pub start_time: Option<i64>,
    /// End of the query window (inclusive, ms).
    pub end_time: Option<i64>,
    /// Maximum number of results.
    pub limit: Option<u32>,
}

impl MarkPriceKlines {
    /// Create a new value.
    pub fn new(symbol: impl Into<String>, interval: KlineInterval) -> Self {
        Self {
            symbol: symbol.into(),
            interval,
            start_time: None,
            end_time: None,
            limit: None,
        }
    }
}

impl Endpoint for MarkPriceKlines {
    type Response = Vec<Kline>;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/markPriceKlines";
    const VENUE: Venue = Venue::Futures;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        p.set("interval", &self.interval.as_str());
        p.set("startTime", &self.start_time);
        p.set("endTime", &self.end_time);
        p.set("limit", &self.limit);
        Ok(p)
    }
}

/// Market klines (the path the live server is documented to expose).
#[derive(Debug, Clone)]
pub struct MarketKlines {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Kline interval.
    pub interval: KlineInterval,
    /// Start of the query window (inclusive, ms).
    pub start_time: Option<i64>,
    /// End of the query window (inclusive, ms).
    pub end_time: Option<i64>,
    /// Maximum number of results.
    pub limit: Option<u32>,
}

impl MarketKlines {
    /// Create a new value.
    pub fn new(symbol: impl Into<String>, interval: KlineInterval) -> Self {
        Self {
            symbol: symbol.into(),
            interval,
            start_time: None,
            end_time: None,
            limit: None,
        }
    }
}

impl Endpoint for MarketKlines {
    type Response = Vec<Kline>;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/marketKlines";
    const VENUE: Venue = Venue::Futures;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        p.set("interval", &self.interval.as_str());
        p.set("startTime", &self.start_time);
        p.set("endTime", &self.end_time);
        p.set("limit", &self.limit);
        Ok(p)
    }
}

/// Mark price, index price and funding rate. Omit `symbol` for all symbols.
#[derive(Debug, Clone, Default)]
pub struct PremiumIndex {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: Option<String>,
}

impl Endpoint for PremiumIndex {
    type Response = OneOrMany<PremiumIndexPayload>;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/premiumIndex";
    const VENUE: Venue = Venue::Futures;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        Ok(p)
    }
}

/// Funding-rate history (ascending).
#[derive(Debug, Clone, Default)]
pub struct FundingRate {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: Option<String>,
    /// Start of the query window (inclusive, ms).
    pub start_time: Option<i64>,
    /// End of the query window (inclusive, ms).
    pub end_time: Option<i64>,
    /// Maximum number of results.
    pub limit: Option<u32>,
}

impl Endpoint for FundingRate {
    type Response = Vec<FundingRateRecord>;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/fundingRate";
    const VENUE: Venue = Venue::Futures;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        p.set("startTime", &self.start_time);
        p.set("endTime", &self.end_time);
        p.set("limit", &self.limit);
        Ok(p)
    }
}

/// Funding-rate configuration per symbol.
#[derive(Debug, Clone, Default)]
pub struct FundingInfo {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: Option<String>,
}

impl Endpoint for FundingInfo {
    type Response = Vec<FundingConfig>;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/fundingInfo";
    const VENUE: Venue = Venue::Futures;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        Ok(p)
    }
}

/// Component exchanges and weights of a symbol's index price.
#[derive(Debug, Clone)]
pub struct IndexReferences {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
}

impl Endpoint for IndexReferences {
    type Response = IndexReferencePayload;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/indexreferences";
    const VENUE: Venue = Venue::Futures;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        Ok(p)
    }
}

/// 24h rolling statistics. Omit `symbol` for all symbols (weight 40).
#[derive(Debug, Clone, Default)]
pub struct Ticker24hr {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: Option<String>,
}

impl Ticker24hr {
    /// trading pair, e.g. `BTCUSDT`.
    pub fn symbol(symbol: impl Into<String>) -> Self {
        Self {
            symbol: Some(symbol.into()),
        }
    }
}

impl Endpoint for Ticker24hr {
    type Response = OneOrMany<Ticker24h>;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/ticker/24hr";
    const VENUE: Venue = Venue::Futures;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        Ok(p)
    }

    fn weight(&self) -> u32 {
        if self.symbol.is_some() { 1 } else { 40 }
    }
}

/// Latest price. Omit `symbol` for all symbols.
#[derive(Debug, Clone, Default)]
pub struct PriceTicker {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: Option<String>,
}

impl PriceTicker {
    /// trading pair, e.g. `BTCUSDT`.
    pub fn symbol(symbol: impl Into<String>) -> Self {
        Self {
            symbol: Some(symbol.into()),
        }
    }
}

impl Endpoint for PriceTicker {
    type Response = OneOrMany<PriceTickerPayload>;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/ticker/price";
    const VENUE: Venue = Venue::Futures;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        Ok(p)
    }
}

/// Best bid/ask. Omit `symbol` for all symbols.
#[derive(Debug, Clone, Default)]
pub struct BookTicker {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: Option<String>,
}

impl BookTicker {
    /// trading pair, e.g. `BTCUSDT`.
    pub fn symbol(symbol: impl Into<String>) -> Self {
        Self {
            symbol: Some(symbol.into()),
        }
    }
}

impl Endpoint for BookTicker {
    type Response = OneOrMany<BookTickerPayload>;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/ticker/bookTicker";
    const VENUE: Venue = Venue::Futures;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        Ok(p)
    }
}

/// Remaining openable notional at a leverage tier; `"-1"` means uncapped.
#[derive(Debug, Clone)]
pub struct RemainingOpenableNotionalValue {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Leverage.
    pub leverage: u32,
}

impl Endpoint for RemainingOpenableNotionalValue {
    type Response = RemainingOpenableNotional;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/remainingOpenableNotionalValue";
    const VENUE: Venue = Venue::Futures;

    fn params(&self) -> Result<Params> {
        if self.leverage == 0 {
            return Err(Error::InvalidParam("leverage"));
        }
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        p.set("leverage", &self.leverage);
        Ok(p)
    }

    fn weight(&self) -> u32 {
        50
    }
}

// The public model names are reused, but a few response payloads need
// aliases so endpoint types can keep their natural names.
pub use crate::model::FundingInfo as FundingConfig;
pub use crate::model::IndexReferences as IndexReferencePayload;
pub use crate::model::PremiumIndex as PremiumIndexPayload;
pub use crate::model::ServerTime as ServerTimePayload;
pub use crate::model::{
    BookTicker as BookTickerPayload, Depth as DepthPayload, PriceTicker as PriceTickerPayload,
};
