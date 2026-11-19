//! Contract tests for futures + spot REST endpoints: method, path, venue,
//! security type and the exact canonical parameter set. These are the wire
//! names Aster validates, so a typo here would otherwise only surface live.

use aster_client::client::{Endpoint, Method, Security, Venue};
use aster_client::model::{
    KlineInterval, NewOrderRespType, OrderType, PegPriceType, PositionSide, SecurityType, Side,
    StpMode, StrategyType, TimeInForce, WorkingType,
};
use rust_decimal::Decimal;

fn d(value: &str) -> Decimal {
    value.parse().unwrap()
}

const GET: Method = Method::Get;
const POST: Method = Method::Post;
const PUT: Method = Method::Put;
const DELETE: Method = Method::Delete;
const FUT: Venue = Venue::Futures;
const SPOT: Venue = Venue::Spot;
const NONE: Security = Security::None;
const SIGNER: Security = Security::Signer;

/// Assert endpoint metadata.
fn meta<E: Endpoint>(
    endpoint: &E,
    method: Method,
    path: &'static str,
    venue: Venue,
    security: Security,
) {
    assert_eq!(E::METHOD, method, "method for {path}");
    assert_eq!(E::PATH, path, "path");
    assert_eq!(E::VENUE, venue, "venue for {path}");
    assert_eq!(E::SECURITY, security, "security for {path}");
    let _ = endpoint.weight();
}

