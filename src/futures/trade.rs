//! Futures trading endpoints (orders, batch orders, strategies, listen keys).

use rust_decimal::Decimal;
use serde::Serialize;

use crate::client::{Endpoint, Method, Security, Venue};
use crate::error::{Error, Result};
use crate::model::*;
use crate::params::Params;

/// A new futures order.
///
/// Construct it through the order-type constructors: they fix `type` and the
/// parameters the exchange requires for that type. [`Endpoint::params`] then
/// rejects combinations the server would answer with
/// `PARAM_NOT_REQUIRED`/`-1102`.
///
/// ```
/// use aster_client::futures::NewOrder;
/// use aster_client::model::Side;
///
/// let order = NewOrder::limit("AAPLUSDT", Side::Buy, "1.5", "200.25")?
///     .time_in_force(aster_client::model::TimeInForce::Gtc)
///     .client_order_id("a1b2");
/// assert_eq!(order.symbol, "AAPLUSDT");
/// # Ok::<(), aster_client::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct NewOrder {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Order side.
    pub side: Side,
    /// Order type.
    pub order_type: OrderType,
    /// Position side (`BOTH` unless hedge mode).
    pub position_side: Option<PositionSide>,
    /// How long the order stays active.
    pub time_in_force: Option<TimeInForce>,
    /// Order quantity, in the base asset.
    pub quantity: Option<Decimal>,
    /// Whether the order may only reduce a position.
    pub reduce_only: Option<bool>,
    /// Price.
    pub price: Option<Decimal>,
    /// Client order id; the idempotency key for orders.
    pub client_order_id: Option<String>,
    /// Trigger price for conditional orders.
    pub stop_price: Option<Decimal>,
    /// Whether this is a close-all order.
    pub close_position: Option<bool>,
    /// Activation price of a trailing-stop order.
    pub activation_price: Option<Decimal>,
    /// Trailing-stop callback rate, 1 = 1%.
    pub callback_rate: Option<Decimal>,
    /// Price used to trigger the order.
    pub working_type: Option<WorkingType>,
    /// Whether the conditional trigger is price-protected.
    pub price_protect: Option<bool>,
    /// Response type requested for the order.
    pub new_order_resp_type: Option<NewOrderRespType>,
    /// BBO peg mode.
    pub peg_price_type: Option<PegPriceType>,
    /// Account-level self-trade prevention mode.
    pub stp_mode: Option<StpMode>,
    /// Builder.
    /// Account-level self-trade prevention mode.
    /// Aster Code builder address; requires `fee_rate`.
    pub builder: Option<String>,
    /// Builder fee rate; mandatory together with `builder`.
    pub fee_rate: Option<Decimal>,
    /// Master wallet address.
    /// Builder fee rate; mandatory together with `builder`.
    /// Master wallet address for builder-attributed orders.
    pub user: Option<String>,
}

/// Parse a wire decimal (accepts `Decimal`, integers and strings).
fn decimal(value: impl TryInto<rust_decimal::Decimal>) -> Result<rust_decimal::Decimal> {
    value.try_into().map_err(|_| Error::InvalidParam("decimal"))
}

impl NewOrder {
    fn base(symbol: impl Into<String>, side: Side, order_type: OrderType) -> Self {
        Self {
            symbol: symbol.into(),
            side,
            order_type,
            position_side: None,
            time_in_force: None,
            quantity: None,
            reduce_only: None,
            price: None,
            client_order_id: None,
            stop_price: None,
            close_position: None,
            activation_price: None,
            callback_rate: None,
            working_type: None,
            price_protect: None,
            new_order_resp_type: None,
            peg_price_type: None,
            stp_mode: None,
            builder: None,
            fee_rate: None,
            user: None,
        }
    }

    /// `LIMIT`: requires `timeInForce`, `quantity`, `price`.
    pub fn limit(
        symbol: impl Into<String>,
        side: Side,
        quantity: impl TryInto<Decimal>,
        price: impl TryInto<Decimal>,
    ) -> Result<Self> {
        let mut order = Self::base(symbol, side, OrderType::Limit);
        order.quantity = Some(decimal(quantity)?);
        order.price = Some(decimal(price)?);
        Ok(order)
    }

