//! Futures builder-scoped queries (`/fapi/v3/builder/*`).
//!
//! The authenticated account *is* the builder: there is no `builder`
//! parameter. Every response is a [`Paginated`](crate::model::Paginated) envelope; addresses that could
//! not be returned (no deposit, privacy mode, not bound) appear in `errors`
//! instead of failing the request.

use crate::client::{Endpoint, Method, Security, Venue};
use crate::error::Result;
use crate::model::*;
use crate::params::Params;

/// Common filters for builder queries.
#[derive(Debug, Clone, Default)]
pub struct BuilderQuery {
    /// Wallet addresses to query (max 50).
    /// Up to 50 wallet addresses.
    pub user_addresses: Vec<String>,
    /// Start of the query window (inclusive, ms).
    pub start_time: Option<i64>,
    /// End of the query window (inclusive, ms).
    pub end_time: Option<i64>,
    /// Page number, starting at 1.
    pub page: Option<u32>,
    /// Page size.
    pub limit: Option<u32>,
}

impl BuilderQuery {
    /// Restrict the query to these wallet addresses.
    pub fn addresses(addresses: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            user_addresses: addresses.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }

    fn write(&self, p: &mut Params) -> Result<()> {
        if !self.user_addresses.is_empty() {
            p.set_str("userAddresses", self.user_addresses.join(","));
        }
        p.set("startTime", &self.start_time);
        p.set("endTime", &self.end_time);
        p.set("page", &self.page);
        p.set("limit", &self.limit);
        Ok(())
    }
}

macro_rules! builder_endpoints {
    ($(
        $(#[$meta:meta])*
        $name:ident => $path:literal, $response:ty $(, extra: $extra:ident)?;
    )*) => {$(
        $(#[$meta])*
        #[derive(Debug, Clone, Default)]
        pub struct $name {
            /// Common filters for the query.
            pub query: BuilderQuery,
            $(/// Extra filter specific to this query.
               pub $extra: Option<String>,)?
        }

        impl Endpoint for $name {
            type Response = $response;
            const METHOD: Method = Method::Get;
            const PATH: &'static str = $path;
            const VENUE: Venue = Venue::Futures;
            const SECURITY: Security = Security::Signer;

            fn params(&self) -> Result<Params> {
                let mut p = Params::new();
                self.query.write(&mut p)?;
                $(p.set(stringify!($extra), &self.$extra);)?
                Ok(p)
            }

            fn weight(&self) -> u32 {
                5
            }
        }
    )*};
}

builder_endpoints! {
    /// `GET /fapi/v3/builder/userAccounts`.
    BuilderUserAccounts => "/fapi/v3/builder/userAccounts", Paginated<BuilderUserAccount>, extra: symbol;
    /// `GET /fapi/v3/builder/userOpenOrders`.
    BuilderUserOpenOrders => "/fapi/v3/builder/userOpenOrders", Paginated<BuilderOrder>;
    /// `GET /fapi/v3/builder/userBalances`.
    BuilderUserBalances => "/fapi/v3/builder/userBalances", Paginated<BuilderUserBalance>;
    /// `GET /fapi/v3/builder/userPositionRisk`.
    BuilderUserPositions => "/fapi/v3/builder/userPositionRisk", Paginated<BuilderUserPositionRisk>;
    /// `GET /fapi/v3/builder/userCommissionRates`.
    BuilderUserCommissionRates => "/fapi/v3/builder/userCommissionRates", Paginated<BuilderUserCommissionRate>;
    /// `GET /fapi/v3/builder/userTrades`.
    BuilderUserTrades => "/fapi/v3/builder/userTrades", Paginated<BuilderTrade>;
    /// `GET /fapi/v3/builder/userCommissions` (works even for privacy-mode users).
    BuilderUserCommissions => "/fapi/v3/builder/userCommissions", Paginated<BuilderCommission>;
    /// `GET /fapi/v3/builder/userAllOrders`.
    BuilderUserAllOrders => "/fapi/v3/builder/userAllOrders", Paginated<BuilderOrder>;
    /// `GET /fapi/v3/builder/approvedUserList`.
    BuilderApprovedUsers => "/fapi/v3/builder/approvedUserList", Paginated<ApprovedUser>;
}
