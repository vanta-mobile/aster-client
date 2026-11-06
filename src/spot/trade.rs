//! Spot trading, account, wallet and listen-key endpoints.

use rust_decimal::Decimal;

use crate::client::{Endpoint, Method, Security, Venue};
use crate::error::{Error, Result};
use crate::model::*;
use crate::params::Params;

/// `POST /api/v3/order`.
///
/// Use the constructors: they fix `type` and enforce the mandatory matrix
/// (`FOK` is not supported by Aster spot and is not modelled).
#[derive(Debug, Clone)]
pub struct NewSpotOrder {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Order side.
    pub side: Side,
    /// Order type.
    pub order_type: OrderType,
    /// How long the order stays active.
    pub time_in_force: Option<TimeInForce>,
    /// Order quantity, in the base asset.
    pub quantity: Option<Decimal>,
    /// Quote-asset amount to spend (market buys).
    pub quote_order_qty: Option<Decimal>,
    /// Price.
    pub price: Option<Decimal>,
    /// Client order id; the idempotency key for orders.
    pub client_order_id: Option<String>,
    /// Trigger price for conditional orders.
    pub stop_price: Option<Decimal>,
    /// Self-trade prevention: 1 none, 2 cancel taker, 4 cancel maker, 8 both.
    pub self_trade_protection_mode: Option<u8>,
    /// Response type requested for the order.
    pub new_order_resp_type: Option<NewOrderRespType>,
    /// Builder.
    pub builder: Option<String>,
    /// Builder fee rate; mandatory together with `builder`.
    pub fee_rate: Option<Decimal>,
}

/// Parse a wire decimal (accepts `Decimal`, integers and strings).
fn decimal(value: impl TryInto<Decimal>) -> Result<Decimal> {
    value.try_into().map_err(|_| Error::InvalidParam("decimal"))
}

impl NewSpotOrder {
    fn base(symbol: impl Into<String>, side: Side, order_type: OrderType) -> Self {
        Self {
            symbol: symbol.into(),
            side,
            order_type,
            time_in_force: None,
            quantity: None,
            quote_order_qty: None,
            price: None,
            client_order_id: None,
            stop_price: None,
            self_trade_protection_mode: None,
            new_order_resp_type: None,
            builder: None,
            fee_rate: None,
        }
    }

    /// `LIMIT`: `timeInForce`, `quantity`, `price`.
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

    /// `MARKET` spending `quoteOrderQty` (the documented way to market-buy).
    pub fn market_buy_quote(
        symbol: impl Into<String>,
        quote_order_qty: impl TryInto<Decimal>,
    ) -> Result<Self> {
        let mut order = Self::base(symbol, Side::Buy, OrderType::Market);
        order.quote_order_qty = Some(decimal(quote_order_qty)?);
        Ok(order)
    }

    /// `MARKET` selling `quantity` of the base asset.
    pub fn market_sell(symbol: impl Into<String>, quantity: impl TryInto<Decimal>) -> Result<Self> {
        let mut order = Self::base(symbol, Side::Sell, OrderType::Market);
        order.quantity = Some(decimal(quantity)?);
        Ok(order)
    }

    /// `STOP`: `quantity`, `price`, `stopPrice`.
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

    /// `TAKE_PROFIT`: `quantity`, `price`, `stopPrice`.
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

    /// `STOP_MARKET`: `quantity`, `stopPrice`.
    pub fn stop_market(
        symbol: impl Into<String>,
        side: Side,
        quantity: impl TryInto<Decimal>,
        stop_price: impl TryInto<Decimal>,
    ) -> Result<Self> {
        let mut order = Self::base(symbol, side, OrderType::StopMarket);
        order.quantity = Some(decimal(quantity)?);
        order.stop_price = Some(decimal(stop_price)?);
        Ok(order)
    }

    /// `TAKE_PROFIT_MARKET`: `quantity`, `stopPrice`.
    pub fn take_profit_market(
        symbol: impl Into<String>,
        side: Side,
        quantity: impl TryInto<Decimal>,
        stop_price: impl TryInto<Decimal>,
    ) -> Result<Self> {
        let mut order = Self::base(symbol, side, OrderType::TakeProfitMarket);
        order.quantity = Some(decimal(quantity)?);
        order.stop_price = Some(decimal(stop_price)?);
        Ok(order)
    }

    /// how long the order stays active.
    pub fn time_in_force(mut self, time_in_force: TimeInForce) -> Self {
        self.time_in_force = Some(time_in_force);
        self
    }

