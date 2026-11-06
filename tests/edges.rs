//! Remaining encode/decode branches that the matrix tests do not reach.

use aster_client::client::Endpoint;
use aster_client::error::{ApiError, Error};
use aster_client::futures::{
    BuilderQuery, BuilderUserOpenOrders, ChaseOffsetType, ChaseOrder, GuardedBatchCancelOrders,
    ModifyOrder, QuantityUnit, StrategySubOrderRequest, UpdateStrategyOrder,
};
use aster_client::model::{
    BatchItem, Filter, FilterKind, OneOrMany, OrderType, PriceTicker, SecurityType, Side,
    StrategyType, SymbolInfo, TimeInForce,
};
use rust_decimal::Decimal;
use serde_json::json;

fn d(value: &str) -> Decimal {
    value.parse().unwrap()
}

#[test]
fn builder_query_from_addresses() {
    let endpoint = BuilderUserOpenOrders {
        query: BuilderQuery::addresses(["0xa", "0xb", "0xc"]),
    };
    let params = endpoint.params().unwrap();
    assert_eq!(params.get("userAddresses"), Some("0xa,0xb,0xc"));
    assert_eq!(params.len(), 1);
}

#[test]
fn market_constructors_default_to_no_limit() {
    let depth = aster_client::futures::Depth::new("BTCUSDT");
    assert_eq!(depth.symbol, "BTCUSDT");
    assert_eq!(depth.limit, None);
    assert_eq!(
        depth.params().unwrap().pairs(),
        vec![("symbol", "BTCUSDT".to_owned())]
    );

    let trades = aster_client::futures::Trades::new("BTCUSDT");
    assert_eq!(trades.limit, None);
    assert_eq!(
        trades.params().unwrap().pairs(),
        vec![("symbol", "BTCUSDT".to_owned())]
    );

    let depth = aster_client::spot::Depth::new("BTCUSDT");
    assert_eq!(depth.limit, None);
    assert_eq!(
        depth.params().unwrap().pairs(),
        vec![("symbol", "BTCUSDT".to_owned())]
    );
}

#[test]
fn modify_order_client_id_builder() {
    let order = ModifyOrder::new("BTCUSDT", d("1"), d("100")).client_order_id("c1");
    let params = order.params().unwrap();
    assert_eq!(params.get("origClientOrderId"), Some("c1"));
    assert_eq!(params.get("orderId"), None);
}

