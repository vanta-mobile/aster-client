//! Live read-only smoke tests against mainnet. Run with:
//!
//! ```sh
//! cargo test --test live -- --ignored --nocapture
//! ```

use aster_client::{Client, Environment, futures, model::KlineInterval, spot};

#[tokio::test]
#[ignore = "network"]
async fn futures_market_data_round_trip() {
    let client = Client::new(Environment::Mainnet);

    client.send(futures::Ping).await.expect("ping");
    let time = client.send(futures::ServerTime).await.expect("time");
    assert!(time.server_time > 1_600_000_000_000);

    let info = client
        .send(futures::ExchangeInfo)
        .await
        .expect("exchangeInfo");
    assert!(info.symbols.len() > 100, "symbols: {}", info.symbols.len());
    let symbol = info.symbol("BTCUSDT").expect("BTCUSDT listed");
    let tick = symbol.tick_size().expect("PRICE_FILTER parsed");
    assert!(tick > rust_decimal::Decimal::ZERO, "tickSize {tick}");
    assert!(symbol.step_size().expect("LOT_SIZE parsed") > rust_decimal::Decimal::ZERO);
    assert!(symbol.is_trading());

    let depth = client
        .send(futures::Depth {
            symbol: "BTCUSDT".into(),
            limit: Some(5),
        })
        .await
        .expect("depth");
    assert!(!depth.bids.is_empty());
    assert_eq!(depth.bids.len().min(5), 5);

    let klines = client
        .send(futures::Klines::new("BTCUSDT", KlineInterval::Min1))
        .await
        .expect("klines");
    assert!(!klines.is_empty());
    assert!(klines[0].high >= klines[0].low);

    let ticker = client
        .send(futures::PriceTicker::symbol("BTCUSDT"))
        .await
        .expect("price");
    let one = ticker.as_slice();
    assert_eq!(one.len(), 1);
    assert!(one[0].price > rust_decimal::Decimal::ZERO);

    let premium = client
        .send(futures::PremiumIndex::default())
        .await
        .expect("premiumIndex");
    assert!(premium.as_slice().len() > 100);
}

#[tokio::test]
#[ignore = "network"]
async fn spot_market_data_round_trip() {
    let client = Client::new(Environment::Mainnet);
    let info = client
        .send(spot::ExchangeInfo)
        .await
        .expect("spot exchangeInfo");
    assert!(!info.symbols.is_empty());

    let book = client
        .send(spot::Depth {
            symbol: "BTCUSDT".into(),
            limit: Some(5),
        })
        .await
        .expect("spot depth");
    assert!(!book.asks.is_empty());
}

#[tokio::test]
#[ignore = "network"]
async fn api_errors_are_typed() {
    let client = Client::new(Environment::Mainnet);
    let error = client
        .send(futures::Depth {
            symbol: "NOPEUSDT".into(),
            limit: Some(5),
        })
        .await
        .expect_err("invalid symbol must fail");
    assert!(
        error.code().is_some(),
        "expected an exchange code, got {error:?}"
    );
}

#[tokio::test]
#[ignore = "network"]
async fn websocket_market_stream() {
    use futures_util::StreamExt;

    let streams = aster_client::ws::Streams::new()
        .agg_trade("BTCUSDT")
        .kline("BTCUSDT", KlineInterval::Min1);
    let mut events = aster_client::ws::subscribe(
        Environment::Mainnet,
        aster_client::client::Venue::Futures,
        &streams,
    )
    .await
    .expect("websocket connect");

    let first = tokio::time::timeout(std::time::Duration::from_secs(30), events.next())
        .await
        .expect("timeout waiting for a stream event")
        .expect("stream ended")
        .expect("decode event");
    assert!(first.event_time().is_some());
}

#[tokio::test]
#[ignore = "network"]
async fn aster_chain_public_round_trip() {
    use aster_client::aster_chain;

    // `/aster-chain/v3` is served by `chainapi.*`, not `fapi.*`.
    let client = Client::new(Environment::Mainnet);

    // Public (NONE): locked volume for the current lock period.
    let locked = client
        .send(aster_chain::LockedAster)
        .await
        .expect("getLockedAster");
    assert!(!locked.period_code.is_empty());
    assert!(locked.total_volume > rust_decimal::Decimal::ZERO);

    // Public (NONE): withdraw fee estimate for USDT on Ethereum.
    let fee = client
        .send(aster_chain::EstimateWithdrawFee {
            chain_id: 1,
            asset: "USDT".into(),
        })
        .await
        .expect("estimateFee");
    assert!(fee.gas_cost >= rust_decimal::Decimal::ZERO);
    assert!(fee.token_price > rust_decimal::Decimal::ZERO);
}
