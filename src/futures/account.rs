//! Futures account, position, wallet and reporting endpoints.

use crate::client::{Endpoint, Method, Security, Venue};
use crate::error::{Error, Result};
use crate::model::*;
use crate::params::Params;

/// Order lookup used by `order` / `openOrder` (id or client id).
#[derive(Debug, Clone)]
pub struct OrderLookup {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Exchange order id.
    pub order_id: Option<i64>,
    /// Client id of the existing order.
    pub orig_client_order_id: Option<String>,
}

impl OrderLookup {
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

/// `GET /fapi/v3/order`.
#[derive(Debug, Clone)]
pub struct QueryOrder(pub OrderLookup);

impl Endpoint for QueryOrder {
    type Response = Order;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/order";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        self.0.write(&mut p)?;
        Ok(p)
    }
}

/// `GET /fapi/v3/openOrder`.
#[derive(Debug, Clone)]
pub struct QueryOpenOrder(pub OrderLookup);

impl Endpoint for QueryOpenOrder {
    type Response = Order;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/openOrder";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        self.0.write(&mut p)?;
        Ok(p)
    }
}

/// `GET /fapi/v3/openOrders` (weight 1 with a symbol, 40 without).
#[derive(Debug, Clone, Default)]
pub struct OpenOrders {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: Option<String>,
}

impl OpenOrders {
    /// trading pair, e.g. `BTCUSDT`.
    pub fn symbol(symbol: impl Into<String>) -> Self {
        Self {
            symbol: Some(symbol.into()),
        }
    }
}

impl Endpoint for OpenOrders {
    type Response = Vec<Order>;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/openOrders";
    const VENUE: Venue = Venue::Futures;
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

/// `GET /fapi/v3/allOrders` (7-day window).
#[derive(Debug, Clone, Default)]
pub struct AllOrders {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: Option<String>,
    /// Exchange order id.
    pub order_id: Option<i64>,
    /// Start of the query window (inclusive, ms).
    pub start_time: Option<i64>,
    /// End of the query window (inclusive, ms).
    pub end_time: Option<i64>,
    /// Maximum number of results.
    pub limit: Option<u32>,
}

impl Endpoint for AllOrders {
    type Response = Vec<Order>;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/allOrders";
    const VENUE: Venue = Venue::Futures;
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

/// `GET /fapi/v3/balance`.
#[derive(Debug, Clone, Copy, Default)]
pub struct Balance;

impl Endpoint for Balance {
    type Response = Vec<BalancePayload>;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/balance";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn weight(&self) -> u32 {
        5
    }
}

/// `GET /fapi/v3/account`.
#[derive(Debug, Clone, Copy, Default)]
pub struct Account;

impl Endpoint for Account {
    type Response = AccountInfo;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/account";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn weight(&self) -> u32 {
        5
    }
}

/// `GET /fapi/v3/accountWithJoinMargin` (joint-margin account view).
#[derive(Debug, Clone, Copy, Default)]
pub struct AccountWithJoinMargin;

impl Endpoint for AccountWithJoinMargin {
    type Response = AccountInfo;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/accountWithJoinMargin";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn weight(&self) -> u32 {
        5
    }
}

/// `GET /fapi/v3/positionRisk`.
#[derive(Debug, Clone, Default)]
pub struct Positions {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: Option<String>,
}

impl Positions {
    /// trading pair, e.g. `BTCUSDT`.
    pub fn symbol(symbol: impl Into<String>) -> Self {
        Self {
            symbol: Some(symbol.into()),
        }
    }
}

impl Endpoint for Positions {
    type Response = Vec<PositionRisk>;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/positionRisk";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        Ok(p)
    }

    fn weight(&self) -> u32 {
        5
    }
}

/// `GET /fapi/v3/userTrades`.
#[derive(Debug, Clone)]
pub struct UserTrades {
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
    /// Maximum number of results.
    pub limit: Option<u32>,
}

impl UserTrades {
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

impl Endpoint for UserTrades {
    type Response = Vec<FuturesTrade>;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/userTrades";
    const VENUE: Venue = Venue::Futures;
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

/// `GET /fapi/v3/income` (7-day default window).
#[derive(Debug, Clone, Default)]
pub struct IncomeHistory {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: Option<String>,
    /// Income type.
    pub income_type: Option<String>,
    /// Start of the query window (inclusive, ms).
    pub start_time: Option<i64>,
    /// End of the query window (inclusive, ms).
    pub end_time: Option<i64>,
    /// Maximum number of results.
    pub limit: Option<u32>,
}

impl Endpoint for IncomeHistory {
    type Response = Vec<Income>;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/income";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        p.set("incomeType", &self.income_type);
        p.set("startTime", &self.start_time);
        p.set("endTime", &self.end_time);
        p.set("limit", &self.limit);
        Ok(p)
    }