    /// client order id; the idempotency key for orders.
    pub fn client_order_id(mut self, id: impl Into<String>) -> Self {
        self.client_order_id = Some(id.into());
        self
    }

    /// `1` none, `2` cancel taker, `4` cancel maker, `8` cancel both.
    pub fn self_trade_protection(mut self, mode: u8) -> Self {
        self.self_trade_protection_mode = Some(mode);
        self
    }

    /// Set the requested order response type.
    pub fn resp_type(mut self, resp_type: NewOrderRespType) -> Self {
        self.new_order_resp_type = Some(resp_type);
        self
    }

    /// builder fee charged on the trade.
    pub fn builder_fee(
        mut self,
        builder: impl Into<String>,
        fee_rate: impl TryInto<Decimal>,
    ) -> Result<Self> {
        self.builder = Some(builder.into());
        self.fee_rate = Some(decimal(fee_rate)?);
        Ok(self)
    }

    fn validate(&self) -> Result<()> {
        use OrderType::*;
        match self.order_type {
            Limit => {
                if self.time_in_force.is_none() || self.quantity.is_none() || self.price.is_none() {
                    return Err(Error::InvalidParam("timeInForce/quantity/price"));
                }
            }
            Market => {
                if self.quantity.is_none() && self.quote_order_qty.is_none() {
                    return Err(Error::InvalidParam("quantity/quoteOrderQty"));
                }
                if self.price.is_some() || self.stop_price.is_some() {
                    return Err(Error::InvalidParam("price/stopPrice"));
                }
            }
            Stop | TakeProfit => {
                if self.quantity.is_none() || self.price.is_none() || self.stop_price.is_none() {
                    return Err(Error::InvalidParam("quantity/price/stopPrice"));
                }
            }
            StopMarket | TakeProfitMarket => {
                if self.quantity.is_none() || self.stop_price.is_none() {
                    return Err(Error::InvalidParam("quantity/stopPrice"));
                }
                if self.price.is_some() {
                    return Err(Error::InvalidParam("price"));
                }
            }
            TrailingStopMarket => return Err(Error::InvalidParam("type")),
        }
        if let Some(mode) = self.self_trade_protection_mode {
            if !matches!(mode, 1 | 2 | 4 | 8) {
                return Err(Error::InvalidParam("selfTradingProtectionMode"));
            }
        }
        if let Some(id) = &self.client_order_id {
            if !valid_client_order_id(id) {
                return Err(Error::InvalidParam("newClientOrderId"));
            }
        }
        if self.builder.is_some() != self.fee_rate.is_some() {
            return Err(Error::InvalidParam("builder/feeRate"));
        }
        Ok(())
    }
}

impl Endpoint for NewSpotOrder {
    type Response = SpotOrder;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/api/v3/order";
    const VENUE: Venue = Venue::Spot;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        self.validate()?;
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        p.set("side", &self.side);
        p.set("type", &self.order_type);
        p.set("timeInForce", &self.time_in_force);
        p.set("quantity", &self.quantity);
        p.set("quoteOrderQty", &self.quote_order_qty);
        p.set("price", &self.price);
        p.set("newClientOrderId", &self.client_order_id);
        p.set("stopPrice", &self.stop_price);
        p.set(
            "selfTradingProtectionMode",
            &self.self_trade_protection_mode,
        );
        p.set("newOrderRespType", &self.new_order_resp_type);
        p.set("builder", &self.builder);
        p.set("feeRate", &self.fee_rate);
        Ok(p)
    }
}

/// `DELETE /api/v3/order`.
#[derive(Debug, Clone)]
pub struct CancelSpotOrder {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Exchange order id.
    pub order_id: Option<i64>,
    /// Client id of the existing order.
    pub orig_client_order_id: Option<String>,
}

impl CancelSpotOrder {
    /// Target the order by exchange order id.
    pub fn by_id(symbol: impl Into<String>, order_id: i64) -> Self {
        Self {
            symbol: symbol.into(),
            order_id: Some(order_id),
            orig_client_order_id: None,
        }
    }

    /// Target the order by client order id.
    pub fn by_client_id(symbol: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            order_id: None,
            orig_client_order_id: Some(id.into()),
        }
    }
}

