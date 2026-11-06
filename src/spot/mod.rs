//! Spot API (`/api/v3`): market data, trading, account, builder.

/// builder.
pub mod builder;
/// market.
pub mod market;
/// trade.
pub mod trade;

pub use builder::*;
pub use market::*;
pub use trade::*;
