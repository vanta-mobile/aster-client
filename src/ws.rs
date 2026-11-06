//! WebSocket market and user-data streams.
//!
//! Three layers, use the shallowest one that fits:
//!
//! - [`Streams`](crate::ws::Streams) builds a combined-stream URL.
//! - [`subscribe`](crate::ws::subscribe) / [`EventStream`](crate::ws::EventStream) give one raw connection; it ends when the
//!   server closes it (every 24h) and does not reconnect.
//! - [`Supervisor`](crate::ws::Supervisor) reconnects with backoff, re-uses the same subscription set
//!   and rotates before the 24h limit. [`UserStream`](crate::ws::UserStream) adds the `listenKey`
//!   lifecycle (60 min) on top.

use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use futures_util::{SinkExt, Stream, StreamExt};
use serde_json::Value;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

use crate::auth::{Signer, now_micros};
use crate::client::{Client, Environment, Transport, Venue};
use crate::error::{Error, Result};
use crate::model::{Event, KlineInterval};

/// Subscription list for one connection (max 200 streams, 10 msgs/s).
#[derive(Debug, Clone, Default)]
pub struct Streams {
    names: Vec<String>,
}

impl Streams {
    /// Create a new value.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a raw stream name (lowercase symbol, e.g. `btcusdt@aggTrade`).
    pub fn stream(mut self, name: impl Into<String>) -> Self {
        self.names.push(name.into());
        self
    }

    /// Add an `aggTrade` stream.
    pub fn agg_trade(self, symbol: &str) -> Self {
        self.stream(format!("{}@aggTrade", symbol.to_lowercase()))
    }

    /// candle payload.
    pub fn kline(self, symbol: &str, interval: KlineInterval) -> Self {
        self.stream(format!(
            "{}@kline_{}",
            symbol.to_lowercase(),
            interval.as_str()
        ))
    }

    /// Mark price.
    pub fn mark_price(self, symbol: &str) -> Self {
        self.stream(format!("{}@markPrice", symbol.to_lowercase()))
    }

    /// Add the all-symbols mark-price stream.
    pub fn all_mark_prices(self) -> Self {
        self.stream("!markPrice@arr")
    }

    /// Add a mini-ticker stream.
    pub fn mini_ticker(self, symbol: &str) -> Self {
        self.stream(format!("{}@miniTicker", symbol.to_lowercase()))
    }

    /// Add the all-symbols mini-ticker stream.
    pub fn all_mini_tickers(self) -> Self {
        self.stream("!miniTicker@arr")
    }

    /// Add a 24h ticker stream.
    pub fn ticker(self, symbol: &str) -> Self {
        self.stream(format!("{}@ticker", symbol.to_lowercase()))
    }

    /// Add the all-symbols ticker stream.
    pub fn all_tickers(self) -> Self {
        self.stream("!ticker@arr")
    }

    /// Add a `bookTicker` stream.
    pub fn book_ticker(self, symbol: &str) -> Self {
        self.stream(format!("{}@bookTicker", symbol.to_lowercase()))
    }

    /// Add the all-symbols `bookTicker` stream.
    pub fn all_book_tickers(self) -> Self {
        self.stream("!bookTicker")
    }

    /// Add the all-market liquidation stream.
    pub fn force_orders(self) -> Self {
        self.stream("!forceOrder@arr")
    }

    /// Partial book depth (levels: 5, 10 or 20).
    pub fn partial_depth(self, symbol: &str, levels: u32) -> Self {
        self.stream(format!("{}@depth{levels}", symbol.to_lowercase()))
    }

    /// Diff book depth; align with a REST snapshot using `U`/`u`/`pu`.
    pub fn diff_depth(self, symbol: &str) -> Self {
        self.stream(format!("{}@depth", symbol.to_lowercase()))
    }

    /// Add a TradePro stream.
    pub fn trade_pro(self, symbol: &str) -> Self {
        self.stream(format!("{}@tradepro", symbol.to_lowercase()))
    }

    /// Subscribe to a `listenKey` user-data stream.
    pub fn user_data(self, listen_key: &str) -> Self {
        self.stream(listen_key)
    }

    /// Stream names in subscription order.
    pub fn names(&self) -> &[String] {
        &self.names
    }

