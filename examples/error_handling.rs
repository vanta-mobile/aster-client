//! Error handling: distinguish "unknown execution state" (HTTP 503) from rate
//! limiting, business errors and local validation. Runs without network by
//! using a scripted [`Transport`].
//!
//! ```sh
//! cargo run --example error_handling
//! ```

use std::sync::Mutex;

use aster_client::client::{HttpRequest, HttpResponse, Transport};
use aster_client::model::{Side, TimeInForce};
use aster_client::{Client, Endpoint, Environment, Error, futures};

/// Returns a scripted response for every request.
struct Scripted {
    response: HttpResponse,
    seen: Mutex<Vec<HttpRequest>>,
}

impl Transport for Scripted {
    async fn execute(&self, request: HttpRequest) -> aster_client::Result<HttpResponse> {
        self.seen.lock().unwrap().push(request);
        Ok(self.response.clone())
    }
}

fn client(status: u16, body: &str) -> Client<Scripted> {
    Client::with_transport(
        Environment::Mainnet,
        Scripted {
            response: HttpResponse {
                status,
                body: body.to_owned(),
                retry_after: None,
            },
            seen: Mutex::new(Vec::new()),
        },
    )
}

#[tokio::main]
async fn main() -> aster_client::Result<()> {
    // 1. HTTP 503: the request may have reached the server. For writes,
    //    reconcile by client order id before deciding to retry.
    let error = client(503, "gateway timeout")
        .send(futures::Depth {
            symbol: "BTCUSDT".into(),
            limit: Some(5),
        })
        .await
        .unwrap_err();
    assert!(error.is_execution_unknown());
    println!("503 -> unknown state: {error}");

    // 2. Business errors can arrive with HTTP 200 (e.g. -5050 deposit gate).
    let error = client(200, r#"{"code":-5050,"msg":"deposit required"}"#)
        .send(futures::Ping)
        .await
        .unwrap_err();
    println!("HTTP 200 + code -> api error {:?}", error.code());

    // 3. Local validation rejects a LIMIT order without time in force.
    let order = futures::NewOrder::limit("BTCUSDT", Side::Buy, "0.001", "50000")?;
    match order.params() {
        Err(Error::InvalidParam(param)) => println!("rejected locally: missing {param}"),
        other => panic!("expected local rejection, got {other:?}"),
    }
    let order = order.time_in_force(TimeInForce::Gtc);

    // 4. Validated parameters are encoded exactly as signed/sent.
    println!("payload: {}", order.params()?.encode());

    // 5. A scripted success response decodes into the typed model.
    let depth = client(
        200,
        r#"{
        "lastUpdateId": 1,
        "bids": [["50000.00","0.5"]],
        "asks": [["50001.00","0.5"]]
    }"#,
    )
    .send(futures::Depth {
        symbol: "BTCUSDT".into(),
        limit: Some(5),
    })
    .await?;
    println!("best bid {:?}", depth.bids.first());

    // 6. Rate limiting carries the server's Retry-After hint.
    let limited = Client::with_transport(
        Environment::Mainnet,
        Scripted {
            response: HttpResponse {
                status: 429,
                body: "too many requests".to_owned(),
                retry_after: Some(std::time::Duration::from_secs(30)),
            },
            seen: Mutex::new(Vec::new()),
        },
    );
    match limited.send(futures::Ping).await {
        Err(Error::RateLimited { retry_after, .. }) => {
            println!("back off for {retry_after:?}");
        }
        other => panic!("expected rate limiting, got {other:?}"),
    }
    Ok(())
}