    fn weight(&self) -> u32 {
        30
    }
}

/// `GET /fapi/v3/incomePage` (recommended for large windows).
#[derive(Debug, Clone, Default)]
pub struct IncomeHistoryPage {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: Option<String>,
    /// Income type.
    pub income_type: Option<String>,
    /// Start of the query window (inclusive, ms).
    pub start_time: Option<i64>,
    /// End of the query window (inclusive, ms).
    pub end_time: Option<i64>,
    /// Page number, starting at 1.
    pub page: Option<u32>,
    /// Page size.
    pub limit: Option<u32>,
}

impl Endpoint for IncomeHistoryPage {
    type Response = IncomePage;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/incomePage";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        p.set("incomeType", &self.income_type);
        p.set("startTime", &self.start_time);
        p.set("endTime", &self.end_time);
        p.set("page", &self.page);
        p.set("limit", &self.limit);
        Ok(p)
    }

    fn weight(&self) -> u32 {
        30
    }
}

/// `GET /fapi/v3/leverageBrackets` (object or array depending on `symbol`).
#[derive(Debug, Clone, Default)]
pub struct LeverageBrackets {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: Option<String>,
}

impl Endpoint for LeverageBrackets {
    type Response = OneOrMany<LeverageBracket>;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/leverageBrackets";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        Ok(p)
    }
}

/// `POST /fapi/v3/leverage`.
#[derive(Debug, Clone)]
pub struct ChangeLeverage {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Leverage.
    pub leverage: u32,
}

impl Endpoint for ChangeLeverage {
    type Response = LeverageChange;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/fapi/v3/leverage";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        if self.leverage == 0 || self.leverage > 125 {
            return Err(Error::InvalidParam("leverage"));
        }
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        p.set("leverage", &self.leverage);
        Ok(p)
    }
}

/// `POST /fapi/v3/marginType`.
#[derive(Debug, Clone)]
pub struct ChangeMarginType {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Margin mode.
    pub margin_type: MarginType,
}

impl Endpoint for ChangeMarginType {
    type Response = Ack;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/fapi/v3/marginType";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        p.set("marginType", &self.margin_type);
        Ok(p)
    }
}

/// `POST /fapi/v3/positionMargin`.
#[derive(Debug, Clone)]
pub struct ModifyPositionMargin {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// Amount.
    pub amount: rust_decimal::Decimal,
    /// Position side (`BOTH` unless hedge mode).
    pub position_side: Option<PositionSide>,
    /// `1` add margin, `2` reduce margin.
    /// Position side (`BOTH` unless hedge mode).
    /// `1` add, `2` reduce.
    pub change_type: u8,
    /// Client transaction id; idempotency key for transfers.
    pub client_tran_id: Option<String>,
}

impl Endpoint for ModifyPositionMargin {
    type Response = ModifyPositionMarginResult;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/fapi/v3/positionMargin";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        if !(1..=2).contains(&self.change_type) {
            return Err(Error::InvalidParam("type"));
        }
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        p.set("amount", &self.amount);
        p.set("positionSide", &self.position_side);
        p.set("type", &self.change_type);
        p.set("clientTranId", &self.client_tran_id);
        Ok(p)
    }
}

/// `GET /fapi/v3/positionMargin/history`.
#[derive(Debug, Clone)]
pub struct PositionMarginHistory {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
    /// `1` add margin, `2` reduce margin.
    pub change_type: Option<u8>,
    /// Start of the query window (inclusive, ms).
    pub start_time: Option<i64>,
    /// End of the query window (inclusive, ms).
    pub end_time: Option<i64>,
    /// Maximum number of results.
    pub limit: Option<u32>,
}

impl Endpoint for PositionMarginHistory {
    type Response = Vec<PositionMarginChange>;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/positionMargin/history";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        p.set("type", &self.change_type);
        p.set("startTime", &self.start_time);
        p.set("endTime", &self.end_time);
        p.set("limit", &self.limit);
        Ok(p)
    }
}

/// `GET /fapi/v3/positionSide/dual`.
#[derive(Debug, Clone, Copy, Default)]
pub struct PositionMode;

impl Endpoint for PositionMode {
    type Response = DualSidePosition;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/positionSide/dual";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn weight(&self) -> u32 {
        30
    }
}

/// `POST /fapi/v3/positionSide/dual`. Must be called before any position opens.
#[derive(Debug, Clone)]
pub struct ChangePositionMode {
    /// `true` for hedge mode, `false` for one-way mode.
    pub dual_side_position: bool,
}

impl Endpoint for ChangePositionMode {
    type Response = Ack;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/fapi/v3/positionSide/dual";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("dualSidePosition", &self.dual_side_position);
        Ok(p)
    }
}

/// `GET /fapi/v3/stpMode`.
#[derive(Debug, Clone, Copy, Default)]
pub struct StpModeQuery;

impl Endpoint for StpModeQuery {
    type Response = StpModeResponse;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/stpMode";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn weight(&self) -> u32 {
        30
    }
}