    /// Add one name, rejecting duplicates and the 200-stream ceiling.
    fn add(&mut self, name: &str) -> Result<bool> {
        if self.names.iter().any(|n| n == name) {
            return Ok(false);
        }
        if self.names.len() >= MAX_STREAMS {
            return Err(Error::InvalidParam("streams: over 200 per connection"));
        }
        self.names.push(name.to_owned());
        Ok(true)
    }

    fn remove(&mut self, name: &str) {
        self.names.retain(|n| n != name);
    }

    /// Whether nothing was collected.
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    /// `wss://…/ws/<name>` for one stream, `wss://…/stream?streams=a/b`
    /// for several. Returns `None` when no stream was added.
    pub fn url(&self, environment: Environment, venue: Venue) -> Option<String> {
        if self.names.is_empty() {
            return None;
        }
        let base = match venue {
            Venue::Futures => environment.futures_ws(),
            Venue::Spot => environment.spot_ws(),
            // Aster-Chain publishes no WebSocket streams; there is no stream base URL.
            Venue::Chain => return None,
        };
        Some(if self.names.len() == 1 {
            format!("{base}/ws/{}", self.names[0])
        } else {
            format!("{base}/stream?streams={}", self.names.join("/"))
        })
    }
}

/// A decoded event stream. Ends when the socket closes; reconnect externally.
///
/// Generic over the byte stream so callers can plug in their own TLS/socket
/// wrapper; [`EventStream::connect`] uses the default WebSocket transport.
pub struct EventStream<S = MaybeTlsStream<tokio::net::TcpStream>> {
    inner: WebSocketStream<S>,
}