    /// `MARKET`: requires `quantity`.
    pub fn market(
        symbol: impl Into<String>,
        side: Side,
        quantity: impl TryInto<Decimal>,
    ) -> Result<Self> {
        let mut order = Self::base(symbol, side, OrderType::Market);
        order.quantity = Some(decimal(quantity)?);
        Ok(order)
    }

    /// `STOP`: requires `quantity`, `price`, `stopPrice`.
    pub fn stop(
        symbol: impl Into<String>,
        side: Side,
        quantity: impl TryInto<Decimal>,
        price: impl TryInto<Decimal>,
        stop_price: impl TryInto<Decimal>,
    ) -> Result<Self> {
        let mut order = Self::base(symbol, side, OrderType::Stop);
        order.quantity = Some(decimal(quantity)?);
        order.price = Some(decimal(price)?);
        order.stop_price = Some(decimal(stop_price)?);
        Ok(order)
    }

    /// `TAKE_PROFIT`: requires `quantity`, `price`, `stopPrice`.
    pub fn take_profit(
        symbol: impl Into<String>,
        side: Side,
        quantity: impl TryInto<Decimal>,
        price: impl TryInto<Decimal>,
        stop_price: impl TryInto<Decimal>,
    ) -> Result<Self> {
        let mut order = Self::base(symbol, side, OrderType::TakeProfit);
        order.quantity = Some(decimal(quantity)?);
        order.price = Some(decimal(price)?);
        order.stop_price = Some(decimal(stop_price)?);
        Ok(order)
    }

    /// `STOP_MARKET`: requires `stopPrice` (and `quantity` unless close-all).
    pub fn stop_market(
        symbol: impl Into<String>,
        side: Side,
        stop_price: impl TryInto<Decimal>,
    ) -> Result<Self> {
        let mut order = Self::base(symbol, side, OrderType::StopMarket);
        order.stop_price = Some(decimal(stop_price)?);
        Ok(order)
    }

    /// `TAKE_PROFIT_MARKET`: requires `stopPrice`.
    pub fn take_profit_market(
        symbol: impl Into<String>,
        side: Side,
        stop_price: impl TryInto<Decimal>,
    ) -> Result<Self> {
        let mut order = Self::base(symbol, side, OrderType::TakeProfitMarket);
        order.stop_price = Some(decimal(stop_price)?);
        Ok(order)
    }

    /// `TRAILING_STOP_MARKET`: requires `callbackRate` (0.1–5, 1 = 1%).
    pub fn trailing_stop_market(
        symbol: impl Into<String>,
        side: Side,
        callback_rate: impl TryInto<Decimal>,
    ) -> Result<Self> {
        let mut order = Self::base(symbol, side, OrderType::TrailingStopMarket);
        order.callback_rate = Some(decimal(callback_rate)?);
        Ok(order)
    }

    /// how long the order stays active.
    pub fn time_in_force(mut self, time_in_force: TimeInForce) -> Self {
        self.time_in_force = Some(time_in_force);
        self
    }

    /// order quantity, in the base asset.
    pub fn quantity(mut self, quantity: impl Into<Decimal>) -> Self {
        self.quantity = Some(quantity.into());
        self
    }

    /// whether the order may only reduce a position.
    pub fn reduce_only(mut self, reduce_only: bool) -> Self {
        self.reduce_only = Some(reduce_only);
        self
    }

    /// position side (`BOTH` unless hedge mode).
    pub fn position_side(mut self, position_side: PositionSide) -> Self {
        self.position_side = Some(position_side);
        self
    }

    /// Idempotency key (`^[\.A-Z\:/a-z0-9_-]{1,36}$`). Reuse it when
    /// reconciling an HTTP 503.
    pub fn client_order_id(mut self, id: impl Into<String>) -> Self {
        self.client_order_id = Some(id.into());
        self
    }

    /// Close-all for `STOP_MARKET`/`TAKE_PROFIT_MARKET`.
    pub fn close_position(mut self, close_position: bool) -> Self {
        self.close_position = Some(close_position);
        self
    }

    /// activation price of a trailing-stop order.
    pub fn activation_price(mut self, price: impl Into<Decimal>) -> Self {
        self.activation_price = Some(price.into());
        self
    }

    /// price used to trigger the order.
    pub fn working_type(mut self, working_type: WorkingType) -> Self {
        self.working_type = Some(working_type);
        self
    }