impl Endpoint for CancelSpotOrder {
    type Response = SpotOrder;
    const METHOD: Method = Method::Delete;
    const PATH: &'static str = "/api/v3/order";
    const VENUE: Venue = Venue::Spot;
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

/// Spot order lookup.
#[derive(Debug, Clone)]
pub struct SpotOrderLookup {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Exchange order id.
    pub order_id: Option<i64>,
    /// Client id of the existing order.
    pub orig_client_order_id: Option<String>,
}

impl SpotOrderLookup {
    /// Target the order by exchange order id.
    pub fn by_id(symbol: impl Into<String>, order_id: i64) -> Self {
        Self {
            symbol: symbol.into(),
            order_id: Some(order_id),
            orig_client_order_id: None,
        }
    }

    /// Target the order by client order id.
    pub fn by_client_id(symbol: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            order_id: None,
            orig_client_order_id: Some(id.into()),
        }
    }

    fn write(&self, p: &mut Params) -> Result<()> {
        if self.order_id.is_none() && self.orig_client_order_id.is_none() {
            return Err(Error::InvalidParam("orderId/origClientOrderId"));
        }
        p.set("symbol", &self.symbol);
        p.set("orderId", &self.order_id);
        p.set("origClientOrderId", &self.orig_client_order_id);
        Ok(())
    }
}

/// `GET /api/v3/order`.
#[derive(Debug, Clone)]
pub struct QuerySpotOrder(pub SpotOrderLookup);

impl Endpoint for QuerySpotOrder {
    type Response = SpotOrder;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/api/v3/order";
    const VENUE: Venue = Venue::Spot;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        self.0.write(&mut p)?;
        Ok(p)
    }
}

/// `GET /api/v3/openOrder`.
#[derive(Debug, Clone)]
pub struct QueryOpenSpotOrder(pub SpotOrderLookup);

impl Endpoint for QueryOpenSpotOrder {
    type Response = SpotOrder;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/api/v3/openOrder";
    const VENUE: Venue = Venue::Spot;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        self.0.write(&mut p)?;
        Ok(p)
    }
}

/// `GET /api/v3/openOrders` (weight 1 with symbol, 40 without).
#[derive(Debug, Clone, Default)]
pub struct OpenSpotOrders {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: Option<String>,
}

impl Endpoint for OpenSpotOrders {
    type Response = Vec<SpotOrder>;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/api/v3/openOrders";
    const VENUE: Venue = Venue::Spot;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        Ok(p)
    }

    fn weight(&self) -> u32 {
        if self.symbol.is_some() { 1 } else { 40 }
    }
}

/// `GET /api/v3/allOrders`.
#[derive(Debug, Clone, Default)]
pub struct AllSpotOrders {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: Option<String>,
    /// Exchange order id.
    pub order_id: Option<i64>,
    /// Start of the query window (inclusive, ms).
    pub start_time: Option<i64>,
    /// End of the query window (inclusive, ms).
    pub end_time: Option<i64>,
    /// Page size.
    pub limit: Option<u32>,
}

impl Endpoint for AllSpotOrders {
    type Response = Vec<SpotOrder>;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/api/v3/allOrders";
    const VENUE: Venue = Venue::Spot;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        p.set("orderId", &self.order_id);
        p.set("startTime", &self.start_time);
        p.set("endTime", &self.end_time);
        p.set("limit", &self.limit);
        Ok(p)
    }

    fn weight(&self) -> u32 {
        5
    }
}

/// `DELETE /api/v3/allOpenOrders`.
#[derive(Debug, Clone)]
pub struct CancelAllSpotOrders {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Exchange order ids.
    pub order_ids: Vec<i64>,
    /// Client order ids.
    pub client_order_ids: Vec<String>,
}

impl Endpoint for CancelAllSpotOrders {
    type Response = Ack;
    const METHOD: Method = Method::Delete;
    const PATH: &'static str = "/api/v3/allOpenOrders";
    const VENUE: Venue = Venue::Spot;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        if !self.order_ids.is_empty() {
            p.set_str("orderIdList", serde_json::to_string(&self.order_ids)?);
        }
        if !self.client_order_ids.is_empty() {
            p.set_str(
                "origClientOrderIdList",
                serde_json::to_string(&self.client_order_ids)?,
            );
        }
        Ok(p)
    }
}

/// `DELETE /api/v3/batchOrders`.
#[derive(Debug, Clone)]
pub struct CancelSpotBatchOrders {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Exchange order ids.
    pub order_ids: Vec<i64>,
    /// Client order ids.
    pub client_order_ids: Vec<String>,
}

