//! Order enums and order/trade response models.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

macro_rules! wire_enum {
    ($(#[$meta:meta])* $name:ident { $($variant:ident => $wire:literal),* $(,)? }) => {
        $(#[$meta])*
        // Variant names mirror the wire strings and are self-describing.
        #[allow(missing_docs)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum $name {
            $(#[serde(rename = $wire)] $variant),*
        }

        impl $name {
            /// Wire representation of this value.
            pub fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $wire),* }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl crate::params::Param for $name {
            fn write(&self, key: &'static str, out: &mut crate::params::Params) {
                out.set_str(key, self.as_str());
            }
        }
    };
}

wire_enum!(
    /// Order side.
    Side { Buy => "BUY", Sell => "SELL" }
);

wire_enum!(
    /// Order type.
    OrderType {
        Limit => "LIMIT",
        Market => "MARKET",
        Stop => "STOP",
        StopMarket => "STOP_MARKET",
        TakeProfit => "TAKE_PROFIT",
        TakeProfitMarket => "TAKE_PROFIT_MARKET",
        TrailingStopMarket => "TRAILING_STOP_MARKET",
    }
);

wire_enum!(
    /// Time in force. `FOK` is rejected by both spot and futures order
    /// placement (`-1115 INVALID_TIF`) and is not modelled.
    TimeInForce { Gtc => "GTC", Ioc => "IOC", Gtx => "GTX", Hidden => "HIDDEN" }
);

wire_enum!(
    /// Position side. `BOTH` in one-way mode, `LONG`/`SHORT` in hedge mode.
    PositionSide { Both => "BOTH", Long => "LONG", Short => "SHORT" }
);

wire_enum!(
    /// Price used to trigger conditional orders.
    WorkingType { MarkPrice => "MARK_PRICE", ContractPrice => "CONTRACT_PRICE" }
);

wire_enum!(
    /// `newOrderRespType`.
    NewOrderRespType { Ack => "ACK", Result => "RESULT", Full => "FULL" }
);

wire_enum!(
    /// Self-trade prevention mode (`stpMode`).
    StpMode { ExpireTaker => "EXPIRE_TAKER", ExpireMaker => "EXPIRE_MAKER", ExpireBoth => "EXPIRE_BOTH" }
);

wire_enum!(
    /// BBO peg mode for limit orders.
    PegPriceType {
        Counterparty1 => "COUNTERPARTY_1",
        Counterparty3 => "COUNTERPARTY_3",
        Counterparty5 => "COUNTERPARTY_5",
        Counterparty10 => "COUNTERPARTY_10",
        Counterparty20 => "COUNTERPARTY_20",
        Queue1 => "QUEUE_1",
        Queue3 => "QUEUE_3",
        Queue5 => "QUEUE_5",
        Queue10 => "QUEUE_10",
        Queue20 => "QUEUE_20",
    }
);

wire_enum!(
    /// Order status.
    OrderStatus {
        New => "NEW",
        PartiallyFilled => "PARTIALLY_FILLED",
        Filled => "FILLED",
        Canceled => "CANCELED",
        Rejected => "REJECTED",
        Expired => "EXPIRED",
    }
);

impl OrderStatus {
    /// Whether the order can still change.
    pub fn is_open(self) -> bool {
        matches!(self, OrderStatus::New | OrderStatus::PartiallyFilled)
    }

    /// Whether the order reached a final state.
    pub fn is_terminal(self) -> bool {
        !self.is_open()
    }
}

wire_enum!(
    /// Margin mode.
    MarginType { Isolated => "ISOLATED", Crossed => "CROSSED" }
);

wire_enum!(
    /// Wallet transfer direction.
    TransferKind { FutureSpot => "FUTURE_SPOT", SpotFuture => "SPOT_FUTURE" }
);

/// Futures order (new/modify/query/open/all responses share this shape).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    /// Average fill price.
    #[serde(default)]
    pub avg_price: Option<Decimal>,
    /// Client order id; the idempotency key for orders.
    pub client_order_id: String,
    /// Cumulative filled quote amount.
    #[serde(default)]
    pub cum_quote: Option<Decimal>,
    /// Cumulative filled quantity.
    pub executed_qty: Decimal,
    /// Exchange order id.
    pub order_id: i64,
    /// Original order quantity.
    pub orig_qty: Decimal,
    /// Order type before any trigger.
    #[serde(default)]
    pub orig_type: Option<OrderType>,
    /// Price.
    pub price: Decimal,
    /// Whether the order may only reduce a position.
    #[serde(default)]
    pub reduce_only: Option<bool>,
    /// Order side.
    pub side: Side,
    /// Position side (`BOTH` unless hedge mode).
    #[serde(default)]
    pub position_side: Option<PositionSide>,
    /// Order status.
    pub status: OrderStatus,
    /// Trigger price for conditional orders.
    #[serde(default)]
    pub stop_price: Option<Decimal>,
    /// Whether this is a close-all order.
    #[serde(default)]
    pub close_position: Option<bool>,
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Timestamp in milliseconds.
    #[serde(default)]
    pub time: Option<i64>,
    /// How long the order stays active.
    #[serde(default)]
    pub time_in_force: Option<TimeInForce>,
    /// Order type.
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// Activate Price.
    #[serde(default)]
    pub activate_price: Option<Decimal>,
    /// Trailing callback rate as returned by the server.
    #[serde(default)]
    pub price_rate: Option<Decimal>,
    /// Price used to trigger the order.
    #[serde(default)]
    pub working_type: Option<WorkingType>,
    /// Whether the conditional trigger is price-protected.
    #[serde(default)]
    pub price_protect: Option<bool>,
    /// BBO peg mode.
    #[serde(default)]
    pub peg_price_type: Option<String>,
    /// Account-level self-trade prevention mode.
    #[serde(default)]
    pub stp_mode: Option<String>,
    /// Last update time in milliseconds.
    #[serde(default)]
    pub update_time: Option<i64>,
}