    /// whether the conditional trigger is price-protected.
    pub fn price_protect(mut self, price_protect: bool) -> Self {
        self.price_protect = Some(price_protect);
        self
    }

    /// Set the requested order response type.
    pub fn resp_type(mut self, resp_type: NewOrderRespType) -> Self {
        self.new_order_resp_type = Some(resp_type);
        self
    }

    /// bBO peg mode.
    pub fn peg_price_type(mut self, peg: PegPriceType) -> Self {
        self.peg_price_type = Some(peg);
        self
    }

    /// account-level self-trade prevention mode.
    pub fn stp_mode(mut self, stp_mode: StpMode) -> Self {
        self.stp_mode = Some(stp_mode);
        self
    }

    /// Builder attribution plus fee rate: both are mandatory together.
    pub fn builder_fee(
        mut self,
        builder: impl Into<String>,
        fee_rate: impl TryInto<Decimal>,
    ) -> Result<Self> {
        self.builder = Some(builder.into());
        self.fee_rate = Some(decimal(fee_rate)?);
        Ok(self)
    }

    /// master wallet address.
    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    fn validate(&self) -> Result<()> {
        use OrderType::*;
        let close_all = self.close_position == Some(true);

        match self.order_type {
            Limit => {
                if self.time_in_force.is_none() || self.quantity.is_none() || self.price.is_none() {
                    return Err(Error::InvalidParam("timeInForce/quantity/price"));
                }
                if self.stop_price.is_some() {
                    return Err(Error::InvalidParam("stopPrice"));
                }
            }
            Market => {
                if self.quantity.is_none() {
                    return Err(Error::InvalidParam("quantity"));
                }
                if self.time_in_force.is_some() || self.price.is_some() || self.stop_price.is_some()
                {
                    return Err(Error::InvalidParam("timeInForce/price/stopPrice"));
                }
            }
            Stop | TakeProfit => {
                if self.quantity.is_none() || self.price.is_none() || self.stop_price.is_none() {
                    return Err(Error::InvalidParam("quantity/price/stopPrice"));
                }
            }
            StopMarket | TakeProfitMarket => {
                if self.stop_price.is_none() {
                    return Err(Error::InvalidParam("stopPrice"));
                }
                if self.price.is_some() {
                    return Err(Error::InvalidParam("price"));
                }
                if !close_all && self.quantity.is_none() {
                    return Err(Error::InvalidParam("quantity"));
                }
            }
            TrailingStopMarket => {
                if self.callback_rate.is_none() {
                    return Err(Error::InvalidParam("callbackRate"));
                }
                if self.price.is_some() || self.stop_price.is_some() {
                    return Err(Error::InvalidParam("price/stopPrice"));
                }
                if let Some(rate) = self.callback_rate {
                    if rate < Decimal::new(1, 1) || rate > Decimal::from(5) {
                        return Err(Error::InvalidParam("callbackRate"));
                    }
                }
            }
        }

        if close_all {
            if !matches!(self.order_type, StopMarket | TakeProfitMarket) {
                return Err(Error::InvalidParam("closePosition"));
            }
            if self.quantity.is_some() {
                return Err(Error::InvalidParam("quantity"));
            }
            if self.reduce_only == Some(true) {
                return Err(Error::InvalidParam("reduceOnly"));
            }
        }

        if self.activation_price.is_some() && self.order_type != TrailingStopMarket {
            return Err(Error::InvalidParam("activationPrice"));
        }

        if self.builder.is_some() != self.fee_rate.is_some() {
            return Err(Error::InvalidParam("builder/feeRate"));
        }

        if let Some(id) = &self.client_order_id {
            if !valid_client_order_id(id) {
                return Err(Error::InvalidParam("newClientOrderId"));
            }
        }

        Ok(())
    }
}

