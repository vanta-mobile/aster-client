//! Everything the Pro futures trade screen (`/trade/pro/futures/BTCUSDT`) shows,
//! in one REST snapshot, plus the exact WebSocket stream set that screen uses.
//!
//! ```sh
//! cargo run --example trade_screen
//! ```

use aster_client::model::KlineInterval;
use aster_client::{Client, Environment, futures};

#[tokio::main]
async fn main() -> aster_client::Result<()> {
    let client = Client::new(Environment::Mainnet);
    let symbol = "BTCUSDT";

    // Trading rules and the symbol header's max leverage.
    let info = client.send(futures::ExchangeInfo).await?;
    let Some(meta) = info.symbol(symbol) else {
        return Err(aster_client::Error::InvalidParam(symbol));
    };
    println!(
        "meta  {} status={:?} tick={:?} step={:?} minNotional={:?} maxQty={:?}",
        meta.symbol,
        meta.status,
        meta.tick_size(),
        meta.step_size(),
        meta.min_notional(),
        meta.max_qty()
    );

    // Header: mark / index / funding countdown.
    let premium = client
        .send(futures::PremiumIndex {
            symbol: Some(symbol.into()),
        })
        .await?;
    for p in premium.as_slice().iter() {
        println!(
            "mark  {} mark={} index={} funding={:?} nextFunding={:?}",
            p.symbol, p.mark_price, p.index_price, p.last_funding_rate, p.next_funding_time
        );
    }

    // Symbol list row.
    for t in client
        .send(futures::Ticker24hr::symbol(symbol))
        .await?
        .as_slice()
        .iter()
    {
        println!(
            "24h   {} last={} high={} low={} vol={}",
            t.symbol, t.last_price, t.high_price, t.low_price, t.volume
        );
    }

    // Order book (page loads 1000 levels) and the trades tape.
    let depth = client
        .send(futures::Depth {
            symbol: symbol.into(),
            limit: Some(1000),
        })
        .await?;
    println!(
        "book  {} asks={} bids={} best={:?}/{:?}",
        symbol,
        depth.asks.len(),
        depth.bids.len(),
        depth.bids.first(),
        depth.asks.first()
    );
    let trades = client
        .send(futures::AggTrades {
            symbol: symbol.into(),
            from_id: None,
            start_time: None,
            end_time: None,
            limit: Some(80),
        })
        .await?;
    println!("tape  {} trades, last={:?}", trades.len(), trades.last());

    // Chart: 1d candles, same interval the page opens with.
    let klines = client
        .send(futures::Klines {
            symbol: symbol.into(),
            interval: KlineInterval::Day1,
            start_time: None,
            end_time: None,
            limit: Some(500),
        })
        .await?;
    println!(
        "klines {} candles, last close={:?}",
        klines.len(),
        klines.last().map(|k| k.close)
    );

    // Funding interval / cap, needed for the funding tooltip.
    for f in client
        .send(futures::FundingInfo {
            symbol: Some(symbol.into()),
        })
        .await?
    {
        println!(
            "fund  {} interval={}h cap={}",
            f.symbol, f.funding_interval_hours, f.funding_fee_cap
        );
    }

    // "Max openable" preview: needs a leverage tier (`leverageBrackets` is
    // signed, so it is left to the caller).
    let remaining = client
        .send(futures::RemainingOpenableNotionalValue {
            symbol: symbol.into(),
            leverage: 20,
        })
        .await?;
    println!(
        "open  {} remainingNotional={}",
        symbol, remaining.remaining_openable_notional_value
    );

    // The page subscribes on one combined connection and switches symbol by
    // re-subscribing; `@500ms` depth is not modelled yet, so pass it raw.
    let streams = aster_client::ws::Streams::new()
        .stream("btcusdt@aggTrade")
        .stream("btcusdt@depth@500ms")
        .kline(symbol, KlineInterval::Day1)
        .all_mark_prices()
        .all_mini_tickers();
    println!(
        "ws    {}",
        streams
            .url(Environment::Mainnet, aster_client::Venue::Futures)
            .unwrap_or_default()
    );

    Ok(())
}