/// Spot order (spot responses carry `orderListId` and omit position fields).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotOrder {
    /// Exchange order id.
    pub order_id: i64,
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Order status.
    pub status: OrderStatus,
    /// Client order id; the idempotency key for orders.
    pub client_order_id: String,
    /// Price.
    pub price: Decimal,
    /// Average fill price.
    pub avg_price: Decimal,
    /// Original order quantity.
    pub orig_qty: Decimal,
    /// Cumulative filled quantity.
    pub executed_qty: Decimal,
    /// Cumulative filled quote amount.
    #[serde(default)]
    pub cum_quote: Option<Decimal>,
    /// How long the order stays active.
    #[serde(default)]
    pub time_in_force: Option<TimeInForce>,
    /// Order type.
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// Trigger price for conditional orders.
    #[serde(default)]
    pub stop_price: Option<Decimal>,
    /// Order type before any trigger.
    #[serde(default)]
    pub orig_type: Option<OrderType>,
    /// Timestamp in milliseconds.
    #[serde(default)]
    pub time: Option<i64>,
    /// Last update time in milliseconds.
    #[serde(default)]
    pub update_time: Option<i64>,
    /// Order-list id (`-1` when unused).
    #[serde(default)]
    pub order_list_id: Option<i64>,
    /// Order side.
    #[serde(default)]
    pub side: Option<Side>,
}

/// Futures trade from `GET /fapi/v3/userTrades`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FuturesTrade {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Trade id.
    pub id: i64,
    /// Exchange order id.
    pub order_id: i64,
    /// Order side.
    pub side: Side,
    /// Price.
    pub price: Decimal,
    /// Quantity.
    pub qty: Decimal,
    /// Realized PnL of the trade.
    #[serde(default)]
    pub realized_pnl: Option<Decimal>,
    /// Settlement asset.
    #[serde(default)]
    pub margin_asset: Option<String>,
    /// Quote-asset quantity.
    #[serde(default)]
    pub quote_qty: Option<Decimal>,
    /// Commission charged.
    pub commission: Decimal,
    /// Asset the commission was charged in.
    pub commission_asset: String,
    /// Timestamp in milliseconds.
    pub time: i64,
    /// Position side (`BOTH` unless hedge mode).
    #[serde(default)]
    pub position_side: Option<PositionSide>,
    /// Whether the account was the buyer.
    #[serde(default)]
    pub buyer: Option<bool>,
    /// Whether the account was the maker.
    #[serde(default)]
    pub maker: Option<bool>,
}

/// Spot trade from `GET /api/v3/userTrades`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotTrade {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Trade id.
    pub id: i64,
    /// Exchange order id.
    pub order_id: i64,
    /// Order side.
    pub side: Option<Side>,
    /// Price.
    pub price: Decimal,
    /// Quantity.
    pub qty: Decimal,
    /// Quote-asset quantity.
    #[serde(default)]
    pub quote_qty: Option<Decimal>,
    /// Commission charged.
    pub commission: Decimal,
    /// Asset the commission was charged in.
    pub commission_asset: String,
    /// Timestamp in milliseconds.
    pub time: i64,
    /// Whether the account was the buyer.
    #[serde(default)]
    pub is_buyer: Option<bool>,
    /// Whether the account was the maker.
    #[serde(default)]
    pub is_maker: Option<bool>,
    /// Whether the trade was the best price match.
    #[serde(default)]
    pub is_best_match: Option<bool>,
}