impl Endpoint for NewOrder {
    type Response = Order;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/fapi/v3/order";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        self.validate()?;
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        p.set("side", &self.side);
        p.set("type", &self.order_type);
        p.set("positionSide", &self.position_side);
        p.set("timeInForce", &self.time_in_force);
        p.set("quantity", &self.quantity);
        p.set("reduceOnly", &self.reduce_only);
        p.set("price", &self.price);
        p.set("newClientOrderId", &self.client_order_id);
        p.set("stopPrice", &self.stop_price);
        p.set("closePosition", &self.close_position);
        p.set("activationPrice", &self.activation_price);
        p.set("callbackRate", &self.callback_rate);
        p.set("workingType", &self.working_type);
        if let Some(price_protect) = self.price_protect {
            p.set_str("priceProtect", if price_protect { "TRUE" } else { "FALSE" });
        }
        p.set("newOrderRespType", &self.new_order_resp_type);
        p.set("pegPriceType", &self.peg_price_type);
        p.set("stpMode", &self.stp_mode);
        p.set("builder", &self.builder);
        p.set("feeRate", &self.fee_rate);
        p.set("user", &self.user);
        Ok(p)
    }
}

/// `PUT /fapi/v3/order` — replace price/quantity of a resting `LIMIT` order.
#[derive(Debug, Clone)]
pub struct ModifyOrder {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Order side.
    pub side: Option<Side>,
    /// Exchange order id.
    pub order_id: Option<i64>,
    /// Client id of the existing order.
    pub orig_client_order_id: Option<String>,
    /// Order quantity, in the base asset.
    pub quantity: Decimal,
    /// Price.
    pub price: Decimal,
}

impl ModifyOrder {
    /// Create a new value.
    pub fn new(
        symbol: impl Into<String>,
        quantity: impl Into<Decimal>,
        price: impl Into<Decimal>,
    ) -> Self {
        Self {
            symbol: symbol.into(),
            side: None,
            order_id: None,
            orig_client_order_id: None,
            quantity: quantity.into(),
            price: price.into(),
        }
    }

    /// exchange order id.
    pub fn order_id(mut self, order_id: i64) -> Self {
        self.order_id = Some(order_id);
        self
    }

    /// client order id; the idempotency key for orders.
    pub fn client_order_id(mut self, id: impl Into<String>) -> Self {
        self.orig_client_order_id = Some(id.into());
        self
    }

    /// order side.
    pub fn side(mut self, side: Side) -> Self {
        self.side = Some(side);
        self
    }
}

impl Endpoint for ModifyOrder {
    type Response = Order;
    const METHOD: Method = Method::Put;
    const PATH: &'static str = "/fapi/v3/order";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        if self.order_id.is_none() && self.orig_client_order_id.is_none() {
            return Err(Error::InvalidParam("orderId/origClientOrderId"));
        }
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        p.set("side", &self.side);
        p.set("orderId", &self.order_id);
        p.set("origClientOrderId", &self.orig_client_order_id);
        p.set("quantity", &self.quantity);
        p.set("price", &self.price);
        Ok(p)
    }
}

/// `DELETE /fapi/v3/order`.
#[derive(Debug, Clone)]
pub struct CancelOrder {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Exchange order id.
    pub order_id: Option<i64>,
    /// Client id of the existing order.
    pub orig_client_order_id: Option<String>,
}

impl CancelOrder {
    /// Target the order by exchange order id.
    pub fn by_id(symbol: impl Into<String>, order_id: i64) -> Self {
        Self {
            symbol: symbol.into(),
            order_id: Some(order_id),
            orig_client_order_id: None,
        }
    }

    /// Target the order by client order id.
    pub fn by_client_id(symbol: impl Into<String>, client_order_id: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            order_id: None,
            orig_client_order_id: Some(client_order_id.into()),
        }
    }
}

impl Endpoint for CancelOrder {
    type Response = Order;
    const METHOD: Method = Method::Delete;
    const PATH: &'static str = "/fapi/v3/order";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        if self.order_id.is_none() && self.orig_client_order_id.is_none() {
            return Err(Error::InvalidParam("orderId/origClientOrderId"));
        }
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        p.set("orderId", &self.order_id);
        p.set("origClientOrderId", &self.orig_client_order_id);
        Ok(p)
    }
}

/// `DELETE /fapi/v3/guardedCancelOrder` — signs with the **order's** nonce.
#[derive(Debug, Clone)]
pub struct GuardedCancelOrder {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Exchange order id.
    pub order_id: Option<i64>,
    /// Client id of the existing order.
    pub orig_client_order_id: Option<String>,
    /// Request nonce: microsecond timestamp, strictly increasing per signer.
    /// Client id of the existing order.
    /// The `nonce` used when the order was placed.
    pub nonce: u64,
}