impl EventStream<MaybeTlsStream<tokio::net::TcpStream>> {
    /// Connect to a combined-market or user-data stream URL.
    pub async fn connect(url: &str) -> Result<Self> {
        let (inner, _response) = connect_async(url)
            .await
            .map_err(|e| Error::Transport(format!("websocket connect failed: {e}")))?;
        Ok(Self { inner })
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> EventStream<S> {
    /// Send one text frame (subscription commands).
    pub async fn send_text(&mut self, text: &str) -> Result<()> {
        self.inner
            .send(Message::Text(text.to_owned().into()))
            .await
            .map_err(|e| Error::Transport(e.to_string()))
    }

    /// Send a ping frame; keeps NAT bindings and half-open sockets honest.
    pub async fn ping(&mut self) -> Result<()> {
        self.inner
            .send(Message::Ping(Vec::new().into()))
            .await
            .map_err(|e| Error::Transport(e.to_string()))
    }

    /// Send a close frame.
    pub async fn close(&mut self) -> Result<()> {
        self.inner
            .send(Message::Close(None))
            .await
            .map_err(|e| Error::Transport(e.to_string()))
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> Stream for EventStream<S> {
    type Item = Result<Event>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        loop {
            match Pin::new(&mut this.inner).poll_next(cx) {
                Poll::Ready(Some(Ok(Message::Text(text)))) => match parse_frame(text.as_str()) {
                    Ok(Some(event)) => return Poll::Ready(Some(Ok(event))),
                    Ok(None) => continue,
                    Err(error) => return Poll::Ready(Some(Err(error))),
                },
                Poll::Ready(Some(Ok(Message::Binary(_)))) => continue,
                Poll::Ready(Some(Ok(_))) => continue,
                Poll::Ready(Some(Err(error))) => {
                    return Poll::Ready(Some(Err(Error::Transport(error.to_string()))));
                }
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

/// Connect to a set of market streams.
pub async fn subscribe(
    environment: Environment,
    venue: Venue,
    streams: &Streams,
) -> Result<EventStream> {
    let url = streams
        .url(environment, venue)
        .ok_or(Error::InvalidParam("streams"))?;
    EventStream::connect(&url).await
}

/// Connect to the user-data stream of a `listenKey`.
pub async fn subscribe_user_data(
    environment: Environment,
    venue: Venue,
    listen_key: &str,
) -> Result<EventStream> {
    EventStream::connect(
        &Streams::new()
            .user_data(listen_key)
            .url(environment, venue)
            .unwrap_or_default(),
    )
    .await
}

/// Decode one frame. `Ok(None)` means a control frame (subscribe ack, error
/// reply) that carries no market event.
fn parse_frame(text: &str) -> Result<Option<Event>> {
    let value: Value = serde_json::from_str(text)?;
    let payload = value.get("data").cloned().unwrap_or(value);
    if payload.get("e").is_none() {
        return Ok(None);
    }
    Ok(Some(serde_json::from_value(payload)?))
}

/// Server limit: streams per connection.
pub const MAX_STREAMS: usize = 200;

/// Reconnect, rotation and liveness policy for [`Supervisor`].
#[derive(Debug, Clone)]
pub struct Policy {
    /// Rotate the socket before the server's 24h limit (default 23h).
    pub rotate_after: Duration,
    /// First reconnect delay; doubles per failed attempt (default 1s).
    pub initial_backoff: Duration,
    /// Backoff ceiling (default 60s).
    pub max_backoff: Duration,
    /// Give up on one connect attempt after this (default 10s).
    pub connect_timeout: Duration,
    /// Ping when no event arrived for this long (default 3 minutes).
    pub ping_every: Duration,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            rotate_after: Duration::from_secs(23 * 3_600),
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(60),
            connect_timeout: Duration::from_secs(10),
            ping_every: Duration::from_secs(3 * 60),
        }
    }
}

enum Target {
    /// Fixed URL; subscription commands are rejected.
    Url(String),
    /// Managed subscription set, used to rebuild the URL on every reconnect.
    Streams {
        environment: Environment,
        venue: Venue,
        streams: Streams,
    },
}

/// A connection that reconnects with backoff and keeps its subscription set.
///
/// - Every connect attempt after the first is paced: `initial_backoff`, doubled
///   per failure up to `max_backoff`, plus up to 25% jitter. Aster resets
///   sockets from IPs that reconnect too fast, so nothing in here ever retries
///   in a tight loop.
/// - A drop is reported **once** as an error, so the caller knows local state
///   went stale; the next [`Supervisor::next`] call reconnects, rebuilding the
///   URL from [`Supervisor::subscribe`] additions.
/// - Sockets rotate after `rotate_after` (23h default) before the server's 24h
///   cutoff, and a ping goes out after `ping_every` without events.
/// - Order books must be re-synced after any reconnect: re-fetch
///   `/fapi/v3/depth`, drop events until `U <= lastUpdateId <= u`, then require
///   `pu` continuity.
pub struct Supervisor {
    target: Target,
    policy: Policy,
    stream: Option<(EventStream, Instant)>,
    backoff: Duration,
    attempts: u32,
    connections: u32,
    request_id: u64,
}

impl Supervisor {
    /// Supervise an already built URL (no runtime subscription commands).
    pub fn new(url: impl Into<String>, policy: Policy) -> Self {
        let backoff = policy.initial_backoff;
        Self {
            target: Target::Url(url.into()),
            policy,
            stream: None,
            backoff,
            attempts: 0,
            connections: 0,
            request_id: 0,
        }
    }

    /// Supervise a market subscription set; [`Supervisor::subscribe`] and
    /// [`Supervisor::unsubscribe`] then work without reconnecting.
    pub fn market(
        environment: Environment,
        venue: Venue,
        streams: &Streams,
        policy: Policy,
    ) -> Result<Self> {
        if streams.is_empty() {
            return Err(Error::InvalidParam("streams"));
        }
        let mut supervisor = Self::new(String::new(), policy);
        supervisor.target = Target::Streams {
            environment,
            venue,
            streams: streams.clone(),
        };
        Ok(supervisor)
    }

    /// URL of the current subscription set.
    pub fn url(&self) -> Option<String> {
        match &self.target {
            Target::Url(url) => Some(url.clone()),
            Target::Streams {
                environment,
                venue,
                streams,
            } => streams.url(*environment, *venue),
        }
    }

    /// Successful connections opened so far (more than one means it reconnected).
    pub fn connections(&self) -> u32 {
        self.connections
    }

    /// Add streams. Applied to the live socket immediately, and restored after
    /// any reconnect.
    pub async fn subscribe(&mut self, streams: &Streams) -> Result<()> {
        let mut added = Vec::new();
        match &mut self.target {
            Target::Streams { streams: held, .. } => {
                for name in streams.names() {
                    if held.add(name)? {
                        added.push(name.clone());
                    }
                }
            }
            Target::Url(_) => return Err(Error::InvalidParam("streams: fixed URL")),
        }
        self.command("SUBSCRIBE", &added).await
    }

    /// Remove streams from the live socket and from the reconnect set.
    pub async fn unsubscribe(&mut self, streams: &Streams) -> Result<()> {
        let mut removed = Vec::new();
        match &mut self.target {
            Target::Streams { streams: held, .. } => {
                for name in streams.names() {
                    if held.names().iter().any(|n| n == name) {
                        held.remove(name);
                        removed.push(name.clone());
                    }
                }
            }
            Target::Url(_) => return Err(Error::InvalidParam("streams: fixed URL")),
        }
        self.command("UNSUBSCRIBE", &removed).await
    }

    /// Replace the whole subscription set and drop the socket, so the next
    /// [`Supervisor::next`] connects with the new URL (user-data key rotation).
    pub fn replace_streams(&mut self, streams: Streams) -> Result<()> {
        match &mut self.target {
            Target::Streams { streams: held, .. } => {
                if streams.is_empty() {
                    return Err(Error::InvalidParam("streams"));
                }
                *held = streams;
                self.stream = None;
                Ok(())
            }
            Target::Url(_) => Err(Error::InvalidParam("streams: fixed URL")),
        }
    }

    /// Send a subscribe/unsubscribe command to the live socket, if any.
    async fn command(&mut self, method: &str, names: &[String]) -> Result<()> {
        let Some((stream, _)) = self.stream.as_mut() else {
            return Ok(());
        };
        if names.is_empty() {
            return Ok(());
        }
        self.request_id += 1;
        let payload = serde_json::json!({
            "method": method,
            "params": names,
            "id": self.request_id,
        })
        .to_string();
        stream.send_text(&payload).await
    }

    /// Close the socket gracefully; the next [`Supervisor::next`] reconnects.
    pub async fn close(&mut self) -> Result<()> {
        let Some((mut stream, _)) = self.stream.take() else {
            return Ok(());
        };
        stream.close().await
    }

    /// Next decoded event, reconnecting, rotating and pinging as needed.
    pub async fn next(&mut self) -> Result<Event> {
        loop {
            let stale = self
                .stream
                .as_ref()
                .is_some_and(|(_, opened)| opened.elapsed() >= self.policy.rotate_after);
            if stale {
                self.stream = None;
            }

            if self.stream.is_some() {
                let ping_every = self.policy.ping_every;
                let mut ping = false;
                let item = match self.stream.as_mut() {
                    Some((stream, _)) => tokio::select! {
                        item = stream.next() => item,
                        _ = tokio::time::sleep(ping_every) => {
                            ping = true;
                            None
                        }
                    },
                    None => None,
                };

                if ping {
                    let Some((stream, _)) = self.stream.as_mut() else {
                        continue;
                    };
                    match stream.ping().await {
                        Ok(()) => continue,
                        Err(error) => {
                            self.stream = None;
                            return Err(error);
                        }
                    }
                }

                match item {
                    Some(Ok(event)) => {
                        self.backoff = self.policy.initial_backoff;
                        return Ok(event);
                    }
                    Some(Err(error)) => {
                        self.stream = None;
                        return Err(error);
                    }
                    None => {
                        self.stream = None;
                        return Err(Error::Transport("websocket closed".into()));
                    }
                }
            }

            // Every attempt after the first connection (or after a failure) is
            // paced, so a dead endpoint cannot turn into a reconnect storm.
            if self.attempts > 0 || self.connections > 0 {
                let wait = self.jittered_backoff();
                self.backoff = (self.backoff * 2).min(self.policy.max_backoff);
                tokio::time::sleep(wait).await;
            }
            self.attempts += 1;

            let Some(url) = self.url() else {
                return Err(Error::InvalidParam("streams"));
            };
            match tokio::time::timeout(self.policy.connect_timeout, EventStream::connect(&url))
                .await
            {
                Ok(Ok(stream)) => {
                    self.stream = Some((stream, Instant::now()));
                    self.connections += 1;
                    self.attempts = 0;
                    self.backoff = self.policy.initial_backoff;
                }
                Ok(Err(error)) => return Err(error),
                Err(_) => {
                    return Err(Error::Transport(format!(
                        "websocket connect timed out after {:?}",
                        self.policy.connect_timeout
                    )));
                }
            }
        }
    }

    /// Backoff never gets shorter; the jitter (0..25%) keeps clients that die
    /// together from reconnecting together.
    fn jittered_backoff(&self) -> Duration {
        let spread = self.backoff.as_nanos() / 4;
        if spread == 0 {
            return self.backoff;
        }
        let noise = u128::from(now_micros()) % (spread + 1);
        self.backoff + Duration::from_nanos(noise as u64)
    }
}

/// Futures user-data stream with the `listenKey` lifecycle built in.
///
/// A `listenKey` lives 60 minutes. [`UserStream::next`] extends it every 30
/// minutes whether or not events arrive, and on `listenKeyExpired` it requests
/// a new key and reconnects. Re-fetch account state after any reconnection:
/// events published while the socket was down are gone.
pub struct UserStream<'c, T, S> {
    client: &'c Client<T, S>,
    supervisor: Supervisor,
    keepalive_every: Duration,
    last_keepalive: Instant,
}

impl<'c, T: Transport, S: Signer> UserStream<'c, T, S> {
    /// Create a key and connect with the default [`Policy`].
    pub async fn connect(client: &'c Client<T, S>) -> Result<Self> {
        Self::with_policy(client, Policy::default()).await
    }

    /// Like [`UserStream::connect`], with an explicit reconnect policy.
    pub async fn with_policy(client: &'c Client<T, S>, policy: Policy) -> Result<Self> {
        let streams = Self::listen_key_streams(client).await?;
        Ok(Self {
            client,
            supervisor: Supervisor::market(client.environment(), Venue::Futures, &streams, policy)?,
            keepalive_every: Duration::from_secs(30 * 60),
            last_keepalive: Instant::now(),
        })
    }

    /// Change the keepalive interval (default 30 minutes; the key dies at 60).
    pub fn keepalive_every(mut self, every: Duration) -> Self {
        self.keepalive_every = every;
        self
    }

    /// Next user-data event, keeping the key alive in the meantime.
    pub async fn next(&mut self) -> Result<Event> {
        loop {
            let now = Instant::now();
            let deadline = self.last_keepalive + self.keepalive_every;
            if now >= deadline {
                self.client.send(crate::futures::KeepaliveListenKey).await?;
                self.last_keepalive = Instant::now();
                continue;
            }

            let outcome = tokio::select! {
                _ = tokio::time::sleep(deadline - now) => None,
                event = self.supervisor.next() => Some(event),
            };

            let Some(event) = outcome else {
                continue;
            };
            let event = event?;
            if matches!(event, Event::ListenKeyExpired(_)) {
                self.supervisor
                    .replace_streams(Self::listen_key_streams(self.client).await?)?;
                self.last_keepalive = Instant::now();
                continue;
            }
            return Ok(event);
        }
    }

    async fn listen_key_streams(client: &Client<T, S>) -> Result<Streams> {
        let key = client
            .send(crate::futures::StartListenKey)
            .await?
            .listen_key;
        Ok(Streams::new().user_data(&key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn subscribe_rewrites_the_reconnect_url() {
        let mut supervisor = Supervisor::market(
            Environment::Mainnet,
            Venue::Futures,
            &Streams::new().agg_trade("BTCUSDT"),
            Policy::default(),
        )
        .unwrap();
        let before = supervisor.url().unwrap();
        assert!(before.contains("btcusdt@aggTrade"));

        supervisor
            .subscribe(&Streams::new().agg_trade("ETHUSDT"))
            .await
            .unwrap();
        let after = supervisor.url().unwrap();
        assert!(after.contains("ethusdt@aggTrade"), "{after}");

        supervisor
            .unsubscribe(&Streams::new().agg_trade("BTCUSDT"))
            .await
            .unwrap();
        let reduced = supervisor.url().unwrap();
        assert!(!reduced.contains("btcusdt@aggTrade"), "{reduced}");
    }

    #[tokio::test]
    async fn subscription_limits_and_fixed_urls_are_rejected() {
        let mut streams = Streams::new();
        for i in 0..MAX_STREAMS {
            streams = streams.stream(format!("sym{i}@aggTrade"));
        }
        let mut supervisor = Supervisor::market(
            Environment::Mainnet,
            Venue::Futures,
            &streams,
            Policy::default(),
        )
        .unwrap();
        assert!(
            supervisor
                .subscribe(&Streams::new().agg_trade("ONEUSDT"))
                .await
                .is_err()
        );

        let mut fixed = Supervisor::new("wss://example.invalid/ws/x", Policy::default());
        assert!(
            fixed
                .subscribe(&Streams::new().agg_trade("BTCUSDT"))
                .await
                .is_err()
        );
        assert!(
            fixed
                .replace_streams(Streams::new().agg_trade("BTCUSDT"))
                .is_err()
        );
    }

    #[tokio::test]
    async fn outbound_frames_reach_the_socket() {
        use tokio_tungstenite::tungstenite::protocol::Role;

        let (client_io, server_io) = tokio::io::duplex(8 * 1024);
        let client = WebSocketStream::from_raw_socket(client_io, Role::Client, None).await;
        let mut server = WebSocketStream::from_raw_socket(server_io, Role::Server, None).await;
        let mut stream = EventStream { inner: client };

        stream
            .send_text(r#"{"method":"SUBSCRIBE","params":["btcusdt@aggTrade"],"id":1}"#)
            .await
            .unwrap();
        match server.next().await.unwrap().unwrap() {
            Message::Text(text) => assert!(text.contains("SUBSCRIBE")),
            other => panic!("{other:?}"),
        }

        stream.ping().await.unwrap();
        assert!(matches!(
            server.next().await.unwrap().unwrap(),
            Message::Ping(_)
        ));

        stream.close().await.unwrap();
        assert!(matches!(
            server.next().await.unwrap().unwrap(),
            Message::Close(_)
        ));
    }

    #[test]
    fn builds_combined_url() {
        let streams = Streams::new()
            .agg_trade("BTCUSDT")
            .kline("BTCUSDT", KlineInterval::Min1);
        assert_eq!(
            streams.url(Environment::Mainnet, Venue::Futures).unwrap(),
            "wss://fstream.asterdex.com/stream?streams=btcusdt@aggTrade/btcusdt@kline_1m"
        );
        let single = Streams::new().user_data("abc");
        assert_eq!(
            single.url(Environment::Testnet, Venue::Spot).unwrap(),
            "wss://sstream.asterdex-testnet.com/ws/abc"
        );
    }

    #[test]
    fn stream_builders_cover_every_channel() {
        let streams = Streams::new()
            .agg_trade("BTCUSDT")
            .kline("BTCUSDT", KlineInterval::Min1)
            .mark_price("BTCUSDT")
            .all_mark_prices()
            .mini_ticker("BTCUSDT")
            .all_mini_tickers()
            .ticker("BTCUSDT")
            .all_tickers()
            .book_ticker("BTCUSDT")
            .all_book_tickers()
            .force_orders()
            .partial_depth("BTCUSDT", 20)
            .diff_depth("BTCUSDT")
            .trade_pro("BTCUSDT");
        assert_eq!(streams.names().len(), 14);
        assert!(!streams.is_empty());
        let url = streams.url(Environment::Mainnet, Venue::Spot).unwrap();
        assert!(url.starts_with("wss://sstream.asterdex.com/stream?streams=btcusdt@aggTrade/"));
        assert!(url.contains("!forceOrder@arr"));
        assert!(url.contains("btcusdt@depth20"));
        assert!(url.contains("btcusdt@tradepro"));
        assert_eq!(
            Streams::new().url(Environment::Mainnet, Venue::Futures),
            None
        );
        assert!(Streams::new().is_empty());
    }

    #[tokio::test]
    async fn skips_binary_frames_and_reports_decode_errors() {
        use futures_util::{SinkExt, StreamExt};
        use std::time::Duration;
        use tokio_tungstenite::tungstenite::protocol::Role;

        let (client_io, server_io) = tokio::io::duplex(64 * 1024);
        let client = WebSocketStream::from_raw_socket(client_io, Role::Client, None).await;
        let mut server = WebSocketStream::from_raw_socket(server_io, Role::Server, None).await;
        let mut events = EventStream { inner: client };

        server
            .send(Message::Binary(vec![1, 2, 3].into()))
            .await
            .unwrap();
        server
            .send(Message::Text(
                r#"{"e":"aggTrade","E":1,"s":"BTCUSDT","a":2,"p":"1.5","q":"2","f":1,"l":1,"T":1,"m":true}"#
                    .into(),
            ))
            .await
            .unwrap();
        let event = tokio::time::timeout(Duration::from_secs(2), events.next())
            .await
            .expect("event")
            .unwrap()
            .unwrap();
        assert!(matches!(event, Event::AggTrade(_)));

        server.send(Message::Text("not json".into())).await.unwrap();
        let error = tokio::time::timeout(Duration::from_secs(2), events.next())
            .await
            .expect("error frame")
            .unwrap()
            .unwrap_err();
        assert!(matches!(error, Error::Decode(_)));
    }

    #[test]
    fn parses_combined_and_plain_events() {
        let combined = r#"{"stream":"btcusdt@aggTrade","data":{"e":"aggTrade","E":1,"s":"BTCUSDT","a":2,"p":"1.5","q":"2","f":1,"l":1,"T":1,"m":true}}"#;
        let event = parse_frame(combined).unwrap().unwrap();
        assert!(matches!(event, Event::AggTrade(_)));

        let ack = r#"{"result":null,"id":1}"#;
        assert!(parse_frame(ack).unwrap().is_none());
    }

    #[tokio::test]
    async fn polls_frames_and_skips_control_messages() {
        use futures_util::{SinkExt, StreamExt};
        use std::time::Duration;
        use tokio_tungstenite::tungstenite::protocol::Role;

        let (client_io, server_io) = tokio::io::duplex(64 * 1024);
        let client = WebSocketStream::from_raw_socket(client_io, Role::Client, None).await;
        let mut server = WebSocketStream::from_raw_socket(server_io, Role::Server, None).await;
        let mut events = EventStream { inner: client };

        // Subscription ack: dropped.
        server
            .send(Message::Text(r#"{"result":null,"id":1}"#.into()))
            .await
            .unwrap();
        // AggTrade: decoded.
        server
            .send(Message::Text(
                r#"{"e":"aggTrade","E":1,"s":"BTCUSDT","a":2,"p":"1.5","q":"2","f":1,"l":1,"T":1,"m":true}"#
                    .into(),
            ))
            .await
            .unwrap();
        // Unknown event type: decoded as `Event::Unknown`.
        server
            .send(Message::Text(r#"{"e":"brandNew","E":2}"#.into()))
            .await
            .unwrap();

        let first = tokio::time::timeout(Duration::from_secs(2), events.next())
            .await
            .expect("first frame")
            .unwrap()
            .unwrap();
        assert!(matches!(first, Event::AggTrade(_)));
        let second = tokio::time::timeout(Duration::from_secs(2), events.next())
            .await
            .expect("second frame")
            .unwrap()
            .unwrap();
        assert!(matches!(second, Event::Unknown));
    }

    #[test]
    fn unknown_event_does_not_fail() {
        let raw = r#"{"e":"somethingNew","E":1}"#;
        assert!(matches!(parse_frame(raw).unwrap(), Some(Event::Unknown)));
    }

    const AGG_TRADE: &str = r#"{"e":"aggTrade","E":1,"s":"BTCUSDT","a":2,"p":"1.5","q":"2","f":1,"l":1,"T":1,"m":true}"#;

    #[tokio::test]
    async fn connect_receives_server_events() {
        use futures_util::{SinkExt, StreamExt};
        use std::time::Duration;

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (tcp, _) = listener.accept().await.unwrap();
            let mut socket = tokio_tungstenite::accept_async(tcp).await.unwrap();
            socket.send(Message::Text(AGG_TRADE.into())).await.unwrap();
        });

        let mut events = EventStream::connect(&format!("ws://{addr}/ws"))
            .await
            .unwrap();
        let event = tokio::time::timeout(Duration::from_secs(2), events.next())
            .await
            .expect("event")
            .unwrap()
            .unwrap();
        assert!(matches!(event, Event::AggTrade(_)));
        server.await.unwrap();
    }

    #[tokio::test]
    async fn connect_fails_when_nothing_listens() {
        match EventStream::connect("ws://127.0.0.1:1").await {
            Err(Error::Transport(_)) => {}
            Err(other) => panic!("expected transport error, got {other}"),
            Ok(_) => panic!("expected transport error, got a connection"),
        }
    }

    #[tokio::test]
    async fn pending_when_no_frame_arrives() {
        use futures_util::StreamExt;
        use std::time::Duration;
        use tokio_tungstenite::tungstenite::protocol::Role;

        let (client_io, _server_io) = tokio::io::duplex(1024);
        let client = WebSocketStream::from_raw_socket(client_io, Role::Client, None).await;
        let mut events = EventStream { inner: client };

        let polled = tokio::time::timeout(Duration::from_millis(50), events.next()).await;
        assert!(
            polled.is_err(),
            "no frame was sent, next() must stay pending"
        );
    }

    #[tokio::test]
    async fn skips_text_control_frame() {
        use futures_util::{SinkExt, StreamExt};
        use std::time::Duration;
        use tokio_tungstenite::tungstenite::protocol::Role;

        let (client_io, server_io) = tokio::io::duplex(64 * 1024);
        let client = WebSocketStream::from_raw_socket(client_io, Role::Client, None).await;
        let mut server = WebSocketStream::from_raw_socket(server_io, Role::Server, None).await;
        let mut events = EventStream { inner: client };

        // Text frame with no `e` field: dropped by `parse_frame`.
        server.send(Message::Text("{}".into())).await.unwrap();
        server.send(Message::Text(AGG_TRADE.into())).await.unwrap();

        let event = tokio::time::timeout(Duration::from_secs(2), events.next())
            .await
            .expect("event")
            .unwrap()
            .unwrap();
        assert!(matches!(event, Event::AggTrade(_)));
    }

    #[tokio::test]
    async fn close_frame_ends_stream() {
        use futures_util::{SinkExt, StreamExt};
        use std::time::Duration;
        use tokio_tungstenite::tungstenite::protocol::Role;

        let (client_io, server_io) = tokio::io::duplex(64 * 1024);
        let client = WebSocketStream::from_raw_socket(client_io, Role::Client, None).await;
        let mut server = WebSocketStream::from_raw_socket(server_io, Role::Server, None).await;
        let mut events = EventStream { inner: client };

        // Keep driving the server so the close handshake and its reply both complete.
        let server = tokio::spawn(async move {
            server.send(Message::Close(None)).await.unwrap();
            while let Some(Ok(_)) = server.next().await {}
        });
        let end = tokio::time::timeout(Duration::from_secs(2), events.next())
            .await
            .expect("close ends stream");
        assert!(end.is_none());
        let _ = server.await;
    }

    #[tokio::test]
    async fn invalid_utf8_frame_is_a_transport_error() {
        use futures_util::StreamExt;
        use std::time::Duration;
        use tokio::io::AsyncWriteExt;
        use tokio_tungstenite::tungstenite::protocol::Role;

        let (client_io, mut server_io) = tokio::io::duplex(1024);
        let client = WebSocketStream::from_raw_socket(client_io, Role::Client, None).await;
        let mut events = EventStream { inner: client };

        // Unmasked text frame whose payload is not valid UTF-8.
        server_io
            .write_all(&[0x81, 0x02, 0xff, 0xff])
            .await
            .unwrap();
        server_io.flush().await.unwrap();

        let error = tokio::time::timeout(Duration::from_secs(2), events.next())
            .await
            .expect("error frame")
            .unwrap()
            .unwrap_err();
        assert!(matches!(error, Error::Transport(_)));
    }

    #[tokio::test]
    async fn subscribe_rejects_empty_streams() {
        match subscribe(Environment::Mainnet, Venue::Futures, &Streams::new()).await {
            Err(Error::InvalidParam("streams")) => {}
            Err(other) => panic!("expected invalid param, got {other}"),
            Ok(_) => panic!("expected invalid param, got a connection"),
        }
    }
}