/// Batch endpoints answer per element: either a result or an inline error.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum BatchItem<T> {
    /// Successful element.
    Ok(T),
    /// Inline element error.
    Err(crate::error::ApiError),
}

impl<T> BatchItem<T> {
    /// Discard the error and keep the success value.
    pub fn ok(self) -> Option<T> {
        match self {
            BatchItem::Ok(value) => Some(value),
            BatchItem::Err(_) => None,
        }
    }

    /// Convert into a `Result`, mapping an inline element error.
    pub fn as_result(self) -> Result<T, crate::error::ApiError> {
        match self {
            BatchItem::Ok(value) => Ok(value),
            BatchItem::Err(error) => Err(error),
        }
    }
}

/// `POST .../asset/wallet/transfer` response.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferResult {
    /// Transaction id.
    pub tran_id: i64,
    /// Transfer status.
    pub status: String,
}

/// `POST /fapi/v3/chase`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChaseOrderResult {
    /// Exchange strategy order id.
    pub strategy_id: i64,
    /// Client-assigned strategy id.
    #[serde(default)]
    pub client_strategy_id: Option<String>,
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Order side.
    pub side: Side,
    /// Position side (`BOTH` unless hedge mode).
    #[serde(default)]
    pub position_side: Option<PositionSide>,
    /// Order quantity, in the base asset.
    pub quantity: Decimal,
    /// Quantity Unit.
    #[serde(default)]
    pub quantity_unit: Option<String>,
    /// Whether the order may only reduce a position.
    #[serde(default)]
    pub reduce_only: Option<bool>,
    /// Chase Offset.
    #[serde(default)]
    pub chase_offset: Option<Decimal>,
    /// Chase Offset Type.
    #[serde(default)]
    pub chase_offset_type: Option<String>,
    /// Max Chase Offset.
    #[serde(default)]
    pub max_chase_offset: Option<Decimal>,
    /// Max Chase Offset Type.
    #[serde(default)]
    pub max_chase_offset_type: Option<String>,
    /// How long the order stays active.
    #[serde(default)]
    pub time_in_force: Option<TimeInForce>,
    /// Strategy status reported by the server.
    pub strategy_status: String,
    /// Time the strategy was booked (ms).
    #[serde(default)]
    pub book_time: Option<i64>,
    /// Last update time in milliseconds.
    #[serde(default)]
    pub update_time: Option<i64>,
}

wire_enum!(
    /// Strategy order family.
    StrategyType { Oto => "OTO", Oco => "OCO", Otoco => "OTOCO" }
);

wire_enum!(
    /// Security type of a strategy leg.
    SecurityType { UsdtFutures => "USDT_FUTURES", CoinFutures => "COIN_FUTURES", Options => "OPTIONS" }
);

/// `POST /fapi/v3/placeStrategyOrder`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyOrderResult {
    /// Exchange strategy order id.
    pub strategy_id: i64,
    /// Client-assigned strategy id.
    #[serde(default)]
    pub client_strategy_id: Option<String>,
    /// Strategy family (`OTO`, `OCO`, `OTOCO`).
    pub strategy_type: String,
    /// Strategy status reported by the server.
    pub strategy_status: String,
    /// Last update time in milliseconds.
    #[serde(default)]
    pub update_time: Option<i64>,
    /// Failure code; `0` means success (may arrive with HTTP 200).
    /// Last update time in milliseconds.
    /// `0` on success; non-zero failures arrive with HTTP 200.
    #[serde(default)]
    pub failure_code: i32,
    /// Human-readable failure reason.
    #[serde(default)]
    pub failure_reason: String,
}

impl StrategyOrderResult {
    /// A rejected BBO leg answers HTTP 200 with `failureCode` set — always
    /// check this instead of the HTTP status.
    pub fn rejection(&self) -> Option<crate::error::ApiError> {
        if self.failure_code == 0 {
            None
        } else {
            Some(crate::error::ApiError::new(
                self.failure_code,
                self.failure_reason.clone(),
            ))
        }
    }
}

/// One element of `POST /fapi/v3/updateStrategyOrder`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyUpdateResult {
    /// Exchange strategy order id.
    pub strategy_id: i64,
    /// Client-assigned strategy id.
    #[serde(default)]
    pub client_strategy_id: Option<String>,
    /// Strategy family (`OTO`, `OCO`, `OTOCO`).
    pub strategy_type: String,
    /// Strategy status reported by the server.
    pub strategy_status: String,
    /// Leg that was updated.
    #[serde(default)]
    pub updated_sub_order: Option<u32>,
    /// Per-leg update status.
    #[serde(default)]
    pub update_status: Option<String>,
    /// Last update time in milliseconds.
    #[serde(default)]
    pub update_time: Option<i64>,
    /// Failure code; `0` means success (may arrive with HTTP 200).
    #[serde(default)]
    pub failure_code: i32,
    /// Human-readable failure reason.
    #[serde(default)]
    pub failure_reason: String,
}

