//! Contract tests for spot trading, wallet and builder endpoints.

use aster_client::client::{Endpoint, Method, Security, Venue};
use aster_client::model::*;
use rust_decimal::Decimal;

fn d(value: &str) -> Decimal {
    value.parse().unwrap()
}

const GET: Method = Method::Get;
const PUT: Method = Method::Put;
const POST: Method = Method::Post;
const DELETE: Method = Method::Delete;
const SPOT: Venue = Venue::Spot;
const SIGNER: Security = Security::Signer;

const KEY: &str = "0x0123456789012345678901234567890123456789012345678901234567890123";

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
fn spot_order_endpoints() {
    use aster_client::spot::*;

    let limit = NewSpotOrder::limit("BTCUSDT", Side::Buy, "0.5", "20000")
        .unwrap()
        .time_in_force(TimeInForce::Gtx)
        .client_order_id("s1")
        .self_trade_protection(2)
        .resp_type(NewOrderRespType::Full)
        .builder_fee("0xbuilder", "0.001")
        .unwrap();
    check(
        limit,
        POST,
        "/api/v3/order",
        SPOT,
        SIGNER,
        &[
            ("symbol", "BTCUSDT"),
            ("side", "BUY"),
            ("type", "LIMIT"),
            ("timeInForce", "GTX"),
            ("quantity", "0.5"),
            ("price", "20000"),
            ("newClientOrderId", "s1"),
            ("selfTradingProtectionMode", "2"),
            ("newOrderRespType", "FULL"),
            ("builder", "0xbuilder"),
            ("feeRate", "0.001"),
        ],
    );

    check(
        NewSpotOrder::market_buy_quote("BTCUSDT", "100").unwrap(),
        POST,
        "/api/v3/order",
        SPOT,
        SIGNER,
        &[
            ("symbol", "BTCUSDT"),
            ("side", "BUY"),
            ("type", "MARKET"),
            ("quoteOrderQty", "100"),
        ],
    );
    check(
        NewSpotOrder::market_sell("BTCUSDT", "0.5").unwrap(),
        POST,
        "/api/v3/order",
        SPOT,
        SIGNER,
        &[
            ("symbol", "BTCUSDT"),
            ("side", "SELL"),
            ("type", "MARKET"),
            ("quantity", "0.5"),
        ],
    );
    check(
        NewSpotOrder::stop("BTCUSDT", Side::Sell, "1", "19000", "18500").unwrap(),
        POST,
        "/api/v3/order",
        SPOT,
        SIGNER,
        &[
            ("symbol", "BTCUSDT"),
            ("side", "SELL"),
            ("type", "STOP"),
            ("quantity", "1"),
            ("price", "19000"),
            ("stopPrice", "18500"),
        ],
    );
    check(
        NewSpotOrder::take_profit("BTCUSDT", Side::Sell, "1", "21000", "21500").unwrap(),
        POST,
        "/api/v3/order",
        SPOT,
        SIGNER,
        &[
            ("symbol", "BTCUSDT"),
            ("side", "SELL"),
            ("type", "TAKE_PROFIT"),
            ("quantity", "1"),
            ("price", "21000"),
            ("stopPrice", "21500"),
        ],
    );
    check(
        NewSpotOrder::stop_market("BTCUSDT", Side::Sell, "1", "18500").unwrap(),
        POST,
        "/api/v3/order",
        SPOT,
        SIGNER,
        &[
            ("symbol", "BTCUSDT"),
            ("side", "SELL"),
            ("type", "STOP_MARKET"),
            ("quantity", "1"),
            ("stopPrice", "18500"),
        ],
    );
    check(
        NewSpotOrder::take_profit_market("BTCUSDT", Side::Sell, "1", "21500").unwrap(),
        POST,
        "/api/v3/order",
        SPOT,
        SIGNER,
        &[
            ("symbol", "BTCUSDT"),
            ("side", "SELL"),
            ("type", "TAKE_PROFIT_MARKET"),
            ("quantity", "1"),
            ("stopPrice", "21500"),
        ],
    );

    // Local validation mirrors the documented matrix.
    let mut no_tif = NewSpotOrder::limit("BTCUSDT", Side::Buy, "1", "100").unwrap();
    no_tif.time_in_force = None;
    assert!(no_tif.params().is_err());

    let mut bad_mode = NewSpotOrder::limit("BTCUSDT", Side::Buy, "1", "100")
        .unwrap()
        .time_in_force(TimeInForce::Gtc);
    bad_mode.self_trade_protection_mode = Some(3);
    assert!(bad_mode.params().is_err());

    let mut bad_id = NewSpotOrder::limit("BTCUSDT", Side::Buy, "1", "100")
        .unwrap()
        .time_in_force(TimeInForce::Gtc);
    bad_id.client_order_id = Some("space id".into());
    assert!(bad_id.params().is_err());

    let mut trailing = NewSpotOrder::limit("BTCUSDT", Side::Buy, "1", "100").unwrap();
    trailing.order_type = OrderType::TrailingStopMarket;
    assert!(trailing.params().is_err());

    check(
        CancelSpotOrder::by_id("BTCUSDT", 7),
        DELETE,
        "/api/v3/order",
        SPOT,
        SIGNER,
        &[("symbol", "BTCUSDT"), ("orderId", "7")],
    );
    check(
        CancelSpotOrder::by_client_id("BTCUSDT", "s1"),
        DELETE,
        "/api/v3/order",
        SPOT,
        SIGNER,
        &[("symbol", "BTCUSDT"), ("origClientOrderId", "s1")],
    );
    check(
        QuerySpotOrder(SpotOrderLookup::by_id("BTCUSDT", 7)),
        GET,
        "/api/v3/order",
        SPOT,
        SIGNER,
        &[("symbol", "BTCUSDT"), ("orderId", "7")],
    );
    check(
        QueryOpenSpotOrder(SpotOrderLookup::by_client_id("BTCUSDT", "s1")),
        GET,
        "/api/v3/openOrder",
        SPOT,
        SIGNER,
        &[("symbol", "BTCUSDT"), ("origClientOrderId", "s1")],
    );
    check(
        OpenSpotOrders {
            symbol: Some("BTCUSDT".into()),
        },
        GET,
        "/api/v3/openOrders",
        SPOT,
        SIGNER,
        &[("symbol", "BTCUSDT")],
    );
    assert_eq!(OpenSpotOrders::default().weight(), 40);
    check(
        AllSpotOrders {
            symbol: Some("BTCUSDT".into()),
            order_id: Some(1),
            start_time: Some(2),
            end_time: Some(3),
            limit: Some(4),
        },
        GET,
        "/api/v3/allOrders",
        SPOT,
        SIGNER,
        &[
            ("symbol", "BTCUSDT"),
            ("orderId", "1"),
            ("startTime", "2"),
            ("endTime", "3"),
            ("limit", "4"),
        ],
    );
    check(
        CancelAllSpotOrders {
            symbol: "BTCUSDT".into(),
            order_ids: vec![1, 2],
            client_order_ids: vec!["a".into()],
        },
        DELETE,
        "/api/v3/allOpenOrders",
        SPOT,
        SIGNER,
        &[
            ("symbol", "BTCUSDT"),
            ("orderIdList", "[1,2]"),
            ("origClientOrderIdList", r#"["a"]"#),
        ],
    );
    check(
        CancelSpotBatchOrders {
            symbol: "BTCUSDT".into(),
            order_ids: vec![],
            client_order_ids: vec!["a".into()],
        },
        DELETE,
        "/api/v3/batchOrders",
        SPOT,
        SIGNER,
        &[("symbol", "BTCUSDT"), ("origClientOrderIdList", r#"["a"]"#)],
    );
    assert!(
        CancelSpotBatchOrders {
            symbol: "BTCUSDT".into(),
            order_ids: vec![],
            client_order_ids: vec![],
        }
        .params()
        .is_err()
    );
    check(
        SpotUserTrades {
            symbol: "BTCUSDT".into(),
            order_id: Some(1),
            start_time: Some(2),
            end_time: Some(3),
            from_id: Some(4),
            limit: Some(5),
        },
        GET,
        "/api/v3/userTrades",
        SPOT,
        SIGNER,
        &[
            ("symbol", "BTCUSDT"),
            ("orderId", "1"),
            ("startTime", "2"),
            ("endTime", "3"),
            ("fromId", "4"),
            ("limit", "5"),
        ],
    );
    check(SpotAccount, GET, "/api/v3/account", SPOT, SIGNER, &[]);
    check(
        SpotWalletTransfer {
            amount: d("10"),
            asset: "USDT".into(),
            client_tran_id: "t1".into(),
            kind: TransferKind::FutureSpot,
        },
        POST,
        "/api/v3/asset/wallet/transfer",
        SPOT,
        SIGNER,
        &[
            ("amount", "10"),
            ("asset", "USDT"),
            ("clientTranId", "t1"),
            ("kindType", "FUTURE_SPOT"),
        ],
    );
    assert!(
        SpotWalletTransfer {
            amount: d("1"),
            asset: "USDT".into(),
            client_tran_id: "x".repeat(65),
            kind: TransferKind::FutureSpot,
        }
        .params()
        .is_err()
    );
    check(Noop, POST, "/api/v3/noop", SPOT, SIGNER, &[]);
    check(StartListenKey, POST, "/api/v3/listenKey", SPOT, SIGNER, &[]);
    check(
        KeepaliveListenKey,
        PUT,
        "/api/v3/listenKey",
        SPOT,
        SIGNER,
        &[],
    );
    check(
        CloseListenKey,
        DELETE,
        "/api/v3/listenKey",
        SPOT,
        SIGNER,
        &[],
    );
}

#[test]
fn spot_builder_endpoints() {
    use aster_client::futures::{ApproveBuilder, DelBuilder, UpdateBuilder};
    use aster_client::spot::*;

    let signer = aster_client::auth::EvmSigner::from_hex(KEY).unwrap();
    let approve = SpotApproveBuilder(
        ApproveBuilder::sign(
            &signer,
            "0xbuilder",
            "0.01",
            "acme",
            aster_client::auth::AsterChain::Mainnet,
            1,
        )
        .unwrap(),
    );
    meta(
        &approve,
        POST,
        "/api/v3/approveBuilder",
        SPOT,
        Security::None,
    );
    assert_eq!(approve.params().unwrap().get("maxFeeRate"), Some("0.01"));
    assert_eq!(approve.params().unwrap().get("signatureChainId"), Some("56"));

    let update = SpotUpdateBuilder(
        UpdateBuilder::sign(
            &signer,
            "0xbuilder",
            "0.005",
            aster_client::auth::AsterChain::Mainnet,
            2,
        )
        .unwrap(),
    );
    meta(&update, POST, "/api/v3/updateBuilder", SPOT, Security::None);
    assert_eq!(update.params().unwrap().get("nonce"), Some("2"));

    let del = SpotDelBuilder(
        DelBuilder::sign(
            &signer,
            "0xbuilder",
            aster_client::auth::AsterChain::Mainnet,
            3,
        )
        .unwrap(),
    );
    meta(&del, DELETE, "/api/v3/delBuilder", SPOT, Security::None);
    assert_eq!(del.params().unwrap().get("builder"), Some("0xbuilder"));

    check(
        SpotBuilderAuthorizations,
        GET,
        "/api/v3/builder",
        SPOT,
        SIGNER,
        &[],
    );
}
