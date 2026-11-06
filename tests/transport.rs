//! Transport, environment matrix and the remaining request-pipeline branches.
//!
//! The `ReqwestTransport` tests run against a one-shot raw HTTP server on
//! `127.0.0.1:0`; nothing leaves the loopback interface.

use std::sync::Mutex;
use std::time::Duration;

use aster_client::auth::{Domain, EvmSigner};
use aster_client::client::{
    Client, Endpoint, Environment, HttpRequest, HttpResponse, Method, ReqwestTransport, Security,
    Transport, Venue,
};
use aster_client::error::Error;
use aster_client::params::Params;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

const KEY: &str = "0x0123456789012345678901234567890123456789012345678901234567890123";

#[test]
fn environment_matrix() {
    assert_eq!(
        Environment::Mainnet.futures_rest(),
        "https://fapi.asterdex.com"
    );
    assert_eq!(
        Environment::Testnet.futures_rest(),
        "https://fapi.asterdex-testnet.com"
    );
    assert_eq!(
        Environment::Mainnet.spot_rest(),
        "https://sapi.asterdex.com"
    );
    assert_eq!(
        Environment::Testnet.spot_rest(),
        "https://sapi.asterdex-testnet.com"
    );
    assert_eq!(
        Environment::Mainnet.futures_ws(),
        "wss://fstream.asterdex.com"
    );
    assert_eq!(
        Environment::Testnet.futures_ws(),
        "wss://fstream.asterdex-testnet.com"
    );
    assert_eq!(Environment::Mainnet.spot_ws(), "wss://sstream.asterdex.com");
    assert_eq!(
        Environment::Testnet.spot_ws(),
        "wss://sstream.asterdex-testnet.com"
    );

    assert_eq!(Environment::Mainnet.sign_chain_id(), 1666);
    assert_eq!(Environment::Testnet.sign_chain_id(), 714);

    for env in [Environment::Mainnet, Environment::Testnet] {
        assert_eq!(Venue::Futures.base_url(env), env.futures_rest());
        assert_eq!(Venue::Spot.base_url(env), env.spot_rest());
    }

    assert_eq!(Method::Get.as_str(), "GET");
    assert_eq!(Method::Post.as_str(), "POST");
    assert_eq!(Method::Put.as_str(), "PUT");
    assert_eq!(Method::Delete.as_str(), "DELETE");
    assert_eq!(Environment::default(), Environment::Mainnet);
}

// ---- ReqwestTransport against a loopback HTTP server -----------------------

fn head_end(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|w| w == b"\r\n\r\n").map(|p| p + 4)
}

/// Accept one connection, return the raw request it sent and reply `200 OK`
/// with `body`. `extra` is inserted verbatim into the response head.
async fn serve_once(
    extra: &'static str,
    body: &'static str,
) -> (String, tokio::task::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buf = Vec::new();
        let mut chunk = [0u8; 512];
        let end = loop {
            if let Some(end) = head_end(&buf) {
                break end;
            }
            let n = socket.read(&mut chunk).await.unwrap();
            if n == 0 {
                break buf.len();
            }
            buf.extend_from_slice(&chunk[..n]);
        };
        let head = String::from_utf8_lossy(&buf[..end]).to_ascii_lowercase();
        let len = head
            .lines()
            .find_map(|l| l.strip_prefix("content-length:"))
            .and_then(|v| v.trim().parse::<usize>().ok())
            .unwrap_or(0);
        while buf.len() < end + len {
            let n = socket.read(&mut chunk).await.unwrap();
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&chunk[..n]);
        }
        let request = String::from_utf8_lossy(&buf).to_string();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n{extra}Connection: close\r\n\r\n{body}",
            body.len()
        );
        socket.write_all(response.as_bytes()).await.unwrap();
        socket.flush().await.unwrap();
        request
    });
    (format!("http://{addr}"), handle)
}

fn get(url: String) -> HttpRequest {
    HttpRequest {
        method: Method::Get,
        url,
        body: None,
        content_type: None,
    }
}

