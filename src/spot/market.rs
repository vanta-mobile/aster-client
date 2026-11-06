//! Spot market-data endpoints (`/api/v3/*`, unauthenticated).

use crate::client::{Endpoint, Method, Venue};
use crate::error::Result;
use crate::model::*;
use crate::params::Params;

macro_rules! spot_public_get {
    ($(#[$meta:meta])* $name:ident, $path:literal, $response:ty) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, Default)]
        pub struct $name;

        impl Endpoint for $name {
            type Response = $response;
            const METHOD: Method = Method::Get;
            const PATH: &'static str = $path;
            const VENUE: Venue = Venue::Spot;
        }
    };
}

spot_public_get!(
    /// Connectivity probe.
    Ping, "/api/v3/ping", Empty);
spot_public_get!(
    /// Server time in milliseconds.
    ServerTime, "/api/v3/time", ServerTimePayload);
spot_public_get!(
    /// Spot symbols, filters and rate limits.
    ExchangeInfo, "/api/v3/exchangeInfo", SpotExchangeInfo);

/// Order book snapshot.
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
    const PATH: &'static str = "/api/v3/depth";
    const VENUE: Venue = Venue::Spot;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        p.set("limit", &self.limit);
        Ok(p)
    }
}

/// Recent trades.
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
    const PATH: &'static str = "/api/v3/trades";
    const VENUE: Venue = Venue::Spot;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        p.set("limit", &self.limit);
        Ok(p)
    }
}

/// Historical trades (requires a valid market-data entitlement in practice).
#[derive(Debug, Clone)]
pub struct HistoricalTrades {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Maximum number of results.
    pub limit: Option<u32>,
    /// Trade id to fetch from (defaults to the most recent).
    pub from_id: Option<i64>,
}

impl Endpoint for HistoricalTrades {
    type Response = Vec<Trade>;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/api/v3/historicalTrades";
    const VENUE: Venue = Venue::Spot;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        p.set("limit", &self.limit);
        p.set("fromId", &self.from_id);
        Ok(p)
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
    const PATH: &'static str = "/api/v3/aggTrades";
    const VENUE: Venue = Venue::Spot;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        p.set("fromId", &self.from_id);
        p.set("startTime", &self.start_time);
        p.set("endTime", &self.end_time);
        p.set("limit", &self.limit);
        Ok(p)
    }
}

/// Kline/candlestick request.
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
    const PATH: &'static str = "/api/v3/klines";
    const VENUE: Venue = Venue::Spot;

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

/// 24h ticker. Omit `symbol` for all symbols.
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
    const PATH: &'static str = "/api/v3/ticker/24hr";
    const VENUE: Venue = Venue::Spot;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        Ok(p)
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
    const PATH: &'static str = "/api/v3/ticker/price";
    const VENUE: Venue = Venue::Spot;

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
    const PATH: &'static str = "/api/v3/ticker/bookTicker";
    const VENUE: Venue = Venue::Spot;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        Ok(p)
    }
}

/// Maker/taker fees for one symbol.
#[derive(Debug, Clone)]
pub struct CommissionRateQuery {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
}

impl Endpoint for CommissionRateQuery {
    type Response = CommissionRatePayload;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/api/v3/commissionRate";
    const VENUE: Venue = Venue::Spot;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        Ok(p)
    }
}

pub use crate::model::CommissionRate as CommissionRatePayload;
pub use crate::model::ServerTime as ServerTimePayload;
pub use crate::model::{
    BookTicker as BookTickerPayload, Depth as DepthPayload, PriceTicker as PriceTickerPayload,
};