impl Endpoint for GuardedCancelOrder {
    type Response = Order;
    const METHOD: Method = Method::Delete;
    const PATH: &'static str = "/fapi/v3/guardedCancelOrder";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        if self.order_id.is_none() && self.orig_client_order_id.is_none() {
            return Err(Error::InvalidParam("orderId/origClientOrderId"));
        }
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        p.set("orderId", &self.order_id);
        p.set("origClientOrderId", &self.orig_client_order_id);
        Ok(p)
    }

    fn nonce_override(&self) -> Option<u64> {
        Some(self.nonce)
    }
}

/// `DELETE /fapi/v3/allOpenOrders`.
#[derive(Debug, Clone)]
pub struct CancelAllOpenOrders {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
}

impl Endpoint for CancelAllOpenOrders {
    type Response = Ack;
    const METHOD: Method = Method::Delete;
    const PATH: &'static str = "/fapi/v3/allOpenOrders";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        Ok(p)
    }
}

/// `POST /fapi/v3/batchOrders` (max 5 orders).
#[derive(Debug, Clone)]
pub struct BatchNewOrders {
    /// Orders in the batch.
    pub orders: Vec<NewOrder>,
}

impl Endpoint for BatchNewOrders {
    type Response = Vec<BatchItem<Order>>;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/fapi/v3/batchOrders";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        if self.orders.is_empty() || self.orders.len() > 5 {
            return Err(Error::InvalidParam("batchOrders"));
        }
        let mut p = Params::new();
        p.set_str("batchOrders", json_array(&self.orders)?);
        Ok(p)
    }

    fn weight(&self) -> u32 {
        5
    }
}

/// `PUT /fapi/v3/batchOrders` (max 5 modifications).
#[derive(Debug, Clone)]
pub struct BatchModifyOrders {
    /// Orders in the batch.
    pub orders: Vec<ModifyOrder>,
}

impl Endpoint for BatchModifyOrders {
    type Response = Vec<BatchItem<Order>>;
    const METHOD: Method = Method::Put;
    const PATH: &'static str = "/fapi/v3/batchOrders";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        if self.orders.is_empty() || self.orders.len() > 5 {
            return Err(Error::InvalidParam("batchOrders"));
        }
        let mut p = Params::new();
        p.set_str("batchOrders", json_array(&self.orders)?);
        Ok(p)
    }

    fn weight(&self) -> u32 {
        5
    }
}

/// `DELETE /fapi/v3/batchOrders` (max 10 ids).
#[derive(Debug, Clone)]
pub struct BatchCancelOrders {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Exchange order ids.
    pub order_ids: Vec<i64>,
    /// Client order ids.
    pub client_order_ids: Vec<String>,
}

impl Endpoint for BatchCancelOrders {
    type Response = Vec<BatchItem<Order>>;
    const METHOD: Method = Method::Delete;
    const PATH: &'static str = "/fapi/v3/batchOrders";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        if !self.order_ids.is_empty() {
            p.set_str("orderIdList", serde_json::to_string(&self.order_ids)?);
        } else if !self.client_order_ids.is_empty() {
            p.set_str(
                "origClientOrderIdList",
                serde_json::to_string(&self.client_order_ids)?,
            );
        } else {
            return Err(Error::InvalidParam("orderIdList/origClientOrderIdList"));
        }
        Ok(p)
    }
}

/// `DELETE /fapi/v3/guardedBatchOrders` — signs with the batch's nonce.
#[derive(Debug, Clone)]
pub struct GuardedBatchCancelOrders {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Exchange order ids.
    pub order_ids: Vec<i64>,
    /// Client order ids.
    pub client_order_ids: Vec<String>,
    /// Request nonce: microsecond timestamp, strictly increasing per signer.
    pub nonce: u64,
}

impl Endpoint for GuardedBatchCancelOrders {
    type Response = Vec<BatchItem<Order>>;
    const METHOD: Method = Method::Delete;
    const PATH: &'static str = "/fapi/v3/guardedBatchOrders";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        if !self.order_ids.is_empty() {
            p.set_str("orderIdList", serde_json::to_string(&self.order_ids)?);
        } else if !self.client_order_ids.is_empty() {
            p.set_str(
                "origClientOrderIdList",
                serde_json::to_string(&self.client_order_ids)?,
            );
        } else {
            return Err(Error::InvalidParam("orderIdList/origClientOrderIdList"));
        }
        Ok(p)
    }

    fn nonce_override(&self) -> Option<u64> {
        Some(self.nonce)
    }
}

