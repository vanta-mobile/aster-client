//! Public futures/spot market data: no credentials, live mainnet calls.
//!
//! ```sh
//! cargo run --example public_market_data
//! ```

use aster_client::model::{KlineInterval, PriceTicker as PriceTickerPayload};
use aster_client::{Client, Environment, futures, spot};

#[tokio::main]
async fn main() -> aster_client::Result<()> {
    let client = Client::new(Environment::Mainnet);

    client.send(futures::Ping).await?;
    let time = client.send(futures::ServerTime).await?;
    println!("server time: {} ms", time.server_time);

    // Trading rules: use filters, never `pricePrecision`/`quantityPrecision`.
    let info = client.send(futures::ExchangeInfo).await?;
    let symbol = info
        .symbol("BTCUSDT")
        .ok_or(aster_client::Error::InvalidParam("BTCUSDT"))?;
    println!(
        "BTCUSDT tick={:?} step={:?} minNotional={:?}",
        symbol.tick_size(),
        symbol.step_size(),
        symbol.min_notional()
    );
    let price: rust_decimal::Decimal = "100000.1234".parse().unwrap();
    let qty: rust_decimal::Decimal = "0.123456".parse().unwrap();
    println!(
        "quantized price {:?}, qty {:?}",
        symbol.quantize_price(price),
        symbol.quantize_qty(qty)
    );

    // Order book + candles + tickers.
    let depth = client
        .send(futures::Depth {
            symbol: "BTCUSDT".into(),
            limit: Some(5),
        })
        .await?;
    println!(
        "best bid {:?} / ask {:?}",
        depth.bids.first(),
        depth.asks.first()
    );

    let klines = client
        .send(futures::Klines::new("BTCUSDT", KlineInterval::Min1))
        .await?;
    if let Some(last) = klines.last() {
        println!("last 1m candle close={} volume={}", last.close, last.volume);
    }

    let ticker = client.send(futures::Ticker24hr::symbol("BTCUSDT")).await?;
    for ticker in ticker.as_slice() {
        println!(
            "24h {} last={} change={}%",
            ticker.symbol, ticker.last_price, ticker.price_change_percent
        );
    }

    let price: PriceTickerPayload = client
        .send(futures::PriceTicker::symbol("BTCUSDT"))
        .await?
        .into_vec()
        .pop()
        .ok_or(aster_client::Error::InvalidParam("BTCUSDT"))?;
    println!("mark price {}", price.price);

    // Spot uses the same client and models.
    let spot_book = client
        .send(spot::Depth {
            symbol: "BTCUSDT".into(),
            limit: Some(5),
        })
        .await?;
    println!("spot best ask {:?}", spot_book.asks.first());

    Ok(())
}