impl StrategyUpdateResult {
    /// `Some` error when `failureCode` is non-zero (these failures arrive with HTTP 200).
    pub fn rejection(&self) -> Option<crate::error::ApiError> {
        if self.failure_code == 0 {
            None
        } else {
            Some(crate::error::ApiError::new(
                self.failure_code,
                self.failure_reason.clone(),
            ))
        }
    }
}

/// Strategy order returned by `strategyOpenOrder` / `strategyHistoryOrder`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyOrder {
    /// Exchange strategy order id.
    pub strategy_id: i64,
    /// Client-assigned strategy id.
    #[serde(default)]
    pub client_strategy_id: Option<String>,
    /// Strategy family (`OTO`, `OCO`, `OTOCO`).
    pub strategy_type: String,
    /// Strategy status reported by the server.
    pub strategy_status: String,
    /// Time the strategy was booked (ms).
    #[serde(default)]
    pub book_time: Option<i64>,
    /// Last update time in milliseconds.
    #[serde(default)]
    pub update_time: Option<i64>,
    /// Strategy legs.
    #[serde(default)]
    pub sub_orders: Vec<StrategySubOrder>,
}

/// One strategy leg as returned by the server.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategySubOrder {
    /// Exchange strategy order id.
    pub strategy_id: i64,
    /// Exchange order id.
    #[serde(default)]
    pub order_id: Option<i64>,
    /// Client order id; the idempotency key for orders.
    #[serde(default)]
    pub client_order_id: Option<String>,
    /// Leg status.
    pub status: String,
    /// Leg sequence number, starting at 1.
    #[serde(default)]
    pub strategy_sub_id: Option<u32>,
    /// Driving leg for the first trigger.
    #[serde(default)]
    pub first_driven_id: Option<serde_json::Value>,
    /// Event that activates the first trigger.
    #[serde(default)]
    pub first_driven_on: Option<String>,
    /// First trigger action.
    #[serde(default)]
    pub first_trigger: Option<String>,
    /// Driving leg for the second trigger.
    #[serde(default)]
    pub second_driven_id: Option<serde_json::Value>,
    /// Event that activates the second trigger.
    #[serde(default)]
    pub second_driven_on: Option<String>,
    /// Second trigger action.
    #[serde(default)]
    pub second_trigger: Option<String>,
    /// Security type of the strategy leg.
    #[serde(default)]
    pub security_type: Option<String>,
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Order side.
    pub side: Side,
    /// Position side (`BOTH` unless hedge mode).
    #[serde(default)]
    pub position_side: Option<PositionSide>,
    /// Order type.
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// How long the order stays active.
    #[serde(default)]
    pub time_in_force: Option<TimeInForce>,
    /// Order quantity, in the base asset.
    pub quantity: Decimal,
    /// Whether the order may only reduce a position.
    #[serde(default)]
    pub reduce_only: Option<bool>,
    /// Whether this is a close-all order.
    #[serde(default)]
    pub close_position: Option<bool>,
    /// Price.
    pub price: Decimal,
    /// Average fill price.
    #[serde(default)]
    pub avg_price: Option<Decimal>,
    /// Whether the conditional trigger is price-protected.
    #[serde(default)]
    pub price_protect: Option<bool>,
    /// Trigger price for conditional orders.
    #[serde(default)]
    pub stop_price: Option<Decimal>,
    /// Activate Price.
    #[serde(default)]
    pub activate_price: Option<Decimal>,
    /// Trailing-stop callback rate, 1 = 1%.
    #[serde(default)]
    pub callback_rate: Option<Decimal>,
    /// Price used to trigger the order.
    #[serde(default)]
    pub working_type: Option<WorkingType>,
    /// Time the conditional order fired (ms).
    #[serde(default)]
    pub trigger_time: Option<i64>,
    /// BBO peg mode.
    #[serde(default)]
    pub peg_price_type: Option<String>,
}

/// Client order ids must match `^[\.A-Z\:/a-z0-9_-]{1,36}$`.
pub fn valid_client_order_id(id: &str) -> bool {
    (1..=36).contains(&id.len())
        && id
            .bytes()
            .all(|b| matches!(b, b'.' | b'A'..=b'Z' | b':' | b'/' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-'))
}

/// `POST /fapi/v3/countdownCancelAll`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CountdownCancelAllResult {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Countdown in milliseconds; `0` stops the timer.
    pub countdown_time: i64,
}
