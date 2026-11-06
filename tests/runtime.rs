//! Reconnect, listen-key-free supervision, throttling and nonce persistence.
//!
//! Everything here runs on loopback: a local WebSocket server, an in-process
//! transport and temp files. Nothing leaves the machine.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use aster_client::auth::{
    FileNonceStore, MicrosecondNonce, NonceSource, PersistentNonce, now_micros,
};
use aster_client::client::{Client, HttpRequest, HttpResponse, Transport};
use aster_client::limits::RateLimiter;
use aster_client::ws::{Policy, Supervisor};
use aster_client::{Environment, futures};
use futures_util::SinkExt;
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;

const EVENT: &str =
    r#"{"e":"aggTrade","E":1,"s":"BTCUSDT","a":2,"p":"1.5","q":"2","f":1,"l":1,"T":1,"m":true}"#;

/// Accept connections forever; each one sends one `aggTrade` and closes.
async fn one_shot_ws_server() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        loop {
            let Ok((tcp, _)) = listener.accept().await else {
                return;
            };
            tokio::spawn(async move {
                let Ok(mut ws) = tokio_tungstenite::accept_async(tcp).await else {
                    return;
                };
                let _ = ws.send(Message::Text(EVENT.into())).await;
                let _ = ws.close(None).await;
            });
        }
    });
    port
}

#[tokio::test]
async fn supervisor_reconnects_and_reports_the_drop_once() {
    let port = one_shot_ws_server().await;
    let policy = Policy {
        rotate_after: Duration::from_secs(3_600),
        initial_backoff: Duration::from_millis(5),
        max_backoff: Duration::from_millis(20),
        ..Policy::default()
    };
    let mut supervisor = Supervisor::new(
        format!("ws://127.0.0.1:{port}/stream?streams=btcusdt@aggTrade"),
        policy,
    );

    let mut events = 0;
    let mut drops = 0;
    let deadline = Instant::now() + Duration::from_secs(5);
    while events < 3 {
        assert!(Instant::now() < deadline, "no progress");
        match supervisor.next().await {
            Ok(_) => events += 1,
            Err(_) => drops += 1,
        }
    }
    assert_eq!(drops, 2, "one error per closed socket");
    assert_eq!(supervisor.connections(), 3);
}

#[tokio::test]
async fn a_failed_first_connect_is_paced_before_retrying() {
    let policy = Policy {
        initial_backoff: Duration::from_millis(150),
        ..Policy::default()
    };
    let mut supervisor =
        Supervisor::new("ws://127.0.0.1:1/stream?streams=btcusdt@aggTrade", policy);
    assert!(supervisor.next().await.is_err(), "port 1 refuses");

    let started = Instant::now();
    assert!(supervisor.next().await.is_err());
    let waited = started.elapsed();
    assert!(waited >= Duration::from_millis(140), "{waited:?}");
}

// ---- transport-style tests -------------------------------------------------

#[derive(Clone)]
struct Fixed {
    body: String,
    calls: Arc<AtomicUsize>,
}

impl Fixed {
    fn new(body: impl Into<String>) -> Self {
        Self {
            body: body.into(),
            calls: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl Transport for Fixed {
    async fn execute(&self, _request: HttpRequest) -> aster_client::Result<HttpResponse> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Ok(HttpResponse {
            status: 200,
            body: self.body.clone(),
            retry_after: None,
        })
    }
}

#[tokio::test]
async fn attached_limiter_paces_requests_by_weight() {
    let transport = Fixed::new(r#"{"lastUpdateId":1,"bids":[],"asks":[]}"#);
    let client = Client::with_transport(Environment::Mainnet, transport.clone())
        .rate_limiter(Arc::new(RateLimiter::new(20, Duration::from_millis(100))));

    // `depth?limit=1000` weighs 20, i.e. the whole budget.
    let depth = || futures::Depth {
        symbol: "BTCUSDT".into(),
        limit: Some(1000),
    };
    let started = Instant::now();
    client.send(depth()).await.unwrap();
    client.send(depth()).await.unwrap();
    let elapsed = started.elapsed();
    assert!(elapsed >= Duration::from_millis(90), "{elapsed:?}");
    assert_eq!(transport.calls.load(Ordering::Relaxed), 2);
}

#[tokio::test]
async fn calibrate_time_shifts_nonces() {
    let before = now_micros();
    let server_ms = before / 1_000 + 5_000;
    let transport = Fixed::new(format!(r#"{{"serverTime":{server_ms}}}"#));
    let nonce = Arc::new(MicrosecondNonce::new());
    let client =
        Client::with_transport(Environment::Mainnet, transport).nonce_source(nonce.clone());

    let offset = client.calibrate_time().await.unwrap();
    assert!((4_900_000..=5_100_000).contains(&offset), "offset {offset}");
    assert!(nonce.next() >= before + 4_900_000);
}

// ---- nonce persistence -----------------------------------------------------

fn temp_path(tag: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("aster-nonce-{tag}-{}-{nanos}", std::process::id()))
}

#[test]
fn persistent_nonce_survives_a_restart() {
    let path = temp_path("restart");
    let first = {
        let nonce = PersistentNonce::new(FileNonceStore::new(&path));
        let value = nonce.next();
        nonce.flush();
        value
    };
    let second = PersistentNonce::new(FileNonceStore::new(&path)).next();
    assert!(second > first, "{second} <= {first}");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn persistent_nonce_never_goes_below_the_stored_mark() {
    let path = temp_path("seed");
    let future = now_micros() + 3_600_000_000;
    std::fs::write(&path, future.to_string()).unwrap();

    let nonce = PersistentNonce::new(FileNonceStore::new(&path));
    // The stored mark is an hour ahead of the wall clock: nothing below it may
    // be handed out again.
    for _ in 0..100 {
        assert!(nonce.next() > future);
    }
    let _ = std::fs::remove_file(&path);
}
