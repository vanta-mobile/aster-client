//! Decoding and error-mapping behaviour pinned to documented payloads.

use std::sync::Mutex;

use aster_client::client::{Client, Environment, HttpRequest, HttpResponse, Method, Transport};
use aster_client::error::{ApiError, Error};
use aster_client::model::{Ack, BatchItem, Depth, Filter, Kline, Order, OrderStatus};

#[derive(Default)]
struct Canned {
    status: u16,
    body: String,
    retry_after: Option<std::time::Duration>,
    seen: Mutex<Vec<HttpRequest>>,
}

impl Canned {
    fn new(status: u16, body: &str) -> Self {
        Self {
            status,
            body: body.to_owned(),
            retry_after: None,
            seen: Default::default(),
        }
    }
}

impl Transport for Canned {
    async fn execute(&self, request: HttpRequest) -> aster_client::Result<HttpResponse> {
        self.seen.lock().unwrap().push(request);
        Ok(HttpResponse {
            status: self.status,
            body: self.body.clone(),
            retry_after: self.retry_after,
        })
    }
}

fn client(status: u16, body: &str) -> Client<Canned> {
    Client::with_transport(Environment::Mainnet, Canned::new(status, body))
}

#[tokio::test]
async fn http_503_is_unknown_state() {
    let error = client(503, "timeout")
        .send(aster_client::futures::Ping)
        .await
        .unwrap_err();
    assert!(error.is_execution_unknown());
    assert!(matches!(error, Error::UnknownState { status: 503, .. }));
}

#[tokio::test]
async fn http_429_carries_retry_after() {
    let canned = Canned {
        retry_after: Some(std::time::Duration::from_secs(120)),
        ..Canned::new(429, "too many")
    };
    let client = Client::with_transport(Environment::Mainnet, canned);
    let error = client.send(aster_client::futures::Ping).await.unwrap_err();
    assert!(error.is_rate_limited());
    assert!(
        matches!(error, Error::RateLimited { status: 429, retry_after: Some(d), .. } if d.as_secs() == 120)
    );
}