#[tokio::test]
async fn reqwest_get_reports_target_user_agent_and_body() {
    let (base, server) = serve_once("", "{\"ok\":true}").await;
    let response = ReqwestTransport::default()
        .execute(get(format!("{base}/fapi/v3/ping")))
        .await
        .unwrap();
    assert_eq!(response.status, 200);
    assert_eq!(response.body, "{\"ok\":true}");
    assert!(response.retry_after.is_none());

    let request = server.await.unwrap();
    assert!(
        request.starts_with("GET /fapi/v3/ping HTTP/1.1"),
        "{request}"
    );
    let lower = request.to_ascii_lowercase();
    assert!(lower.contains("user-agent: aster-rs/"), "{request}");
    assert!(!lower.contains("content-type"), "{request}");
}

#[tokio::test]
async fn reqwest_write_methods_echo_method_body_and_default_content_type() {
    for method in [Method::Post, Method::Put, Method::Delete] {
        let (base, server) = serve_once("", "{}").await;
        let request = HttpRequest {
            method,
            url: format!("{base}/fapi/v3/order"),
            body: Some("symbol=BTCUSDT".to_owned()),
            content_type: None,
        };
        ReqwestTransport::default().execute(request).await.unwrap();

        let raw = server.await.unwrap();
        assert!(
            raw.starts_with(&format!("{} /fapi/v3/order HTTP/1.1", method.as_str())),
            "{raw}"
        );
        let lower = raw.to_ascii_lowercase();
        assert!(
            lower.contains("content-type: application/x-www-form-urlencoded"),
            "{raw}"
        );
        assert!(raw.ends_with("symbol=BTCUSDT"), "{raw}");
    }
}

#[tokio::test]
async fn reqwest_write_honours_explicit_content_type() {
    let (base, server) = serve_once("", "{}").await;
    let request = HttpRequest {
        method: Method::Post,
        url: format!("{base}/fapi/v3/order"),
        body: Some("{\"a\":1}".to_owned()),
        content_type: Some("application/json"),
    };
    ReqwestTransport::default().execute(request).await.unwrap();

    let raw = server.await.unwrap();
    let lower = raw.to_ascii_lowercase();
    assert!(lower.contains("content-type: application/json"), "{raw}");
    assert!(raw.ends_with("{\"a\":1}"), "{raw}");
}

#[tokio::test]
async fn reqwest_parses_retry_after_header() {
    let (base, server) = serve_once("Retry-After: 3\r\n", "slow down").await;
    let response = ReqwestTransport::default()
        .execute(get(format!("{base}/fapi/v3/ping")))
        .await
        .unwrap();
    let _ = server.await.unwrap();
    assert_eq!(response.body, "slow down");
    assert_eq!(response.retry_after, Some(Duration::from_secs(3)));
}

#[tokio::test]
async fn reqwest_connection_refused_is_transport_error() {
    // Grab a port, release it, then connect: nothing is listening there.
    let probe = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = probe.local_addr().unwrap().port();
    drop(probe);
    // `no_proxy` keeps an ambient HTTP_PROXY from answering the loopback call.
    let client = reqwest::Client::builder().no_proxy().build().unwrap();
    let error = ReqwestTransport::new(client)
        .execute(get(format!("http://127.0.0.1:{port}/")))
        .await
        .unwrap_err();
    assert!(matches!(error, Error::Transport(_)), "{error:?}");
}

// ---- Client construction and master-wallet user ----------------------------

#[derive(Default)]
struct Canned {
    body: String,
    seen: Mutex<Vec<HttpRequest>>,
}

impl Canned {
    fn new(body: &str) -> Self {
        Self {
            body: body.to_owned(),
            ..Self::default()
        }
    }
}

impl Transport for Canned {
    async fn execute(&self, request: HttpRequest) -> aster_client::Result<HttpResponse> {
        self.seen.lock().unwrap().push(request);
        Ok(HttpResponse {
            status: 200,
            body: self.body.clone(),
            retry_after: None,
        })
    }
}

/// Master-wallet endpoint (`Security::User`, no concrete public route yet).
struct UserOnly;

impl Endpoint for UserOnly {
    type Response = serde_json::Value;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/account";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::User;
}

/// Public GET whose response must be an array, to force a decode failure.
struct NeedsArray;