/// `POST /fapi/v3/countdownCancelAll` — dead-man switch for one symbol.
#[derive(Debug, Clone)]
pub struct CountdownCancelAll {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Countdown in milliseconds; `0` stops the timer.
    /// Trading pair, e.g. `BTCUSDT`.
    /// Milliseconds; `0` stops the timer.
    pub countdown_time: i64,
}

impl Endpoint for CountdownCancelAll {
    type Response = CountdownCancelAllResult;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/fapi/v3/countdownCancelAll";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        p.set("countdownTime", &self.countdown_time);
        Ok(p)
    }

    fn weight(&self) -> u32 {
        10
    }
}

/// `POST /fapi/v3/noop` — cancel an in-flight on-chain order with the same nonce.
#[derive(Debug, Clone, Copy, Default)]
pub struct Noop;

impl Endpoint for Noop {
    type Response = Ack;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/fapi/v3/noop";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;
}

/// `POST /fapi/v3/chase` — BBO-pegged GTX order that re-pegs automatically.
#[derive(Debug, Clone)]
pub struct ChaseOrder {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Order side.
    pub side: Side,
    /// Quantity Unit.
    pub quantity_unit: QuantityUnit,
    /// Order quantity, in the base asset.
    pub quantity: Decimal,
    /// Position side (`BOTH` unless hedge mode).
    pub position_side: Option<PositionSide>,
    /// Whether the order may only reduce a position.
    pub reduce_only: Option<bool>,
    /// Chase Offset.
    pub chase_offset: Option<Decimal>,
    /// Chase Offset Type.
    pub chase_offset_type: Option<ChaseOffsetType>,
    /// Max Chase Offset.
    pub max_chase_offset: Option<Decimal>,
    /// Max Chase Offset Type.
    pub max_chase_offset_type: Option<ChaseOffsetType>,
    /// How long the order stays active.
    pub time_in_force: Option<TimeInForce>,
    /// Client-assigned strategy id.
    pub client_strategy_id: Option<String>,
}

/// Unit of a chase order's quantity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantityUnit {
    /// Quantity in the base asset.
    Base,
    /// Quantity in the quote asset.
    Quote,
}

impl QuantityUnit {
    /// Wire representation of this value.
    pub fn as_str(self) -> &'static str {
        match self {
            QuantityUnit::Base => "BASE",
            QuantityUnit::Quote => "QUOTE",
        }
    }
}

/// Unit of a chase offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChaseOffsetType {
    /// Offset in price units.
    Absolute,
    /// Offset in percent.
    Percentage,
}

impl ChaseOffsetType {
    /// Wire representation of this value.
    pub fn as_str(self) -> &'static str {
        match self {
            ChaseOffsetType::Absolute => "ABSOLUTE",
            ChaseOffsetType::Percentage => "PERCENTAGE",
        }
    }
}

impl ChaseOrder {
    /// Create a new value.
    pub fn new(
        symbol: impl Into<String>,
        side: Side,
        quantity_unit: QuantityUnit,
        quantity: impl Into<Decimal>,
    ) -> Self {
        Self {
            symbol: symbol.into(),
            side,
            quantity_unit,
            quantity: quantity.into(),
            position_side: None,
            reduce_only: None,
            chase_offset: None,
            chase_offset_type: None,
            max_chase_offset: None,
            max_chase_offset_type: None,
            time_in_force: None,
            client_strategy_id: None,
        }
    }
}

impl Endpoint for ChaseOrder {
    type Response = ChaseOrderResult;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/fapi/v3/chase";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        if let Some(id) = &self.client_strategy_id {
            if id.len() > 28 || !valid_client_order_id(id) {
                return Err(Error::InvalidParam("clientStrategyId"));
            }
        }
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        p.set("side", &self.side);
        p.set("quantityUnit", &self.quantity_unit.as_str());
        p.set("quantity", &self.quantity);
        p.set("positionSide", &self.position_side);
        p.set("reduceOnly", &self.reduce_only);
        p.set("chaseOffset", &self.chase_offset);
        p.set(
            "chaseOffsetType",
            &self.chase_offset_type.map(ChaseOffsetType::as_str),
        );
        p.set("maxChaseOffset", &self.max_chase_offset);
        p.set(
            "maxChaseOffsetType",
            &self.max_chase_offset_type.map(ChaseOffsetType::as_str),
        );
        p.set("timeInForce", &self.time_in_force);
        p.set("clientStrategyId", &self.client_strategy_id);
        Ok(p)
    }
}