/// Assert metadata + canonical wire parameters.
fn check<E: Endpoint>(
    endpoint: E,
    method: Method,
    path: &'static str,
    venue: Venue,
    security: Security,
    expected: &[(&'static str, &str)],
) {
    meta(&endpoint, method, path, venue, security);
    let mut want: Vec<(&'static str, String)> = expected
        .iter()
        .map(|(k, v)| (*k, (*v).to_owned()))
        .collect();
    want.sort_by(|a, b| a.0.cmp(b.0));
    assert_eq!(
        endpoint.params().unwrap().pairs(),
        want,
        "params for {path}"
    );
}

#[test]
fn futures_market_endpoints() {
    use aster_client::futures::*;

    check(Ping, GET, "/fapi/v3/ping", FUT, NONE, &[]);
    check(ServerTime, GET, "/fapi/v3/time", FUT, NONE, &[]);
    check(ExchangeInfo, GET, "/fapi/v3/exchangeInfo", FUT, NONE, &[]);

    check(
        Depth {
            symbol: "BTCUSDT".into(),
            limit: Some(5),
        },
        GET,
        "/fapi/v3/depth",
        FUT,
        NONE,
        &[("symbol", "BTCUSDT"), ("limit", "5")],
    );
    check(
        Trades {
            symbol: "BTCUSDT".into(),
            limit: Some(10),
        },
        GET,
        "/fapi/v3/trades",
        FUT,
        NONE,
        &[("symbol", "BTCUSDT"), ("limit", "10")],
    );
    check(
        HistoricalTrades {
            symbol: "BTCUSDT".into(),
            limit: Some(10),
            from_id: Some(7),
        },
        GET,
        "/fapi/v3/historicalTrades",
        FUT,
        NONE,
        &[("symbol", "BTCUSDT"), ("limit", "10"), ("fromId", "7")],
    );
    check(
        AggTrades {
            symbol: "BTCUSDT".into(),
            from_id: Some(1),
            start_time: Some(2),
            end_time: Some(3),
            limit: Some(4),
        },
        GET,
        "/fapi/v3/aggTrades",
        FUT,
        NONE,
        &[
            ("symbol", "BTCUSDT"),
            ("fromId", "1"),
            ("startTime", "2"),
            ("endTime", "3"),
            ("limit", "4"),
        ],
    );

    let mut klines = Klines::new("BTCUSDT", KlineInterval::Min1);
    klines.start_time = Some(1);
    klines.end_time = Some(2);
    klines.limit = Some(3);
    check(
        klines,
        GET,
        "/fapi/v3/klines",
        FUT,
        NONE,
        &[
            ("symbol", "BTCUSDT"),
            ("interval", "1m"),
            ("startTime", "1"),
            ("endTime", "2"),
            ("limit", "3"),
        ],
    );

    let mut index = IndexPriceKlines::new("BTCUSDT", KlineInterval::Hour4);
    index.start_time = Some(1);
    index.end_time = Some(2);
    index.limit = Some(9);
    check(
        index,
        GET,
        "/fapi/v3/indexPriceKlines",
        FUT,
        NONE,
        &[
            ("pair", "BTCUSDT"),
            ("interval", "4h"),
            ("startTime", "1"),
            ("endTime", "2"),
            ("limit", "9"),
        ],
    );

    let mut mark = MarkPriceKlines::new("BTCUSDT", KlineInterval::Day1);
    mark.start_time = Some(1);
    mark.end_time = Some(2);
    mark.limit = Some(3);
    check(
        mark,
        GET,
        "/fapi/v3/markPriceKlines",
        FUT,
        NONE,
        &[
            ("symbol", "BTCUSDT"),
            ("interval", "1d"),
            ("startTime", "1"),
            ("endTime", "2"),
            ("limit", "3"),
        ],
    );

    let mut market = MarketKlines::new("BTCUSDT", KlineInterval::Min5);
    market.start_time = Some(1);
    market.end_time = Some(2);
    market.limit = Some(3);
    check(
        market,
        GET,
        "/fapi/v3/marketKlines",
        FUT,
        NONE,
        &[
            ("symbol", "BTCUSDT"),
            ("interval", "5m"),
            ("startTime", "1"),
            ("endTime", "2"),
            ("limit", "3"),
        ],
    );

    check(
        PremiumIndex::default(),
        GET,
        "/fapi/v3/premiumIndex",
        FUT,
        NONE,
        &[],
    );
    check(
        PremiumIndex {
            symbol: Some("BTCUSDT".into()),
        },
        GET,
        "/fapi/v3/premiumIndex",
        FUT,
        NONE,
        &[("symbol", "BTCUSDT")],
    );
    check(
        FundingRate {
            symbol: Some("BTCUSDT".into()),
            start_time: Some(1),
            end_time: Some(2),
            limit: Some(3),
        },
        GET,
        "/fapi/v3/fundingRate",
        FUT,
        NONE,
        &[
            ("symbol", "BTCUSDT"),
            ("startTime", "1"),
            ("endTime", "2"),
            ("limit", "3"),
        ],
    );
    check(
        FundingInfo {
            symbol: Some("BTCUSDT".into()),
        },
        GET,
        "/fapi/v3/fundingInfo",
        FUT,
        NONE,
        &[("symbol", "BTCUSDT")],
    );
    check(
        IndexReferences {
            symbol: "BTCUSDT".into(),
        },
        GET,
        "/fapi/v3/indexreferences",
        FUT,
        NONE,
        &[("symbol", "BTCUSDT")],
    );
    check(
        Ticker24hr::symbol("BTCUSDT"),
        GET,
        "/fapi/v3/ticker/24hr",
        FUT,
        NONE,
        &[("symbol", "BTCUSDT")],
    );
    check(
        PriceTicker::symbol("BTCUSDT"),
        GET,
        "/fapi/v3/ticker/price",
        FUT,
        NONE,
        &[("symbol", "BTCUSDT")],
    );
    check(
        BookTicker::symbol("BTCUSDT"),
        GET,
        "/fapi/v3/ticker/bookTicker",
        FUT,
        NONE,
        &[("symbol", "BTCUSDT")],
    );
    check(
        RemainingOpenableNotionalValue {
            symbol: "BTCUSDT".into(),
            leverage: 5,
        },
        GET,
        "/fapi/v3/remainingOpenableNotionalValue",
        FUT,
        NONE,
        &[("symbol", "BTCUSDT"), ("leverage", "5")],
    );

    // Documented, limit-dependent weights.
    assert_eq!(
        Depth {
            symbol: "BTCUSDT".into(),
            limit: Some(5)
        }
        .weight(),
        2
    );
    assert_eq!(
        Depth {
            symbol: "BTCUSDT".into(),
            limit: Some(100)
        }
        .weight(),
        5
    );
    assert_eq!(
        Depth {
            symbol: "BTCUSDT".into(),
            limit: None
        }
        .weight(),
        10
    );
    assert_eq!(
        Depth {
            symbol: "BTCUSDT".into(),
            limit: Some(1000)
        }
        .weight(),
        20
    );
    assert_eq!(Ticker24hr::default().weight(), 40);
    assert_eq!(Ticker24hr::symbol("BTCUSDT").weight(), 1);
    assert_eq!(
        RemainingOpenableNotionalValue {
            symbol: "BTCUSDT".into(),
            leverage: 1
        }
        .weight(),
        50
    );
    assert!(
        RemainingOpenableNotionalValue {
            symbol: "BTCUSDT".into(),
            leverage: 0
        }
        .params()
        .is_err()
    );
    assert_eq!(Klines::new("BTCUSDT", KlineInterval::Min1).weight(), 5);
}

#[test]
fn spot_market_endpoints() {
    use aster_client::spot::*;

    check(Ping, GET, "/api/v3/ping", SPOT, NONE, &[]);
    check(ServerTime, GET, "/api/v3/time", SPOT, NONE, &[]);
    check(ExchangeInfo, GET, "/api/v3/exchangeInfo", SPOT, NONE, &[]);
    check(
        Depth {
            symbol: "BTCUSDT".into(),
            limit: Some(5),
        },
        GET,
        "/api/v3/depth",
        SPOT,
        NONE,
        &[("symbol", "BTCUSDT"), ("limit", "5")],
    );
    check(
        Trades {
            symbol: "BTCUSDT".into(),
            limit: Some(10),
        },
        GET,
        "/api/v3/trades",
        SPOT,
        NONE,
        &[("symbol", "BTCUSDT"), ("limit", "10")],
    );
    check(
        HistoricalTrades {
            symbol: "BTCUSDT".into(),
            limit: None,
            from_id: Some(7),
        },
        GET,
        "/api/v3/historicalTrades",
        SPOT,
        NONE,
        &[("symbol", "BTCUSDT"), ("fromId", "7")],
    );
    check(
        AggTrades {
            symbol: "BTCUSDT".into(),
            from_id: Some(1),
            start_time: Some(2),
            end_time: Some(3),
            limit: Some(4),
        },
        GET,
        "/api/v3/aggTrades",
        SPOT,
        NONE,
        &[
            ("symbol", "BTCUSDT"),
            ("fromId", "1"),
            ("startTime", "2"),
            ("endTime", "3"),
            ("limit", "4"),
        ],
    );
    check(
        Klines::new("BTCUSDT", KlineInterval::Week1),
        GET,
        "/api/v3/klines",
        SPOT,
        NONE,
        &[("symbol", "BTCUSDT"), ("interval", "1w")],
    );
    check(
        Ticker24hr::symbol("BTCUSDT"),
        GET,
        "/api/v3/ticker/24hr",
        SPOT,
        NONE,
        &[("symbol", "BTCUSDT")],
    );
    check(
        PriceTicker::default(),
        GET,
        "/api/v3/ticker/price",
        SPOT,
        NONE,
        &[],
    );
    check(
        BookTicker::symbol("BTCUSDT"),
        GET,
        "/api/v3/ticker/bookTicker",
        SPOT,
        NONE,
        &[("symbol", "BTCUSDT")],
    );
    check(
        CommissionRateQuery {
            symbol: "BTCUSDT".into(),
        },
        GET,
        "/api/v3/commissionRate",
        SPOT,
        NONE,
        &[("symbol", "BTCUSDT")],
    );
}

#[test]
fn futures_order_endpoints() {
    use aster_client::futures::{
        BatchModifyOrders, BatchNewOrders, CancelAllOpenOrders, CancelOrder, GuardedCancelOrder,
        ModifyOrder, NewOrder,
    };

    let limit = NewOrder::limit("BTCUSDT", Side::Buy, "1.5", "200.25")
        .unwrap()
        .time_in_force(TimeInForce::Gtc)
        .client_order_id("a1b2")
        .reduce_only(true)
        .position_side(PositionSide::Both)
        .working_type(WorkingType::MarkPrice)
        .price_protect(true)
        .resp_type(NewOrderRespType::Result)
        .peg_price_type(PegPriceType::Queue1)
        .stp_mode(StpMode::ExpireTaker)
        .builder_fee("0xbuilder", "0.0001")
        .unwrap()
        .user("0xmaster");
    check(
        limit,
        POST,
        "/fapi/v3/order",
        FUT,
        SIGNER,
        &[
            ("symbol", "BTCUSDT"),
            ("side", "BUY"),
            ("type", "LIMIT"),
            ("timeInForce", "GTC"),
            ("quantity", "1.5"),
            ("price", "200.25"),
            ("newClientOrderId", "a1b2"),
            ("reduceOnly", "true"),
            ("positionSide", "BOTH"),
            ("workingType", "MARK_PRICE"),
            ("priceProtect", "TRUE"),
            ("newOrderRespType", "RESULT"),
            ("pegPriceType", "QUEUE_1"),
            ("stpMode", "EXPIRE_TAKER"),
            ("builder", "0xbuilder"),
            ("feeRate", "0.0001"),
            ("user", "0xmaster"),
        ],
    );

    check(
        NewOrder::market("BTCUSDT", Side::Sell, "2").unwrap(),
        POST,
        "/fapi/v3/order",
        FUT,
        SIGNER,
        &[
            ("symbol", "BTCUSDT"),
            ("side", "SELL"),
            ("type", "MARKET"),
            ("quantity", "2"),
        ],
    );
    check(
        NewOrder::stop("BTCUSDT", Side::Buy, "1", "100", "99").unwrap(),
        POST,
        "/fapi/v3/order",
        FUT,
        SIGNER,
        &[
            ("symbol", "BTCUSDT"),
            ("side", "BUY"),
            ("type", "STOP"),
            ("quantity", "1"),
            ("price", "100"),
            ("stopPrice", "99"),
        ],
    );
    check(
        NewOrder::take_profit("BTCUSDT", Side::Sell, "1", "110", "111").unwrap(),
        POST,
        "/fapi/v3/order",
        FUT,
        SIGNER,
        &[
            ("symbol", "BTCUSDT"),
            ("side", "SELL"),
            ("type", "TAKE_PROFIT"),
            ("quantity", "1"),
            ("price", "110"),
            ("stopPrice", "111"),
        ],
    );
    check(
        NewOrder::stop_market("BTCUSDT", Side::Sell, "95")
            .unwrap()
            .close_position(true),
        POST,
        "/fapi/v3/order",
        FUT,
        SIGNER,
        &[
            ("symbol", "BTCUSDT"),
            ("side", "SELL"),
            ("type", "STOP_MARKET"),
            ("stopPrice", "95"),
            ("closePosition", "true"),
        ],
    );
    check(
        NewOrder::take_profit_market("BTCUSDT", Side::Sell, "120")
            .unwrap()
            .quantity(d("1")),
        POST,
        "/fapi/v3/order",
        FUT,
        SIGNER,
        &[
            ("symbol", "BTCUSDT"),
            ("side", "SELL"),
            ("type", "TAKE_PROFIT_MARKET"),
            ("stopPrice", "120"),
            ("quantity", "1"),
        ],
    );
    check(
        NewOrder::trailing_stop_market("BTCUSDT", Side::Buy, "0.5")
            .unwrap()
            .activation_price(d("100"))
            .quantity(d("3"))
            .time_in_force(TimeInForce::Gtc),
        POST,
        "/fapi/v3/order",
        FUT,
        SIGNER,
        &[
            ("symbol", "BTCUSDT"),
            ("side", "BUY"),
            ("type", "TRAILING_STOP_MARKET"),
            ("callbackRate", "0.5"),
            ("activationPrice", "100"),
            ("quantity", "3"),
            ("timeInForce", "GTC"),
        ],
    );

    check(
        ModifyOrder::new("BTCUSDT", d("1"), d("100"))
            .order_id(9)
            .side(Side::Buy),
        PUT,
        "/fapi/v3/order",
        FUT,
        SIGNER,
        &[
            ("symbol", "BTCUSDT"),
            ("quantity", "1"),
            ("price", "100"),
            ("orderId", "9"),
            ("side", "BUY"),
        ],
    );
    check(
        CancelOrder::by_id("BTCUSDT", 9),
        DELETE,
        "/fapi/v3/order",
        FUT,
        SIGNER,
        &[("symbol", "BTCUSDT"), ("orderId", "9")],
    );
    check(
        CancelOrder::by_client_id("BTCUSDT", "a1"),
        DELETE,
        "/fapi/v3/order",
        FUT,
        SIGNER,
        &[("symbol", "BTCUSDT"), ("origClientOrderId", "a1")],
    );
    let guarded = GuardedCancelOrder {
        symbol: "BTCUSDT".into(),
        order_id: Some(9),
        orig_client_order_id: None,
        nonce: 42,
    };
    check(
        guarded,
        DELETE,
        "/fapi/v3/guardedCancelOrder",
        FUT,
        SIGNER,
        &[("symbol", "BTCUSDT"), ("orderId", "9")],
    );
    check(
        CancelAllOpenOrders {
            symbol: "BTCUSDT".into(),
        },
        DELETE,
        "/fapi/v3/allOpenOrders",
        FUT,
        SIGNER,
        &[("symbol", "BTCUSDT")],
    );

    // `batchOrders` is a JSON string; check the route and the encoded list.
    let batch = BatchNewOrders {
        orders: vec![
            NewOrder::limit("BTCUSDT", Side::Buy, "1", "100")
                .unwrap()
                .time_in_force(TimeInForce::Gtc),
            NewOrder::market("BTCUSDT", Side::Sell, "1").unwrap(),
        ],
    };
    meta(&batch, POST, "/fapi/v3/batchOrders", FUT, SIGNER);
    let encoded = batch.params().unwrap();
    let rows: Vec<serde_json::Value> =
        serde_json::from_str(encoded.get("batchOrders").unwrap()).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[1]["type"], "MARKET");

    let modify = BatchModifyOrders {
        orders: vec![ModifyOrder::new("BTCUSDT", d("1"), d("100")).order_id(1)],
    };
    meta(&modify, PUT, "/fapi/v3/batchOrders", FUT, SIGNER);
    let encoded = modify.params().unwrap();
    let rows: Vec<serde_json::Value> =
        serde_json::from_str(encoded.get("batchOrders").unwrap()).unwrap();
    assert_eq!(rows.len(), 1);
}

#[test]
fn futures_cancel_batch_chase_and_strategy_endpoints() {
    use aster_client::futures::{
        BatchCancelOrders, CancelAllOpenOrders as CancelAll, ChaseOrder, CountdownCancelAll,
        GuardedBatchCancelOrders, NewOrder, Noop, PlaceStrategyOrder, StrategyHistoryOrder,
        StrategyLookup, StrategyOpenOrder, StrategySubOrderRequest, UpdateStrategyOrder,
    };

    let by_ids = BatchCancelOrders {
        symbol: "BTCUSDT".into(),
        order_ids: vec![1, 2],
        client_order_ids: vec![],
    };
    check(
        by_ids,
        DELETE,
        "/fapi/v3/batchOrders",
        FUT,
        SIGNER,
        &[("symbol", "BTCUSDT"), ("orderIdList", "[1,2]")],
    );
    let by_client = BatchCancelOrders {
        symbol: "BTCUSDT".into(),
        order_ids: vec![],
        client_order_ids: vec!["a".into()],
    };
    check(
        by_client,
        DELETE,
        "/fapi/v3/batchOrders",
        FUT,
        SIGNER,
        &[("symbol", "BTCUSDT"), ("origClientOrderIdList", r#"["a"]"#)],
    );
    assert!(
        BatchCancelOrders {
            symbol: "BTCUSDT".into(),
            order_ids: vec![],
            client_order_ids: vec![]
        }
        .params()
        .is_err()
    );

    let guarded = GuardedBatchCancelOrders {
        symbol: "BTCUSDT".into(),
        order_ids: vec![1],
        client_order_ids: vec![],
        nonce: 5,
    };
    check(
        guarded,
        DELETE,
        "/fapi/v3/guardedBatchOrders",
        FUT,
        SIGNER,
        &[("symbol", "BTCUSDT"), ("orderIdList", "[1]")],
    );

    check(
        CountdownCancelAll {
            symbol: "BTCUSDT".into(),
            countdown_time: 120_000,
        },
        POST,
        "/fapi/v3/countdownCancelAll",
        FUT,
        SIGNER,
        &[("symbol", "BTCUSDT"), ("countdownTime", "120000")],
    );
    check(Noop, POST, "/fapi/v3/noop", FUT, SIGNER, &[]);
    check(
        NewOrder::market("BTCUSDT", Side::Buy, "1").unwrap(),
        POST,
        "/fapi/v3/order",
        FUT,
        SIGNER,
        &[
            ("symbol", "BTCUSDT"),
            ("side", "BUY"),
            ("type", "MARKET"),
            ("quantity", "1"),
        ],
    );
    check(
        CancelAll {
            symbol: "BTCUSDT".into(),
        },
        DELETE,
        "/fapi/v3/allOpenOrders",
        FUT,
        SIGNER,
        &[("symbol", "BTCUSDT")],
    );

    let chase = ChaseOrder {
        symbol: "BTCUSDT".into(),
        side: Side::Buy,
        quantity_unit: aster_client::futures::QuantityUnit::Quote,
        quantity: d("100"),
        position_side: Some(PositionSide::Long),
        reduce_only: Some(false),
        chase_offset: Some(d("0.5")),
        chase_offset_type: Some(aster_client::futures::ChaseOffsetType::Absolute),
        max_chase_offset: Some(d("10")),
        max_chase_offset_type: Some(aster_client::futures::ChaseOffsetType::Percentage),
        time_in_force: Some(TimeInForce::Gtx),
        client_strategy_id: Some("s1".into()),
    };
    check(
        chase,
        POST,
        "/fapi/v3/chase",
        FUT,
        SIGNER,
        &[
            ("symbol", "BTCUSDT"),
            ("side", "BUY"),
            ("quantityUnit", "QUOTE"),
            ("quantity", "100"),
            ("positionSide", "LONG"),
            ("reduceOnly", "false"),
            ("chaseOffset", "0.5"),
            ("chaseOffsetType", "ABSOLUTE"),
            ("maxChaseOffset", "10"),
            ("maxChaseOffsetType", "PERCENTAGE"),
            ("timeInForce", "GTX"),
            ("clientStrategyId", "s1"),
        ],
    );

    let leg = StrategySubOrderRequest {
        strategy_sub_id: 1,
        security_type: SecurityType::UsdtFutures,
        symbol: "BTCUSDT".into(),
        side: Side::Buy,
        position_side: Some(PositionSide::Long),
        order_type: OrderType::Limit,
        quantity: Some(d("1")),
        price: Some(d("100")),
        peg_price_type: None,
        stop_price: None,
        time_in_force: Some(TimeInForce::Gtc),
        working_type: None,
        reduce_only: None,
        close_position: None,
        price_protect: None,
        client_order_id: Some("leg1".into()),
        activation_price: None,
        callback_rate: None,
        first_driven_id: Some("1".into()),
        first_driven_on: Some("FILLED".into()),
        first_trigger: Some("PLACE_ORDER".into()),
        second_driven_id: None,
        second_driven_on: None,
        second_trigger: None,
    };
    let mut exit = leg.clone();
    exit.strategy_sub_id = 2;
    exit.order_type = OrderType::TakeProfitMarket;
    exit.price = None;
    exit.stop_price = Some(d("120"));

    let place = PlaceStrategyOrder {
        strategy_type: StrategyType::Oto,
        sub_orders: vec![leg.clone(), exit],
        client_strategy_id: Some("strategy-1".into()),
        builder: Some("0xbuilder".into()),
        fee_rate: Some(d("0.0001")),
    };
    meta(&place, POST, "/fapi/v3/placeStrategyOrder", FUT, SIGNER);
    let encoded = place.params().unwrap();
    assert_eq!(encoded.get("strategyType"), Some("OTO"));
    let legs: Vec<serde_json::Value> =
        serde_json::from_str(encoded.get("subOrderList").unwrap()).unwrap();
    assert_eq!(legs.len(), 2);
    assert_eq!(legs[0]["securityType"], "USDT_FUTURES");
    // 文档把 strategySubId / reduceOnly 等标为 STRING；发数字主网回 -1102（2026-11-19 实测）。
    assert_eq!(legs[0]["strategySubId"], "1");
    assert!(legs.iter().all(|leg| leg["strategySubId"].is_string()));

    let update = UpdateStrategyOrder {
        strategy_id: 7,
        strategy_type: StrategyType::Otoco,
        sub_orders: vec![leg],
    };
    meta(&update, POST, "/fapi/v3/updateStrategyOrder", FUT, SIGNER);

    check(
        StrategyOpenOrder(StrategyLookup {
            strategy_id: Some(7),
            client_strategy_id: None,
            strategy_type: StrategyType::Oco,
        }),
        GET,
        "/fapi/v3/strategyOpenOrder",
        FUT,
        SIGNER,
        &[("strategyId", "7"), ("strategyType", "OCO")],
    );
    check(
        StrategyHistoryOrder {
            lookup: StrategyLookup {
                strategy_id: None,
                client_strategy_id: Some("s1".into()),
                strategy_type: StrategyType::Oto,
            },
            start_time: Some(1),
            end_time: Some(2),
            limit: Some(3),
        },
        GET,
        "/fapi/v3/strategyHistoryOrder",
        FUT,
        SIGNER,
        &[
            ("clientStrategyId", "s1"),
            ("strategyType", "OTO"),
            ("startTime", "1"),
            ("endTime", "2"),
            ("limit", "3"),
        ],
    );

    // Argument validation of the strategy families.
    assert!(
        UpdateStrategyOrder {
            strategy_id: 1,
            strategy_type: StrategyType::Oto,
            sub_orders: vec![],
        }
        .params()
        .is_err()
    );
    assert!(
        PlaceStrategyOrder {
            strategy_type: StrategyType::Oto,
            sub_orders: vec![],
            client_strategy_id: None,
            builder: None,
            fee_rate: None,
        }
        .params()
        .is_err()
    );
}
