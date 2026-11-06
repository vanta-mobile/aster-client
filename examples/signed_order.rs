//! Signed order construction: local validation, canonical signing payload and
//! an optional live submission.
//!
//! ```sh
//! # dry run: signs with the throwaway key from the docs and prints the payload
//! cargo run --example signed_order
//! # live (testnet): set the environment instead of using the dry run
//! ASTER_SIGNER_KEY=0x… ASTER_USER=0x… ASTER_SEND=1 cargo run --example signed_order
//! ```
//!
//! The throwaway key below is public (it is the signing-test vector key); it
//! holds no funds and must never be used for real orders.

use std::env;

use aster_client::model::{Side, TimeInForce};
use aster_client::{Client, Endpoint, Environment, Error, futures};

const TEST_KEY: &str = "0x0123456789012345678901234567890123456789012345678901234567890123";

#[tokio::main]
async fn main() -> aster_client::Result<()> {
    let key = env::var("ASTER_SIGNER_KEY").unwrap_or_else(|_| TEST_KEY.to_owned());
    let user = env::var("ASTER_USER")
        .ok()
        .map(|value| value.parse().map_err(|_| Error::InvalidParam("ASTER_USER")))
        .transpose()?;

    // `with_user` may be called before or after `signer`.
    let client = Client::new(Environment::Testnet)
        .signer(aster_client::auth::EvmSigner::from_hex(&key)?)
        .with_optional_user(user);

    let order = futures::NewOrder::limit("BTCUSDT", Side::Buy, "0.001", "50000")?
        .time_in_force(TimeInForce::Gtx)
        .client_order_id("example-1")
        .reduce_only(false)
        .resp_type(aster_client::model::NewOrderRespType::Result);

    // Local validation (mandatory-parameter matrix) happens here.
    let params = order.params()?;
    println!("signing payload: {}", params.encode());
    println!("signer: {:?}", client.signer_address());

    if env::var("ASTER_SEND").is_ok() {
        let placed = client.send(order).await?;
        println!(
            "placed order {} status {:?} executed {}",
            placed.order_id, placed.status, placed.executed_qty
        );
    } else {
        println!("dry run only; set ASTER_SEND=1 (plus keys) to submit");
    }
    Ok(())
}