/// A strategy leg sent to `placeStrategyOrder` / `updateStrategyOrder`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategySubOrderRequest {
    /// Leg sequence number, starting at 1.
    pub strategy_sub_id: u32,
    /// Security type of the strategy leg.
    pub security_type: SecurityType,
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Order side.
    pub side: Side,
    /// Position side (`BOTH` unless hedge mode).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position_side: Option<PositionSide>,
    /// Order type.
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// Order quantity, in the base asset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<Decimal>,
    /// Price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<Decimal>,
    /// BBO peg mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peg_price_type: Option<PegPriceType>,
    /// Trigger price for conditional orders.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<Decimal>,
    /// How long the order stays active.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<TimeInForce>,
    /// Price used to trigger the order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_type: Option<WorkingType>,
    /// Whether the order may only reduce a position.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_only: Option<bool>,
    /// Whether this is a close-all order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_position: Option<bool>,
    /// Whether the conditional trigger is price-protected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_protect: Option<bool>,
    /// Client order id; the idempotency key for orders.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    /// Activation price of a trailing-stop order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activation_price: Option<Decimal>,
    /// Trailing-stop callback rate, 1 = 1%.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_rate: Option<Decimal>,
    /// Driving leg for the first trigger.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_driven_id: Option<String>,
    /// Event that activates the first trigger.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_driven_on: Option<String>,
    /// First trigger action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_trigger: Option<String>,
    /// Driving leg for the second trigger.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub second_driven_id: Option<String>,
    /// Event that activates the second trigger.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub second_driven_on: Option<String>,
    /// Second trigger action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub second_trigger: Option<String>,
}

/// `POST /fapi/v3/placeStrategyOrder` (OTO / OCO / OTOCO).
#[derive(Debug, Clone)]
pub struct PlaceStrategyOrder {
    /// Strategy family (`OTO`, `OCO`, `OTOCO`).
    pub strategy_type: StrategyType,
    /// Strategy legs.
    pub sub_orders: Vec<StrategySubOrderRequest>,
    /// Client-assigned strategy id.
    pub client_strategy_id: Option<String>,
    /// Builder.
    pub builder: Option<String>,
    /// Builder fee rate; mandatory together with `builder`.
    pub fee_rate: Option<Decimal>,
}

impl Endpoint for PlaceStrategyOrder {
    type Response = StrategyOrderResult;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/fapi/v3/placeStrategyOrder";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        let expected = match self.strategy_type {
            StrategyType::Otoco => 3,
            _ => 2,
        };
        if self.sub_orders.len() != expected {
            return Err(Error::InvalidParam("subOrderList"));
        }
        let mut p = Params::new();
        p.set("strategyType", &self.strategy_type);
        p.set_str("subOrderList", serde_json::to_string(&self.sub_orders)?);
        p.set("clientStrategyId", &self.client_strategy_id);
        p.set("builder", &self.builder);
        p.set("feeRate", &self.fee_rate);
        Ok(p)
    }

    fn weight(&self) -> u32 {
        50
    }
}

/// `POST /fapi/v3/updateStrategyOrder`.
#[derive(Debug, Clone)]
pub struct UpdateStrategyOrder {
    /// Exchange strategy order id.
    pub strategy_id: i64,
    /// Strategy family (`OTO`, `OCO`, `OTOCO`).
    pub strategy_type: StrategyType,
    /// Strategy legs.
    pub sub_orders: Vec<StrategySubOrderRequest>,
}

impl Endpoint for UpdateStrategyOrder {
    type Response = Vec<StrategyUpdateResult>;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/fapi/v3/updateStrategyOrder";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        if self.sub_orders.is_empty() || self.sub_orders.len() > 3 {
            return Err(Error::InvalidParam("subOrderList"));
        }
        let mut p = Params::new();
        p.set("strategyId", &self.strategy_id);
        p.set("strategyType", &self.strategy_type);
        p.set_str("subOrderList", serde_json::to_string(&self.sub_orders)?);
        Ok(p)
    }

    fn weight(&self) -> u32 {
        50
    }
}

