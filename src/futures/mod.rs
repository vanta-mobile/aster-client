//! Futures API (`/fapi/v3`): market data, trading, account, agent, builder and sub-accounts.

/// account.
pub mod account;
/// agent.
pub mod agent;
/// builder.
pub mod builder;
/// market.
pub mod market;
/// sub-account.
pub mod subaccount;
/// trade.
pub mod trade;

pub use account::*;
pub use agent::*;
pub use builder::*;
pub use market::*;
pub use subaccount::*;
pub use trade::*;