#[test]
fn guarded_batch_cancel_id_lists() {
    let by_client = GuardedBatchCancelOrders {
        symbol: "BTCUSDT".into(),
        order_ids: vec![],
        client_order_ids: vec!["a".into(), "b".into()],
        nonce: 42,
    };
    let params = by_client.params().unwrap();
    assert_eq!(params.get("origClientOrderIdList"), Some(r#"["a","b"]"#));
    assert_eq!(params.get("orderIdList"), None);
    assert_eq!(by_client.nonce_override(), Some(42));

    let empty = GuardedBatchCancelOrders {
        symbol: "BTCUSDT".into(),
        order_ids: vec![],
        client_order_ids: vec![],
        nonce: 1,
    };
    assert!(matches!(
        empty.params().unwrap_err(),
        Error::InvalidParam("orderIdList/origClientOrderIdList")
    ));
}

#[test]
fn chase_order_offset_params() {
    let mut chase = ChaseOrder::new("BTCUSDT", Side::Buy, QuantityUnit::Quote, d("12.5"));
    chase.position_side = Some(aster_client::model::PositionSide::Both);
    chase.reduce_only = Some(true);
    chase.chase_offset = Some(d("0.1"));
    chase.chase_offset_type = Some(ChaseOffsetType::Percentage);
    chase.max_chase_offset = Some(d("5"));
    chase.max_chase_offset_type = Some(ChaseOffsetType::Absolute);
    chase.time_in_force = Some(TimeInForce::Gtc);
    chase.client_strategy_id = Some("cs1".into());

    let params = chase.params().unwrap();
    assert_eq!(params.get("quantityUnit"), Some("QUOTE"));
    assert_eq!(params.get("quantity"), Some("12.5"));
    assert_eq!(params.get("positionSide"), Some("BOTH"));
    assert_eq!(params.get("reduceOnly"), Some("true"));
    assert_eq!(params.get("chaseOffset"), Some("0.1"));
    assert_eq!(params.get("chaseOffsetType"), Some("PERCENTAGE"));
    assert_eq!(params.get("maxChaseOffset"), Some("5"));
    assert_eq!(params.get("maxChaseOffsetType"), Some("ABSOLUTE"));
    assert_eq!(params.get("clientStrategyId"), Some("cs1"));
    assert_eq!(ChaseOffsetType::Absolute.as_str(), "ABSOLUTE");
}

#[test]
fn update_strategy_sub_order_params() {
    let leg = |id: u32| StrategySubOrderRequest {
        strategy_sub_id: id,
        security_type: SecurityType::UsdtFutures,
        symbol: "BTCUSDT".into(),
        side: Side::Sell,
        position_side: None,
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
        client_order_id: None,
        activation_price: None,
        callback_rate: None,
        first_driven_id: None,
        first_driven_on: None,
        first_trigger: None,
        second_driven_id: None,
        second_driven_on: None,
        second_trigger: None,
    };
    let update = |legs: Vec<StrategySubOrderRequest>| UpdateStrategyOrder {
        strategy_id: 99,
        strategy_type: StrategyType::Oco,
        sub_orders: legs,
    };

    let params = update(vec![leg(1), leg(2)]).params().unwrap();
    assert_eq!(params.get("strategyId"), Some("99"));
    let encoded = params.get("subOrderList").unwrap();
    assert!(encoded.starts_with('['), "{encoded}");
    assert_eq!(
        serde_json::from_str::<Vec<serde_json::Value>>(encoded)
            .unwrap()
            .len(),
        2
    );

    assert!(matches!(
        update(vec![]).params().unwrap_err(),
        Error::InvalidParam("subOrderList")
    ));
    assert!(matches!(
        update(vec![leg(1), leg(2), leg(3), leg(4)])
            .params()
            .unwrap_err(),
        Error::InvalidParam("subOrderList")
    ));
}

#[test]
fn spot_stop_market_needs_quantity() {
    let mut order =
        aster_client::spot::NewSpotOrder::stop_market("BTCUSDT", Side::Sell, "1", "18500").unwrap();
    order.quantity = None;
    assert_eq!(
        order.params().unwrap_err().to_string(),
        "invalid parameter: quantity/stopPrice"
    );

    assert!(matches!(
        aster_client::spot::CancelSpotOrder {
            symbol: "BTCUSDT".into(),
            order_id: None,
            orig_client_order_id: None,
        }
        .params()
        .unwrap_err(),
        Error::InvalidParam("orderId/origClientOrderId")
    ));

    assert!(matches!(
        aster_client::spot::QuerySpotOrder(aster_client::spot::SpotOrderLookup {
            symbol: "BTCUSDT".into(),
            order_id: None,
            orig_client_order_id: None,
        })
        .params()
        .unwrap_err(),
        Error::InvalidParam("orderId/origClientOrderId")
    ));
}

#[test]
fn spot_cancel_id_lists() {
    let all = aster_client::spot::CancelAllSpotOrders {
        symbol: "BTCUSDT".into(),
        order_ids: vec![1, 2],
        client_order_ids: vec![],
    };
    let params = all.params().unwrap();
    assert_eq!(params.get("orderIdList"), Some("[1,2]"));
    assert_eq!(params.get("origClientOrderIdList"), None);

    let batch = aster_client::spot::CancelSpotBatchOrders {
        symbol: "BTCUSDT".into(),
        order_ids: vec![3],
        client_order_ids: vec![],
    };
    assert_eq!(batch.params().unwrap().get("orderIdList"), Some("[3]"));
}

fn symbol(filters: serde_json::Value) -> SymbolInfo {
    serde_json::from_value(json!({
        "symbol": "BTCUSDT",
        "status": "TRADING",
        "baseAsset": "BTC",
        "quoteAsset": "USDT",
        "filters": filters,
        "orderTypes": [],
        "timeInForce": []
    }))
    .unwrap()
}

#[test]
fn filter_kinds_and_notional_accessors() {
    let filters: Vec<Filter> = serde_json::from_value(json!([
        {"filterType": "PRICE_FILTER", "tickSize": "0.1"},
        {"filterType": "LOT_SIZE", "stepSize": "0.001"},
        {"filterType": "MARKET_LOT_SIZE", "stepSize": "0.001"},
        {"filterType": "MAX_NUM_ORDERS", "limit": 200},
        {"filterType": "MAX_NUM_ALGO_ORDERS", "limit": 100},
        {"filterType": "MIN_NOTIONAL", "notional": "5"},
        {"filterType": "MAX_NOTIONAL", "maxNotional": "100"},
        {"filterType": "NOTIONAL", "minNotional": "7", "maxNotional": "10"},
        {"filterType": "PERCENT_PRICE", "multiplierUp": "5"},
        {"filterType": "PERCENT_PRICE_BY_SIDE", "bidMultiplierUp": "5"},
        {"filterType": "BRAND_NEW", "foo": 1}
    ]))
    .unwrap();
    let kinds: Vec<FilterKind> = filters.iter().map(Filter::kind).collect();
    assert_eq!(
        kinds,
        vec![
            FilterKind::PriceFilter,
            FilterKind::LotSize,
            FilterKind::MarketLotSize,
            FilterKind::MaxNumOrders,
            FilterKind::MaxNumAlgoOrders,
            FilterKind::MinNotional,
            FilterKind::MaxNotional,
            FilterKind::Notional,
            FilterKind::PercentPrice,
            FilterKind::PercentPriceBySide,
            FilterKind::Unknown,
        ]
    );

    // `NOTIONAL.minNotional` is the fallback when `MIN_NOTIONAL` is absent.
    let notional = symbol(json!([{"filterType": "NOTIONAL", "minNotional": "7"}]));
    assert_eq!(notional.min_notional(), Some(Decimal::from(7)));
    // No filters at all: every typed accessor falls back to `None`.
    let bare = symbol(json!([]));
    assert_eq!(bare.tick_size(), None);
    assert_eq!(bare.step_size(), None);
    assert_eq!(bare.market_step_size(), None);
    assert_eq!(bare.max_qty(), None);
    assert_eq!(bare.min_notional(), None);
}

#[test]
fn display_batch_and_one_or_many() {
    assert_eq!(Side::Buy.to_string(), "BUY");
    assert_eq!(OrderType::StopMarket.to_string(), "STOP_MARKET");

    let err = BatchItem::<i32>::Err(ApiError::new(-2011, "Unknown order sent."));
    assert_eq!(err.clone().ok(), None);
    assert_eq!(err.as_result().unwrap_err().code, -2011);
    let ok = BatchItem::Ok(5_i32);
    assert_eq!(ok.clone().ok(), Some(5));
    assert_eq!(ok.as_result().unwrap(), 5);

    let one = OneOrMany::One(PriceTicker {
        symbol: "BTCUSDT".into(),
        price: Decimal::ONE,
        time: None,
    });
    assert_eq!(one.clone().into_vec().len(), 1);
    assert_eq!(one.as_slice().len(), 1);
    assert_eq!(
        serde_json::to_value(&one).unwrap(),
        json!({"symbol": "BTCUSDT", "price": "1", "time": null})
    );
}

#[test]
fn error_code_is_none_for_local_failures() {
    assert_eq!(Error::MissingCredentials.code(), None);
    assert_eq!(Error::InvalidParam("x").code(), None);
    assert_eq!(
        Error::Api(ApiError::new(-1121, "Invalid symbol.")).code(),
        Some(-1121)
    );
}