/// Strategy lookup by id or client id (mutually exclusive).
#[derive(Debug, Clone)]
pub struct StrategyLookup {
    /// Exchange strategy order id.
    pub strategy_id: Option<i64>,
    /// Client-assigned strategy id.
    pub client_strategy_id: Option<String>,
    /// Strategy family (`OTO`, `OCO`, `OTOCO`).
    pub strategy_type: StrategyType,
}

impl StrategyLookup {
    fn validate(&self) -> Result<()> {
        if self.strategy_id.is_some() == self.client_strategy_id.is_some() {
            return Err(Error::InvalidParam("strategyId/clientStrategyId"));
        }
        Ok(())
    }

    fn write(&self, p: &mut Params) {
        p.set("strategyId", &self.strategy_id);
        p.set("clientStrategyId", &self.client_strategy_id);
        p.set("strategyType", &self.strategy_type);
    }
}

/// `GET /fapi/v3/strategyOpenOrder`.
#[derive(Debug, Clone)]
pub struct StrategyOpenOrder(pub StrategyLookup);

impl Endpoint for StrategyOpenOrder {
    type Response = StrategyOrder;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/strategyOpenOrder";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        self.0.validate()?;
        let mut p = Params::new();
        self.0.write(&mut p);
        Ok(p)
    }

    fn weight(&self) -> u32 {
        5
    }
}

/// `GET /fapi/v3/strategyHistoryOrder` (90-day lookback).
#[derive(Debug, Clone)]
pub struct StrategyHistoryOrder {
    /// Strategy lookup parameters.
    pub lookup: StrategyLookup,
    /// Start of the query window (inclusive, ms).
    pub start_time: Option<i64>,
    /// End of the query window (inclusive, ms).
    pub end_time: Option<i64>,
    /// Maximum number of results.
    pub limit: Option<u32>,
}

impl Endpoint for StrategyHistoryOrder {
    type Response = StrategyOrder;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/strategyHistoryOrder";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        self.lookup.validate()?;
        let mut p = Params::new();
        self.lookup.write(&mut p);
        p.set("startTime", &self.start_time);
        p.set("endTime", &self.end_time);
        p.set("limit", &self.limit);
        Ok(p)
    }

    fn weight(&self) -> u32 {
        5
    }
}

/// `POST /fapi/v3/listenKey`.
#[derive(Debug, Clone)]
pub struct StartListenKey;

/// Newly created `listenKey`.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListenKeyResponse {
    /// User-data stream key (valid for 60 minutes).
    pub listen_key: String,
}

impl Endpoint for StartListenKey {
    type Response = ListenKeyResponse;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/fapi/v3/listenKey";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;
}

/// `PUT /fapi/v3/listenKey` — extend by another 60 minutes.
#[derive(Debug, Clone)]
pub struct KeepaliveListenKey;

impl Endpoint for KeepaliveListenKey {
    type Response = Empty;
    const METHOD: Method = Method::Put;
    const PATH: &'static str = "/fapi/v3/listenKey";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;
}

/// `DELETE /fapi/v3/listenKey`.
#[derive(Debug, Clone)]
pub struct CloseListenKey;

impl Endpoint for CloseListenKey {
    type Response = Empty;
    const METHOD: Method = Method::Delete;
    const PATH: &'static str = "/fapi/v3/listenKey";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;
}

/// Serialize an endpoint's validated params as a JSON array element, as the
/// `batchOrders` parameter expects.
fn json_array<T: Endpoint>(orders: &[T]) -> Result<String> {
    let mut objects = Vec::with_capacity(orders.len());
    for order in orders {
        let params = order.params()?;
        let mut map = serde_json::Map::new();
        // `Params` has no public iterator; re-encode and split on the canonical
        // form is fragile, so ask the endpoint for a JSON-safe view instead.
        for (key, value) in params.into_pairs() {
            map.insert(key.to_owned(), serde_json::Value::String(value));
        }
        objects.push(serde_json::Value::Object(map));
    }
    Ok(serde_json::to_string(&objects)?)
}
