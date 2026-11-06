#![warn(missing_docs)]
#![forbid(unsafe_code)]
//! Typed Rust client for the [Aster DEX](https://docs.asterdex.com) V3 API.
//!
//! # Layout
//!
//! * [`client`] — generic request pipeline: [`Client`], [`Endpoint`],
//!   [`Transport`], environments and venues.
//! * [`auth`] — EIP-712 hashing, [`EvmSigner`](auth::EvmSigner), monotonic
//!   nonces and the documented typed payloads (builder approval, withdrawal).
//! * [`params`] — canonical form encoding; the signed string is byte-identical
//!   to the string that is sent.
//! * [`futures`] — `/fapi/v3` market data, trading, account, agent and builder
//!   endpoints.
//! * [`spot`] — `/api/v3` market data, trading, account and builder endpoints.
//! * [`funding`] — withdrawals, fee estimates, history and EVM deposit calldata.
//! * [`aster_chain`] — `/aster-chain/v3` account status, transfers, staking and
//!   the withdraw/transfer flows (served by `chainapi.*`).
//! * [`ws`] — market and user-data streams with typed events.
//! * [`rpc`] — read-only Aster Chain JSON-RPC.
//!
//! # Signing model
//!
//! Every authenticated request signs the **final form-encoded parameter
//! string** as `Message(string msg)` inside the `AsterSignTransaction` domain
//! (`chainId` 1666 mainnet / 714 testnet), then appends `&signature=0x…`.
//! The client therefore always encodes once, signs those bytes, and sends the
//! same bytes — see [`params::Params::encode`].
//!
//! Management payloads (agent/builder approval, withdrawals) use their own
//! EIP-712 structs and, for withdrawals, the [`auth::Domain::withdraw`] domain
//! keyed by the destination chain id.
//!
//! # Example
//!
//! ```no_run
//! use aster_client::{Client, Environment};
//! use aster_client::futures;
//!
//! # async fn demo() -> aster_client::Result<()> {
//! let client = Client::new(Environment::Mainnet)
//!     .signer(aster_client::auth::EvmSigner::from_hex("0x…")?)
//!     .with_user("0x63DD5aCC6b1aa0f563956C0e534DD30B6dcF7C4e".parse().unwrap());
//!
//! let order = client
//!     .send(
//!         futures::NewOrder::limit("AAPLUSDT", aster_client::model::Side::Buy, "1.5", "200.25")?
//!             .time_in_force(aster_client::model::TimeInForce::Gtc)
//!             .client_order_id("a1b2"),
//!     )
//!     .await?;
//! println!("{} {:?}", order.order_id, order.status);
//! # Ok(()) }
//! ```
//!
//! # Unknown execution state
//!
//! HTTP 5xx (notably 503) and timeouts surface as
//! [`Error::UnknownState`]: the order may be live.
//! Reconcile with `newClientOrderId` — never blind-retry a write.

/// Aster-Chain API (`/aster-chain/v3`).
pub mod aster_chain;
/// auth.
pub mod auth;
/// client.
pub mod client;
/// error.
pub mod error;
/// funding.
pub mod funding;
/// futures.
pub mod futures;
/// request-weight throttling.
pub mod limits;
/// model.
pub mod model;
/// request parameters.
pub mod params;
/// rpc.
pub mod rpc;
/// spot.
pub mod spot;
/// ws.
pub mod ws;

pub use client::{
    Client, Endpoint, Environment, HttpRequest, HttpResponse, Method, ReqwestTransport, Security,
    Transport, Venue,
};
pub use error::{ApiError, Error, Result};
