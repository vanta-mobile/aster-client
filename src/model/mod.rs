//! Domain models shared by the futures, spot and Aster-Chain APIs.

/// account.
pub mod account;
/// Aster-Chain models.
pub mod chain;
/// market.
pub mod market;
/// order payload.
pub mod order;
/// ws.
pub mod ws;

pub use account::*;
pub use chain::*;
pub use market::*;
pub use order::*;
pub use ws::*;

use serde::{Deserialize, Serialize};

/// `serverTime` payload of `GET .../time`.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerTime {
    /// Server time in milliseconds.
    pub server_time: i64,
}

/// Empty body or `{}` acknowledgement.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Empty {}

/// Endpoints that answer with one object when a symbol is given and an array
/// when it is omitted (`premiumIndex`, `ticker/*`).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    /// Single object.
    One(T),
    /// Array of objects.
    Many(Vec<T>),
}

impl<T> OneOrMany<T> {
    /// Flatten into a vector, cloning the single case.
    pub fn into_vec(self) -> Vec<T> {
        match self {
            OneOrMany::One(value) => vec![value],
            OneOrMany::Many(values) => values,
        }
    }

    /// Borrow the single value or the array as a slice.
    pub fn as_slice(&self) -> &[T] {
        match self {
            OneOrMany::One(value) => std::slice::from_ref(value),
            OneOrMany::Many(values) => values,
        }
    }
}

/// `{"code": 200, "msg": "success"}` acknowledgement used by several
/// management endpoints. Some endpoints return `code` as a string, so it is
/// decoded from either representation.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ack {
    /// Error or acknowledgement code.
    #[serde(deserialize_with = "i32_or_string")]
    pub code: i32,
    /// Message.
    #[serde(default)]
    pub msg: String,
}

fn i32_or_string<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Code {
        Number(i64),
        Text(String),
    }
    match Code::deserialize(deserializer)? {
        Code::Number(n) => Ok(n as i32),
        Code::Text(text) => text
            .parse()
            .map_err(|_| serde::de::Error::custom("code is not an integer")),
    }
}

/// `u32` from a JSON number or a numeric string.
pub(crate) fn u32_or_string<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Value {
        Number(u64),
        Text(String),
    }
    match Value::deserialize(deserializer)? {
        Value::Number(n) => u32::try_from(n).map_err(serde::de::Error::custom),
        Value::Text(text) => text
            .parse()
            .map_err(|_| serde::de::Error::custom("not an unsigned integer")),
    }
}

impl Ack {
    /// Whether the code is `200`.
    pub fn is_success(&self) -> bool {
        self.code == 200
    }
}

/// Aggregate trade (`GET .../aggTrades`, `@aggTrade`).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AggTrade {
    /// Aggregate trade id (`a`).
    #[serde(rename = "a")]
    pub id: i64,
    /// Aggregate trade id.
    /// Price (`p`).
    #[serde(rename = "p")]
    pub price: rust_decimal::Decimal,
    /// Price.
    /// Quantity (`q`).
    #[serde(rename = "q")]
    pub qty: rust_decimal::Decimal,
    /// Quantity.
    /// First trade id (`f`).
    #[serde(rename = "f")]
    pub first_trade_id: i64,
    /// First trade id in the aggregate.
    /// Last trade id (`l`).
    #[serde(rename = "l")]
    pub last_trade_id: i64,
    /// Last trade id in the aggregate.
    /// Trade time (`T`).
    #[serde(rename = "T")]
    pub time: i64,
    /// Whether the buyer was the maker.
    /// Timestamp in milliseconds.
    /// Buyer is maker (`m`).
    #[serde(rename = "m")]
    pub buyer_is_maker: bool,
}

/// One candle. Aster returns a 12-element array; this struct exposes it with
/// names, with the numeric fields kept as [`rust_decimal::Decimal`].
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(
    from = "(i64, rust_decimal::Decimal, rust_decimal::Decimal, rust_decimal::Decimal, rust_decimal::Decimal, rust_decimal::Decimal, i64, rust_decimal::Decimal, u32, rust_decimal::Decimal, rust_decimal::Decimal, String)"
)]
/// Kline update.
pub struct Kline {
    /// Open time in milliseconds.
    pub open_time: i64,
    /// Open price.
    pub open: rust_decimal::Decimal,
    /// High price.
    pub high: rust_decimal::Decimal,
    /// Low price.
    pub low: rust_decimal::Decimal,
    /// Close price.
    pub close: rust_decimal::Decimal,
    /// Base-asset volume.
    pub volume: rust_decimal::Decimal,
    /// Close time in milliseconds.
    pub close_time: i64,
    /// Quote-asset volume.
    pub quote_volume: rust_decimal::Decimal,
    /// Trade count.
    pub trades: u32,
    /// Taker buy volume in the base asset.
    pub taker_buy_base_volume: rust_decimal::Decimal,
    /// Taker buy volume in the quote asset.
    pub taker_buy_quote_volume: rust_decimal::Decimal,
    /// Unused trailing field returned by the server.
    pub ignore: String,
}

impl
    From<(
        i64,
        rust_decimal::Decimal,
        rust_decimal::Decimal,
        rust_decimal::Decimal,
        rust_decimal::Decimal,
        rust_decimal::Decimal,
        i64,
        rust_decimal::Decimal,
        u32,
        rust_decimal::Decimal,
        rust_decimal::Decimal,
        String,
    )> for Kline
{
    #[allow(clippy::type_complexity)]
    fn from(
        v: (
            i64,
            rust_decimal::Decimal,
            rust_decimal::Decimal,
            rust_decimal::Decimal,
            rust_decimal::Decimal,
            rust_decimal::Decimal,
            i64,
            rust_decimal::Decimal,
            u32,
            rust_decimal::Decimal,
            rust_decimal::Decimal,
            String,
        ),
    ) -> Self {
        Kline {
            open_time: v.0,
            open: v.1,
            high: v.2,
            low: v.3,
            close: v.4,
            volume: v.5,
            close_time: v.6,
            quote_volume: v.7,
            trades: v.8,
            taker_buy_base_volume: v.9,
            taker_buy_quote_volume: v.10,
            ignore: v.11,
        }
    }
}