#[tokio::test]
async fn business_error_on_http_200_is_api_error() {
    let error = client(200, r#"{"code":-5050,"msg":"deposit required"}"#)
        .send(aster_client::futures::Ping)
        .await
        .unwrap_err();
    assert_eq!(error.code(), Some(-5050));
}

#[tokio::test]
async fn business_error_on_http_400_is_api_error() {
    let error = client(400, r#"{"code":-1121,"msg":"Invalid symbol."}"#)
        .send(aster_client::futures::Depth {
            symbol: "X".into(),
            limit: None,
        })
        .await
        .unwrap_err();
    assert_eq!(error.code(), Some(-1121));
    assert_eq!(error.to_string(), "api error: Invalid symbol. (-1121)");
}

#[tokio::test]
async fn signed_get_signs_the_sent_query() {
    let client = client(200, "[]");
    // Public client: the call must fail before any request is made.
    let error = client
        .send(aster_client::futures::AllOrders::default())
        .await
        .unwrap_err();
    assert!(matches!(error, Error::MissingCredentials));
    assert!(client.transport().seen.lock().unwrap().is_empty());
}

#[test]
fn decodes_documents_shapes() {
    let order: Order = serde_json::from_str(
        r#"{
            "clientOrderId": "a1b2",
            "executedQty": "0",
            "orderId": 22542179,
            "origQty": "1.5",
            "origType": "LIMIT",
            "price": "200.25",
            "reduceOnly": false,
            "side": "BUY",
            "positionSide": "BOTH",
            "status": "NEW",
            "stopPrice": "0",
            "closePosition": false,
            "symbol": "AAPLUSDT",
            "timeInForce": "GTC",
            "type": "LIMIT",
            "updateTime": 1793240936605,
            "workingType": "CONTRACT_PRICE",
            "priceProtect": false
        }"#,
    )
    .unwrap();
    assert_eq!(order.status, OrderStatus::New);
    assert_eq!(order.orig_qty.to_string(), "1.5");
    assert_eq!(order.avg_price, None);

    let kline: Kline = serde_json::from_str(
        r#"[1499040000000,"0.01634790","0.80000000","0.01575800","0.01577100","148976.11427815",1499644799999,"2434.19055334",308,"1756.87402397","28.46694368","17928899.62484339"]"#,
    )
    .unwrap();
    assert_eq!(kline.trades, 308);
    assert_eq!(kline.close.to_string(), "0.01577100");

    let depth: Depth = serde_json::from_str(
        r#"{"lastUpdateId":1027024,"E":1589436922972,"T":1589436922959,"bids":[["4.00000000","431.00000000"]],"asks":[["4.00000200","12.00000000"]]}"#,
    )
    .unwrap();
    assert_eq!(depth.bids[0].0.to_string(), "4.00000000");

    let ack: Ack = serde_json::from_str(r#"{"code":"200","msg":"done"}"#).unwrap();
    assert!(ack.is_success());

    let batch: Vec<BatchItem<Order>> = serde_json::from_str(
        r#"[{"orderId":1,"clientOrderId":"x","executedQty":"0","origQty":"1","price":"1","side":"BUY","status":"NEW","symbol":"BTCUSDT","type":"LIMIT"},{"code":-2011,"msg":"Unknown order sent."}]"#,
    )
    .unwrap();
    assert_eq!(batch.len(), 2);
    assert!(batch[0].clone().ok().is_some());
    assert_eq!(batch[1].clone().as_result().unwrap_err().code, -2011);

    let filters: serde_json::Value = serde_json::from_str(
        r#"[{"filterType":"PRICE_FILTER","maxPrice":"300","minPrice":"0.0001","tickSize":"0.0001"},
            {"filterType":"LOT_SIZE","maxQty":"10000000","minQty":"1","stepSize":"1"},
            {"filterType":"MIN_NOTIONAL","notional":"5"},
            {"filterType":"SOMETHING_NEW","foo":1}]"#,
    )
    .unwrap();
    let filters: Vec<Filter> = serde_json::from_value(filters).unwrap();
    assert!(
        matches!(filters[0], Filter::PriceFilter { tick_size, .. } if tick_size.to_string() == "0.0001")
    );
    assert!(
        matches!(filters[2], Filter::MinNotional { notional: Some(n), .. } if n.to_string() == "5")
    );
    assert!(matches!(filters[3], Filter::Unknown));
}

#[test]
fn strategy_failure_is_in_the_body_not_the_status() {
    let result: aster_client::model::StrategyOrderResult = serde_json::from_str(
        r#"{"strategyId":1,"strategyType":"OTO","strategyStatus":"REJECTED","failureCode":-4142,"failureReason":"would trigger"}"#,
    )
    .unwrap();
    assert_eq!(
        result.rejection().unwrap(),
        ApiError::new(-4142, "would trigger")
    );
}

#[tokio::test]
async fn signed_request_reuses_guarded_nonce() {
    use aster_client::auth::{EvmSigner, NonceSource};
    use std::sync::Arc;

    struct Fixed(u64);
    impl NonceSource for Fixed {
        fn next(&self) -> u64 {
            self.0
        }
    }

    let client = Client::with_transport(Environment::Mainnet, Canned::new(200, ORDER))
        .signer(
            EvmSigner::from_hex(
                "0x0123456789012345678901234567890123456789012345678901234567890123",
            )
            .unwrap(),
        )
        .nonce_source(Arc::new(Fixed(999)));
    assert!(client.has_credentials());
    assert!(client.signer_address().is_some());
    assert_eq!(client.environment(), Environment::Mainnet);

    let _ = client
        .send(aster_client::futures::GuardedCancelOrder {
            symbol: "BTCUSDT".into(),
            order_id: Some(1),
            orig_client_order_id: None,
            nonce: 42,
        })
        .await
        .unwrap();

    let body = client.transport().seen.lock().unwrap()[0]
        .body
        .clone()
        .unwrap();
    assert!(body.contains("nonce=42"), "{body}");
    assert!(!body.contains("nonce=999"), "{body}");
    assert!(body.contains("signature=0x"), "{body}");
}

#[tokio::test]
async fn typed_signed_helper_appends_signature() {
    use aster_client::auth::{Domain, EvmSigner, FieldValue, Struct};

    let client = Client::with_transport(Environment::Mainnet, Canned::new(200, "{}")).signer(
        EvmSigner::from_hex("0x0123456789012345678901234567890123456789012345678901234567890123")
            .unwrap(),
    );

    let mut params = aster_client::params::Params::new();
    params.set_str("agent", "0xagent");
    params.set("nonce", &7_u64);
    let data = Struct::new("ApproveAgent")
        .field("Agent", FieldValue::String("0xagent"))
        .field("Enabled", FieldValue::Bool(true))
        .field("Nonce", FieldValue::Uint(7));

    let _: serde_json::Value = client
        .send_typed_signed(
            Method::Post,
            aster_client::client::Venue::Futures,
            "/fapi/v3/approveAgent",
            params,
            &Domain::aster_sign(56),
            &data,
        )
        .await
        .unwrap();

    let request = client.transport().seen.lock().unwrap()[0].clone();
    assert_eq!(
        request.url,
        "https://fapi.asterdex.com/fapi/v3/approveAgent"
    );
    let body = request.body.unwrap();
    assert!(body.contains("agent=0xagent"));
    assert!(body.contains("signature=0x"));

    // Pre-signed endpoints on a public client must not hit the network.
    let public = Client::with_transport(Environment::Mainnet, Canned::new(200, "{}"));
    let mut params = aster_client::params::Params::new();
    params.set("nonce", &1_u64);
    let error = public
        .send_typed_signed::<serde_json::Value>(
            Method::Post,
            aster_client::client::Venue::Futures,
            "/fapi/v3/approveAgent",
            params,
            &Domain::aster_sign(56),
            &Struct::new("ApproveAgent"),
        )
        .await
        .unwrap_err();
    assert!(matches!(error, Error::MissingCredentials));
    assert!(public.transport().seen.lock().unwrap().is_empty());
}

const ORDER: &str = r#"{
    "clientOrderId": "a1", "executedQty": "0", "orderId": 1, "origQty": "1",
    "price": "1", "side": "BUY", "status": "CANCELED", "symbol": "BTCUSDT", "type": "LIMIT"
}"#;