impl Endpoint for NeedsArray {
    type Response = Vec<serde_json::Value>;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/ping";
    const VENUE: Venue = Venue::Futures;
}

#[test]
fn new_builds_a_read_only_client_without_requests() {
    let client = Client::new(Environment::Mainnet);
    assert_eq!(client.environment(), Environment::Mainnet);
    assert!(!client.has_credentials());
    assert!(client.signer_address().is_none());
}

#[tokio::test]
async fn optional_user_sets_and_keeps_the_master_wallet() {
    let address: alloy_primitives::Address = "0x63DD5aCC6b1aa0f563956C0e534DD30B6dcF7C4e"
        .parse()
        .unwrap();
    let client = Client::with_transport(Environment::Mainnet, Canned::new("{}"))
        .signer(EvmSigner::from_hex(KEY).unwrap())
        // `None` must keep the address set by the previous call.
        .with_optional_user(Some(address))
        .with_optional_user(None);

    client.send(UserOnly).await.unwrap();
    let seen = client.transport().seen.lock().unwrap();
    let url = &seen[0].url;
    assert!(
        url.contains("user=0x63dd5acc6b1aa0f563956c0e534dd30b6dcf7c4e"),
        "{url}"
    );
    assert!(url.contains("signature=0x"), "{url}");
}

// ---- send_typed_signed GET branch and api_error_in_body branches -----------

#[tokio::test]
async fn typed_signed_get_puts_params_in_the_query() {
    let client = Client::with_transport(Environment::Mainnet, Canned::new("{}"))
        .signer(EvmSigner::from_hex(KEY).unwrap());

    let mut params = Params::new();
    params.set_str("agent", "0xagent");
    params.set("nonce", &7_u64);
    let payload = aster_client::auth::approve_builder(
        "0xagent",
        "0.001",
        "my-builder",
        aster_client::auth::AsterChain::Mainnet,
        "0xuser",
        7,
    );

    let _: serde_json::Value = client
        .send_typed_signed(
            Method::Get,
            Venue::Futures,
            "/fapi/v3/approveAgent",
            params,
            &Domain::aster_sign(1666),
            &payload,
        )
        .await
        .unwrap();

    let request = client.transport().seen.lock().unwrap()[0].clone();
    assert_eq!(request.method, Method::Get);
    assert!(request.body.is_none());
    assert_eq!(
        request.url.split('?').next().unwrap(),
        "https://fapi.asterdex.com/fapi/v3/approveAgent"
    );
    assert!(request.url.contains('?'), "{}", request.url);
    assert!(request.url.contains("agent=0xagent"), "{}", request.url);
    assert!(request.url.contains("nonce=7"), "{}", request.url);
    assert!(request.url.contains("signature=0x"), "{}", request.url);
}

#[tokio::test]
async fn string_code_in_body_is_an_api_error() {
    let client = Client::with_transport(
        Environment::Mainnet,
        Canned::new(r#"{"code":"-1003","msg":"too many requests"}"#),
    );
    let error = client.send(NeedsArray).await.unwrap_err();
    assert_eq!(error.code(), Some(-1003));
    match error {
        Error::Api(e) => assert_eq!(e.msg, "too many requests"),
        other => panic!("expected Api error, got {other:?}"),
    }
}

#[tokio::test]
async fn non_numeric_code_is_a_decode_error() {
    let client = Client::with_transport(
        Environment::Mainnet,
        Canned::new(r#"{"code":true,"msg":"x"}"#),
    );
    let error = client.send(NeedsArray).await.unwrap_err();
    assert!(matches!(error, Error::Decode(_)), "{error:?}");
}

#[tokio::test]
async fn success_code_with_wrong_shape_falls_back_to_the_api_error() {
    // `code: 200` is not a business error, so the payload is decoded normally;
    // when the shape does not match, the `ApiError` fallback still reports it.
    let client = Client::with_transport(
        Environment::Mainnet,
        Canned::new(r#"{"code":200,"msg":"ok"}"#),
    );
    let error = client.send(NeedsArray).await.unwrap_err();
    match error {
        Error::Api(e) => assert_eq!(e.code, 200),
        other => panic!("expected Api error, got {other:?}"),
    }
}
