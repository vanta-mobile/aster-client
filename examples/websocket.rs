//! WebSocket market streams through the supervisor (reconnects, rotates at 23h)
//! plus typed event decoding.
//!
//! ```sh
//! cargo run --example websocket
//! ```

use aster_client::model::{Event, KlineInterval};
use aster_client::ws::{Policy, Streams, Supervisor};
use aster_client::{Environment, Venue};

#[tokio::main]
async fn main() -> aster_client::Result<()> {
    let streams = Streams::new()
        .agg_trade("BTCUSDT")
        .kline("BTCUSDT", KlineInterval::Min1)
        .book_ticker("BTCUSDT");

    let mut events = Supervisor::market(
        Environment::Mainnet,
        Venue::Futures,
        &streams,
        Policy::default(),
    )?;
    println!("connecting: {}", events.url().unwrap_or_default());

    let mut seen = 0;
    loop {
        let event = match events.next().await {
            Ok(event) => event,
            // One error per drop; the next call reconnects. Re-sync order books
            // and account state here.
            Err(error) => {
                println!(
                    "disconnected ({error}); connections so far {}",
                    events.connections()
                );
                continue;
            }
        };
        match event {
            Event::AggTrade(trade) => println!(
                "aggTrade {} {} @ {} (maker {})",
                trade.symbol, trade.qty, trade.price, trade.buyer_is_maker
            ),
            Event::Kline(kline) => println!(
                "kline {} close={} closed={}",
                kline.symbol, kline.kline.close, kline.kline.closed
            ),
            Event::BookTicker(book) => {
                println!(
                    "book {} {} / {}",
                    book.symbol, book.bid_price, book.ask_price
                )
            }
            other => println!("other event: {other:?}"),
        }
        seen += 1;
        if seen == 10 {
            break;
        }
    }
    println!("connections used: {}", events.connections());
    Ok(())
}
