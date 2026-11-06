//! Futures API (`/fapi/v3`): market data, trading, account, agent and builder.

/// account.
pub mod account;
/// agent.
pub mod agent;
/// builder.
pub mod builder;
/// market.
pub mod market;
/// trade.
pub mod trade;

pub use account::*;
pub use agent::*;
pub use builder::*;
pub use market::*;
pub use trade::*;