#[tokio::test]
async fn empty_body_decodes_to_none() {
    let client = Client::with_transport(Environment::Mainnet, Canned::new(200, "")).signer(
        aster_client::auth::EvmSigner::from_hex(
            "0x0123456789012345678901234567890123456789012345678901234567890123",
        )
        .unwrap(),
    );
    let value: Option<aster_client::model::AssetExchangeResult> = client
        .send(aster_client::futures::AssetExchange)
        .await
        .unwrap();
    assert!(value.is_none());
}

#[tokio::test]
async fn malformed_success_body_is_a_decode_error() {
    let error = client(200, "not json")
        .send(aster_client::futures::Ping)
        .await
        .unwrap_err();
    assert!(matches!(error, Error::Decode(_)));
}

#[tokio::test]
async fn non_json_client_error_keeps_status_and_body() {
    let error = client(403, "forbidden")
        .send(aster_client::futures::Ping)
        .await
        .unwrap_err();
    assert!(matches!(error, Error::Http { status: 403, .. }));
    assert!(error.to_string().contains("forbidden"));
}

#[test]
fn helper_types_and_accessors() {
    use aster_client::auth::{Domain, FieldValue, Signature, Signer};
    use aster_client::params::{Param, Params};

    let mut params = Params::new();
    assert!(params.is_empty());
    params.set_str("b", "2");
    params.set("a", &true);
    params.set("c", &1_u64);
    assert_eq!(params.len(), 3);
    assert_eq!(params.get("a"), Some("true"));
    assert_eq!(params.pairs()[0].0, "a");
    assert_eq!(params.clone().into_pairs().len(), 3);
    assert!(params.encode().contains("a=true"));

    let address: alloy_primitives::Address = "0x14791697260e4c9a71f18484c9f997b308e59325"
        .parse()
        .unwrap();
    let mut params = Params::new();
    params.set("signer", &address);
    assert_eq!(
        params.get("signer"),
        Some("0x14791697260e4c9a71f18484c9f997b308e59325")
    );
    let mut params = Params::new();
    "x".to_owned().write("k", &mut params);
    assert_eq!(params.get("k"), Some("x"));

    assert_eq!(Domain::aster_sign(1666).chain_id, 1666);
    assert_ne!(
        Domain::aster_sign(1666).separator(),
        Domain::aster_sign(714).separator()
    );
    assert_eq!(FieldValue::Bool(true).type_name(), "bool");
    assert_eq!(FieldValue::Uint(1).type_name(), "uint256");

    let signature = Signature::from_hex(
        "0x014c6bbdf0cbe6a5b6c3cee6c7ca8c6caac7ace3489437556513902186bc3940302e19b02e14c748423326c7c641ada2a3b5bf7c1c32a4cd01adbd78a5f756481c",
    )
    .unwrap();
    assert_eq!(signature.as_bytes()[64], 28);
    assert!(signature.to_string().starts_with("0x014c"));
    assert!(Signature::from_hex("0x00").is_err());
    assert!(Signature::from_hex("zz").is_err());

    let no_signer = aster_client::auth::NoSigner;
    assert_eq!(no_signer.address(), alloy_primitives::Address::ZERO);
    assert!(
        no_signer
            .sign_digest(aster_client::auth::keccak(b"x"))
            .is_err()
    );

    assert_eq!(aster_client::client::Method::Get.as_str(), "GET");
    assert_eq!(aster_client::client::Method::Post.as_str(), "POST");
    assert_eq!(aster_client::client::Method::Put.as_str(), "PUT");
    assert_eq!(aster_client::client::Method::Delete.as_str(), "DELETE");
    assert_eq!(Environment::Mainnet.sign_chain_id(), 1666);
    assert_eq!(Environment::Testnet.sign_chain_id(), 714);
    assert_eq!(
        Environment::Mainnet.futures_ws(),
        "wss://fstream.asterdex.com"
    );
    assert_eq!(
        Environment::Testnet.spot_ws(),
        "wss://sstream.asterdex-testnet.com"
    );
    assert_eq!(
        aster_client::client::Venue::Futures.base_url(Environment::Mainnet),
        "https://fapi.asterdex.com"
    );
    assert_eq!(
        aster_client::client::Venue::Spot.base_url(Environment::Testnet),
        "https://sapi.asterdex-testnet.com"
    );
}
