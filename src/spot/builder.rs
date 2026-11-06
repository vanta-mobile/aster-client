//! Spot builder (Aster Code) management endpoints.
//!
//! The EIP-712 payloads are shared with the futures builder endpoints —
//! [`crate::futures::agent::ApproveBuilder`] and friends — only the path and
//! venue differ. Spot caps `maxFeeRate` at `0.01`.

use rust_decimal::Decimal;
use serde::Deserialize;

use crate::client::{Endpoint, Method, Security, Venue};
use crate::error::Result;
use crate::futures::agent::{ApproveBuilder, DelBuilder, UpdateBuilder};
use crate::params::Params;

/// `POST /api/v3/approveBuilder` (spot venue).
#[derive(Debug, Clone)]
pub struct SpotApproveBuilder(pub ApproveBuilder);

impl Endpoint for SpotApproveBuilder {
    type Response = crate::model::Ack;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/api/v3/approveBuilder";
    const VENUE: Venue = Venue::Spot;

    fn params(&self) -> Result<Params> {
        self.0.params()
    }
}

/// `POST /api/v3/updateBuilder` (spot venue).
#[derive(Debug, Clone)]
pub struct SpotUpdateBuilder(pub UpdateBuilder);

impl Endpoint for SpotUpdateBuilder {
    type Response = crate::model::Ack;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/api/v3/updateBuilder";
    const VENUE: Venue = Venue::Spot;

    fn params(&self) -> Result<Params> {
        self.0.params()
    }
}

/// `DELETE /api/v3/delBuilder` (spot venue).
#[derive(Debug, Clone)]
pub struct SpotDelBuilder(pub DelBuilder);

impl Endpoint for SpotDelBuilder {
    type Response = crate::model::Ack;
    const METHOD: Method = Method::Delete;
    const PATH: &'static str = "/api/v3/delBuilder";
    const VENUE: Venue = Venue::Spot;

    fn params(&self) -> Result<Params> {
        self.0.params()
    }
}

/// `GET /api/v3/builder` — builder authorizations granted to this agent.
///
/// `maxFeeRate` is returned as a JSON number here (unlike the string decimals
/// elsewhere), so it is decoded with the decimal visitor.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuilderAuthorization {
    /// Maximum fee rate granted to the builder.
    #[serde(default)]
    pub max_fee_rate: Option<Decimal>,
    /// Builder display name.
    #[serde(default)]
    pub builder_name: Option<String>,
    /// Master wallet address.
    #[serde(default)]
    pub user: Option<String>,
    /// Aster Chain string (`Mainnet`).
    #[serde(default)]
    pub aster_chain: Option<String>,
    /// Fields not modelled by this version.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// `GET /api/v3/builder`.
#[derive(Debug, Clone, Copy, Default)]
pub struct SpotBuilderAuthorizations;

impl Endpoint for SpotBuilderAuthorizations {
    type Response = crate::model::OneOrMany<BuilderAuthorization>;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/api/v3/builder";
    const VENUE: Venue = Venue::Spot;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        Ok(Params::new())
    }
}