impl Endpoint for CancelSpotBatchOrders {
    type Response = Vec<BatchItem<SpotOrder>>;
    const METHOD: Method = Method::Delete;
    const PATH: &'static str = "/api/v3/batchOrders";
    const VENUE: Venue = Venue::Spot;
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

/// `GET /api/v3/userTrades`.
#[derive(Debug, Clone)]
pub struct SpotUserTrades {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Exchange order id.
    pub order_id: Option<i64>,
    /// Start of the query window (inclusive, ms).
    pub start_time: Option<i64>,
    /// End of the query window (inclusive, ms).
    pub end_time: Option<i64>,
    /// Trade id to fetch from (defaults to the most recent).
    pub from_id: Option<i64>,
    /// Page size.
    pub limit: Option<u32>,
}

impl SpotUserTrades {
    /// Create a new value.
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            order_id: None,
            start_time: None,
            end_time: None,
            from_id: None,
            limit: None,
        }
    }
}

impl Endpoint for SpotUserTrades {
    type Response = Vec<SpotTrade>;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/api/v3/userTrades";
    const VENUE: Venue = Venue::Spot;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        p.set("orderId", &self.order_id);
        p.set("startTime", &self.start_time);
        p.set("endTime", &self.end_time);
        p.set("fromId", &self.from_id);
        p.set("limit", &self.limit);
        Ok(p)
    }

    fn weight(&self) -> u32 {
        5
    }
}

/// `GET /api/v3/account`.
#[derive(Debug, Clone, Copy, Default)]
pub struct SpotAccount;

impl Endpoint for SpotAccount {
    type Response = SpotAccountInfo;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/api/v3/account";
    const VENUE: Venue = Venue::Spot;
    const SECURITY: Security = Security::Signer;

    fn weight(&self) -> u32 {
        5
    }
}

/// `POST /api/v3/asset/wallet/transfer`.
#[derive(Debug, Clone)]
pub struct SpotWalletTransfer {
    /// Amount.
    pub amount: Decimal,
    /// Asset.
    pub asset: String,
    /// Client transaction id; idempotency key for transfers.
    pub client_tran_id: String,
    /// Transfer direction.
    pub kind: TransferKind,
}

impl Endpoint for SpotWalletTransfer {
    type Response = TransferResult;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/api/v3/asset/wallet/transfer";
    const VENUE: Venue = Venue::Spot;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        if self.client_tran_id.is_empty() || self.client_tran_id.len() > 64 {
            return Err(Error::InvalidParam("clientTranId"));
        }
        let mut p = Params::new();
        p.set("amount", &self.amount);
        p.set("asset", &self.asset);
        p.set("clientTranId", &self.client_tran_id);
        p.set("kindType", &self.kind);
        Ok(p)
    }
}

/// `POST /api/v3/noop`.
#[derive(Debug, Clone, Copy, Default)]
pub struct Noop;

impl Endpoint for Noop {
    type Response = Ack;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/api/v3/noop";
    const VENUE: Venue = Venue::Spot;
    const SECURITY: Security = Security::Signer;
}

/// `POST /api/v3/listenKey`.
#[derive(Debug, Clone, Copy, Default)]
pub struct StartListenKey;

impl Endpoint for StartListenKey {
    type Response = ListenKeyPayload;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/api/v3/listenKey";
    const VENUE: Venue = Venue::Spot;
    const SECURITY: Security = Security::Signer;
}

/// `PUT /api/v3/listenKey`.
#[derive(Debug, Clone, Copy, Default)]
pub struct KeepaliveListenKey;

impl Endpoint for KeepaliveListenKey {
    type Response = Empty;
    const METHOD: Method = Method::Put;
    const PATH: &'static str = "/api/v3/listenKey";
    const VENUE: Venue = Venue::Spot;
    const SECURITY: Security = Security::Signer;
}

/// `DELETE /api/v3/listenKey`.
#[derive(Debug, Clone, Copy, Default)]
pub struct CloseListenKey;

impl Endpoint for CloseListenKey {
    type Response = Empty;
    const METHOD: Method = Method::Delete;
    const PATH: &'static str = "/api/v3/listenKey";
    const VENUE: Venue = Venue::Spot;
    const SECURITY: Security = Security::Signer;
}

pub use crate::futures::trade::ListenKeyResponse as ListenKeyPayload;
pub use crate::model::SpotAccount as SpotAccountInfo;