/// `POST /fapi/v3/stpMode`.
#[derive(Debug, Clone)]
pub struct ChangeStpMode {
    /// Account-level self-trade prevention mode.
    pub stp_mode: StpMode,
}

impl Endpoint for ChangeStpMode {
    type Response = Ack;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/fapi/v3/stpMode";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("stpMode", &self.stp_mode);
        Ok(p)
    }
}

/// `GET /fapi/v3/multiAssetsMargin`.
#[derive(Debug, Clone, Copy, Default)]
pub struct MultiAssetsMode;

impl Endpoint for MultiAssetsMode {
    type Response = MultiAssetsMargin;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/multiAssetsMargin";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn weight(&self) -> u32 {
        30
    }
}

/// `POST /fapi/v3/multiAssetsMargin`.
#[derive(Debug, Clone)]
pub struct ChangeMultiAssetsMode {
    /// Multi-assets margin mode.
    pub multi_assets_margin: bool,
}

impl Endpoint for ChangeMultiAssetsMode {
    type Response = Ack;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/fapi/v3/multiAssetsMargin";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("multiAssetsMargin", &self.multi_assets_margin);
        Ok(p)
    }
}

/// `POST /fapi/v3/assetExchange` (multi-assets mode only; empty body = no swap).
#[derive(Debug, Clone, Copy, Default)]
pub struct AssetExchange;

impl Endpoint for AssetExchange {
    type Response = Option<AssetExchangeResult>;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/fapi/v3/assetExchange";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;
}

/// `GET /fapi/v3/adlQuantile`.
#[derive(Debug, Clone, Default)]
pub struct AdlQuantileQuery {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: Option<String>,
}

impl Endpoint for AdlQuantileQuery {
    type Response = Vec<AdlQuantile>;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/adlQuantile";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        Ok(p)
    }

    fn weight(&self) -> u32 {
        5
    }
}

/// `GET /fapi/v3/forceOrders`.
#[derive(Debug, Clone, Default)]
pub struct ForceOrders {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: Option<String>,
    /// `true` for ADL/liquidation orders only.
    /// Trading pair, e.g. `BTCUSDT`.
    /// `true` for auto-close (ADL/liquidation) orders only.
    pub auto_close_type: Option<String>,
    /// Start of the query window (inclusive, ms).
    pub start_time: Option<i64>,
    /// End of the query window (inclusive, ms).
    pub end_time: Option<i64>,
    /// Maximum number of results.
    pub limit: Option<u32>,
}

impl Endpoint for ForceOrders {
    type Response = Vec<ForceOrder>;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/forceOrders";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        p.set("autoCloseType", &self.auto_close_type);
        p.set("startTime", &self.start_time);
        p.set("endTime", &self.end_time);
        p.set("limit", &self.limit);
        Ok(p)
    }

    fn weight(&self) -> u32 {
        20
    }
}

/// `GET /fapi/v3/commissionRate`.
#[derive(Debug, Clone)]
pub struct CommissionRateQuery {
    /// Trading pair, e.g. `BTCUSDT`.
    pub symbol: String,
}

impl Endpoint for CommissionRateQuery {
    type Response = CommissionRatePayload;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/commissionRate";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("symbol", &self.symbol);
        Ok(p)
    }
}

/// `POST /fapi/v3/asset/wallet/transfer` — `clientTranId` is the idempotency key.
#[derive(Debug, Clone)]
pub struct WalletTransfer {
    /// Amount.
    pub amount: rust_decimal::Decimal,
    /// Asset.
    pub asset: String,
    /// Client transaction id; idempotency key for transfers.
    pub client_tran_id: String,
    /// Transfer direction.
    pub kind: TransferKind,
}

impl Endpoint for WalletTransfer {
    type Response = TransferResult;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/fapi/v3/asset/wallet/transfer";
    const VENUE: Venue = Venue::Futures;
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

    fn weight(&self) -> u32 {
        50
    }
}

/// `GET /fapi/v3/announcement/direct`.
#[derive(Debug, Clone, Default)]
pub struct DirectAnnouncements {
    /// Page number, starting at 1.
    pub page: Option<u32>,
    /// Page size.
    pub size: Option<u32>,
}

impl Endpoint for DirectAnnouncements {
    type Response = AnnouncementPage;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/announcement/direct";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("page", &self.page);
        p.set("size", &self.size);
        Ok(p)
    }
}

/// `GET /fapi/v3/announcement/directById`.
#[derive(Debug, Clone)]
pub struct DirectAnnouncementById {
    /// Identifier.
    pub id: i64,
}

impl Endpoint for DirectAnnouncementById {
    type Response = AnnouncementPayload;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/announcement/directById";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("id", &self.id);
        Ok(p)
    }
}

pub use crate::model::AccountInfo as AccountInfoPayload;
pub use crate::model::Announcement as AnnouncementPayload;
pub use crate::model::Balance as BalancePayload;
pub use crate::model::CommissionRate as CommissionRatePayload;
