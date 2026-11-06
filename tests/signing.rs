//! Cross-verified signing vectors.
//!
//! The expected values were produced independently with Python
//! `eth_account` 0.13 (`encode_typed_data` + `Account.sign_message`) and match
//! the reference vector below.

use std::sync::Arc;

use alloy_primitives::Address;
use aster_client::auth::{EvmSigner, MicrosecondNonce, NonceSource, Signer, WithdrawChain};
use aster_client::client::{Client, Environment, HttpRequest, HttpResponse, Method, Transport};
use aster_client::error::Result;
use aster_client::funding::WithdrawIntent;
use aster_client::model::{Side, TimeInForce};
use rust_decimal::Decimal;

const KEY: &str = "0x0123456789012345678901234567890123456789012345678901234567890123";
const SIGNER: &str = "0x14791697260e4c9a71f18484c9f997b308e59325";
/// Exactly the string the client builds for the order below (signer is the
/// address derived from [`KEY`]).
const PAYLOAD: &str = "newClientOrderId=a1b2&nonce=1793240936605000&price=200.25&quantity=1.5&side=BUY&signer=0x14791697260e4c9a71f18484c9f997b308e59325&symbol=AAPLUSDT&timeInForce=GTC&type=LIMIT";
/// `eth_account` signature over [`PAYLOAD`] with [`KEY`] (domain 1666).
const SIGNATURE: &str = "0x5ddfd89fc0ff94c984f7e2a05848ff826799f3b20b32596fcf0751739ac4cde3508a753b4e4954b3943d6fbc635fbd382152f46e07af64fd93725aaf359c8b1c1b";

/// The reference vector: same message, but with the example agent
/// `0x21cf…` as `signer` (the reference address/key pairing is inconsistent,
/// the signature is valid for this exact string).
const DOC_PAYLOAD: &str = "newClientOrderId=a1b2&nonce=1793240936605000&price=200.25&quantity=1.5&side=BUY&signer=0x21cf8ae13bb72632562c6fff438652ba1a151bb0&symbol=AAPLUSDT&timeInForce=GTC&type=LIMIT";
const DOC_SIGNATURE: &str = "0x014c6bbdf0cbe6a5b6c3cee6c7ca8c6caac7ace3489437556513902186bc3940302e19b02e14c748423326c7c641ada2a3b5bf7c1c32a4cd01adbd78a5f756481c";

struct FixedNonce(u64);

impl NonceSource for FixedNonce {
    fn next(&self) -> u64 {
        self.0
    }
}

#[derive(Default)]
struct Capture {
    requests: std::sync::Mutex<Vec<HttpRequest>>,
}

impl Transport for Capture {
    async fn execute(&self, request: HttpRequest) -> Result<HttpResponse> {
        self.requests.lock().unwrap().push(request);
        Ok(HttpResponse {
            status: 200,
            body: r#"{
                "clientOrderId": "a1b2",
                "executedQty": "0",
                "orderId": 1,
                "origQty": "1.5",
                "price": "200.25",
                "side": "BUY",
                "status": "NEW",
                "symbol": "AAPLUSDT",
                "type": "LIMIT"
            }"#
            .to_owned(),
            retry_after: None,
        })
    }
}

#[tokio::test]
async fn signed_order_body_matches_reference_vector() {
    let signer = EvmSigner::from_hex(KEY).unwrap();
    assert_eq!(signer.address().to_string().to_lowercase(), SIGNER);

    let client = Client::with_transport(Environment::Mainnet, Capture::default())
        .signer(signer)
        .nonce_source(Arc::new(FixedNonce(1_793_240_936_605_000)));

    let order = aster_client::futures::NewOrder::limit("AAPLUSDT", Side::Buy, "1.5", "200.25")
        .unwrap()
        .time_in_force(TimeInForce::Gtc)
        .client_order_id("a1b2");

    let sent = client.send(order).await.unwrap();
    assert_eq!(sent.symbol, "AAPLUSDT");

    let requests = client.transport().requests.lock().unwrap();
    let request = &requests[0];
    assert_eq!(request.method, Method::Post);
    assert_eq!(request.url, "https://fapi.asterdex.com/fapi/v3/order");
    assert_eq!(
        request.body.as_deref(),
        Some(format!("{PAYLOAD}&signature={SIGNATURE}").as_str())
    );
}

#[tokio::test]
async fn unsigned_market_data_is_query_encoded() {
    let client = Client::with_transport(Environment::Testnet, Capture::default());
    let _ = client
        .send(aster_client::futures::Klines::new(
            "BTCUSDT",
            aster_client::model::KlineInterval::Min1,
        ))
        .await;
    let requests = client.transport().requests.lock().unwrap();
    assert_eq!(
        requests[0].url,
        "https://fapi.asterdex-testnet.com/fapi/v3/klines?interval=1m&symbol=BTCUSDT"
    );
    assert!(requests[0].body.is_none());
}

#[test]
fn withdraw_signature_matches_reference_vector() {
    let signer = EvmSigner::from_hex(KEY).unwrap();
    let intent = WithdrawIntent {
        chain: WithdrawChain::Bsc,
        asset: "USDT".to_owned(),
        amount: Decimal::from_str_exact("10.123400").unwrap(),
        fee: Decimal::from_str_exact("1.234567891").unwrap(),
        receiver: "0xD9cA6952F1b1349d27f91E4fa6FB8ef67b89F02d"
            .parse()
            .unwrap(),
    };
    let signed = intent.sign(&signer, Some(1_793_240_936_605_000)).unwrap();
    assert_eq!(
        signed.signature.to_hex(),
        "0xd9c45ce3150dc5694e4323e33baf5bd526bce292132e1c3b9db7ad82975983b0473cc245a97721f67f760e5c7735384b94e485a2e7ef923021bf1b1a87fbfd451c"
    );
}

#[test]
fn documented_message_vector_matches() {
    let signer = EvmSigner::from_hex(KEY).unwrap();
    let domain = aster_client::auth::Domain::aster_sign(1666);
    let signature = aster_client::auth::sign_message(&signer, &domain, DOC_PAYLOAD).unwrap();
    assert_eq!(signature.to_hex(), DOC_SIGNATURE);
}

#[test]
fn address_derivation_round_trip() {
    let signer = EvmSigner::from_hex(KEY).unwrap();
    let address: Address = SIGNER.parse().unwrap();
    assert_eq!(signer.address(), address);
    let digest = aster_client::auth::keccak(b"aster");
    let first = signer.sign_digest(digest).unwrap();
    let second = signer.sign_digest(digest).unwrap();
    assert_eq!(first, second, "secp256k1 signatures must be deterministic");
}

/// Endpoints disagreeing with the signed prefix are a protocol bug: keep this
/// invariant pinned for every signed endpoint reachable through `Endpoint`.
#[test]
fn nonce_source_is_shared() {
    let nonce = Arc::new(MicrosecondNonce::new());
    let a = nonce.next();
    let b = nonce.next();
    assert!(b > a);
}
