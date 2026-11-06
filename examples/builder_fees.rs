//! Aster Code builder attribution: builder + feeRate on an order, plus the
//! builder-scoped queries (`/fapi/v3/builder/*`).
//!
//! ```sh
//! cargo run --example builder_fees
//! # real queries need an approved agent signer and a funded builder account:
//! ASTER_SIGNER_KEY=0x… ASTER_USER=0x… ASTER_QUERY=1 cargo run --example builder_fees
//! ```

use std::env;

use aster_client::model::{Side, TimeInForce};
use aster_client::{Client, Endpoint, Environment, Error, futures};

const TEST_KEY: &str = "0x0123456789012345678901234567890123456789012345678901234567890123";
const BUILDER: &str = "0x000000000000000000000000000000000000b0b0";
const MASTER: &str = "0x000000000000000000000000000000000000a11c";

#[tokio::main]
async fn main() -> aster_client::Result<()> {
    let key = env::var("ASTER_SIGNER_KEY").unwrap_or_else(|_| TEST_KEY.to_owned());
    let user = env::var("ASTER_USER")
        .ok()
        .map(|value| value.parse().map_err(|_| Error::InvalidParam("ASTER_USER")))
        .transpose()?;

    let client = Client::new(Environment::Testnet)
        .signer(aster_client::auth::EvmSigner::from_hex(&key)?)
        .with_optional_user(user);

    // Builder orders carry `builder`, `feeRate` and the master `user`; the
    // signature is produced by the agent key. Supplying only one of
    // `builder`/`feeRate` is rejected locally (`-1102` server-side).
    let order = futures::NewOrder::limit("BTCUSDT", Side::Buy, "0.001", "50000")?
        .time_in_force(TimeInForce::Gtc)
        .builder_fee(BUILDER, "0.0001")?
        .user(MASTER);
    println!("signed payload: {}", order.params()?.encode());

    if env::var("ASTER_QUERY").is_ok() {
        let mut commissions = futures::BuilderUserCommissions::default();
        commissions.query.limit = Some(10);
        let page = client.send(commissions).await?;
        println!(
            "builder commissions: total={} rows={} errors={}",
            page.total,
            page.rows.len(),
            page.errors.len()
        );

        let mut accounts = futures::BuilderUserAccounts::default();
        accounts.query.limit = Some(5);
        accounts.symbol = Some("BTCUSDT".into());
        let page = client.send(accounts).await?;
        println!("builder users: total={}", page.total);
    } else {
        println!("set ASTER_QUERY=1 to run the builder-scoped queries");
    }
    Ok(())
}
