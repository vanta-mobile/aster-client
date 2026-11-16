//! Agent (API wallet) registration and builder approval.
//!
//! These payloads are **not** signed like ordinary requests: the signature is
//! a standalone request parameter computed over a documented field order, with
//! `domain.chainId` set to `signatureChainId` (56 for EVM, 101 for Solana) —
//! not 1666/714. Keys passed here are the **user's master wallet** key.

use alloy_primitives::Address;

use crate::auth::{self, AsterChain, Domain, Signature, Signer, now_micros, SIGNATURE_CHAIN_ID_EVM};
use crate::client::{Endpoint, Method, Venue};
use crate::error::{Error, Result};
use crate::model::*;
use crate::params::Params;

/// `POST /fapi/v3/registerAndApproveAgent` (public endpoint, user-signed).
///
/// Build it with [`RegisterAndApproveAgent::sign`], which generates the nonce,
/// builds the documented `msg` string and signs it in the
/// `signatureChainId` domain.
#[derive(Debug, Clone)]
pub struct RegisterAndApproveAgent {
    /// Master wallet address.
    pub user: String,
    /// Agent display name.
    pub agent_name: String,
    /// Agent wallet address.
    pub agent_address: String,
    /// Agent wallet address.
    /// Millisecond timestamp after which the agent expires.
    pub expired: i64,
    /// Signature Chain Id.
    /// Expiry timestamp in milliseconds.
    /// 56 for EVM wallets, 101 for Solana wallets.
    pub signature_chain_id: u64,
    /// Whether the agent may place spot orders.
    pub can_spot_trade: bool,
    /// Whether the agent may place perpetual orders.
    pub can_perp_trade: bool,
    /// Whether the account can withdraw.
    pub can_withdraw: bool,
    /// Space-separated IPs/CIDRs; required when `canWithdraw` is true.
    /// Whether the account can withdraw.
    /// Space-separated IPs/CIDRs; mandatory when `can_withdraw` is true.
    pub ip_whitelist: Option<String>,
    /// Agent Code.
    pub agent_code: Option<String>,
    /// Request nonce: microsecond timestamp, strictly increasing per signer.
    pub nonce: u64,
    /// EIP-712 signature.
    pub signature: Signature,
}

/// Fields needed to register an agent, before signing.
#[derive(Debug, Clone)]
pub struct RegisterAgentRequest {
    /// Master wallet address.
    pub user: Address,
    /// Agent display name.
    pub agent_name: String,
    /// Agent wallet address.
    pub agent_address: Address,
    /// Expiry timestamp in milliseconds.
    pub expired: i64,
    /// Whether the agent may place spot orders.
    pub can_spot_trade: bool,
    /// Whether the agent may place perpetual orders.
    pub can_perp_trade: bool,
    /// Whether the account can withdraw.
    pub can_withdraw: bool,
    /// Space-separated IPs/CIDRs; required when `canWithdraw` is true.
    pub ip_whitelist: Option<String>,
    /// Agent Code.
    pub agent_code: Option<String>,
    /// Chain id used for the signature domain (56 EVM, 101 Solana).
    pub signature_chain_id: u64,
}

impl RegisterAndApproveAgent {
    /// Sign the registration message with the **user's** wallet key.
    pub fn sign(signer: &dyn Signer, request: &RegisterAgentRequest) -> Result<Self> {
        if request.can_withdraw
            && request
                .ip_whitelist
                .as_deref()
                .map(str::trim)
                .unwrap_or_default()
                .is_empty()
        {
            return Err(Error::InvalidParam("ipWhitelist"));
        }
        let nonce = now_micros();
        let ip_whitelist = request.ip_whitelist.clone().unwrap_or_default();
        let msg = format!(
            "user={}&nonce={}&agentName={}&agentAddress={}&expired={}&signatureChainId={}&canSpotTrade={}&canPerpTrade={}&canWithdraw={}&ipWhitelist={}",
            request.user,
            nonce,
            request.agent_name,
            request.agent_address,
            request.expired,
            request.signature_chain_id,
            request.can_spot_trade,
            request.can_perp_trade,
            request.can_withdraw,
            ip_whitelist,
        );
        let domain = Domain::aster_sign(request.signature_chain_id);
        let signature = auth::sign_message(signer, &domain, &msg)?;
        Ok(Self {
            user: request.user.to_string(),
            agent_name: request.agent_name.clone(),
            agent_address: request.agent_address.to_string(),
            expired: request.expired,
            signature_chain_id: request.signature_chain_id,
            can_spot_trade: request.can_spot_trade,
            can_perp_trade: request.can_perp_trade,
            can_withdraw: request.can_withdraw,
            ip_whitelist: request.ip_whitelist.clone(),
            agent_code: request.agent_code.clone(),
            nonce,
            signature,
        })
    }
}

impl Endpoint for RegisterAndApproveAgent {
    type Response = Ack;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/fapi/v3/registerAndApproveAgent";
    const VENUE: Venue = Venue::Futures;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("user", &self.user);
        p.set("nonce", &self.nonce);
        p.set("agentName", &self.agent_name);
        p.set("agentAddress", &self.agent_address);
        p.set("expired", &self.expired);
        p.set("signatureChainId", &self.signature_chain_id);
        p.set("canSpotTrade", &self.can_spot_trade);
        p.set("canPerpTrade", &self.can_perp_trade);
        p.set("canWithdraw", &self.can_withdraw);
        p.set("ipWhitelist", &self.ip_whitelist);
        p.set("agentCode", &self.agent_code);
        p.set_str("signature", self.signature.to_hex());
        Ok(p)
    }

    fn weight(&self) -> u32 {
        50
    }
}

/// `GET /fapi/v3/agent` — agents approved for the authenticated account.
///
/// The provided docs do not state the security type; it is modelled as an
/// agent-signed `USER_DATA` endpoint.
#[derive(Debug, Clone, Copy, Default)]
pub struct AgentList;

impl Endpoint for AgentList {
    type Response = Vec<AgentInfo>;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/agent";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: crate::client::Security = crate::client::Security::Signer;
}

/// Contract builder approval (`ApproveBuilder`).
///
/// `signature` must be produced with [`ApproveBuilder::sign`] (master wallet).
#[derive(Debug, Clone)]
pub struct ApproveBuilder {
    /// Builder.
    pub builder: String,
    /// Maximum fee rate granted to the builder.
    pub max_fee_rate: String,
    /// Builder display name.
    pub builder_name: String,
    /// Settlement chain.
    pub chain: AsterChain,
    /// Master wallet address.
    pub user: String,
    /// Request nonce: microsecond timestamp, strictly increasing per signer.
    pub nonce: u64,
    /// EIP-712 signature.
    pub signature: Signature,
}

impl ApproveBuilder {
    /// Build and sign the payload with the user's master wallet key.
    pub fn sign(
        signer: &dyn Signer,
        builder: impl Into<String>,
        max_fee_rate: impl Into<String>,
        builder_name: impl Into<String>,
        chain: AsterChain,
        nonce: u64,
    ) -> Result<Self> {
        let builder = builder.into();
        let max_fee_rate = max_fee_rate.into();
        let builder_name = builder_name.into();
        let user = signer.address().to_string();
        let data =
            auth::approve_builder(&builder, &max_fee_rate, &builder_name, chain, &user, nonce);
        let signature = auth::sign_struct(signer, &Domain::aster_sign(SIGNATURE_CHAIN_ID_EVM), &data)?;
        Ok(Self {
            builder,
            max_fee_rate,
            builder_name,
            chain,
            user,
            nonce,
            signature,
        })
    }

    fn write(&self, p: &mut Params) {
        p.set("builder", &self.builder);
        p.set("maxFeeRate", &self.max_fee_rate);
        p.set("builderName", &self.builder_name);
        p.set("asterChain", &self.chain.as_str());
        p.set("user", &self.user);
        p.set("nonce", &self.nonce);
        // Sent as a request parameter but NOT part of the signed struct; must
        // equal the signing domain's chain id (56 EVM, 101 Solana).
        p.set("signatureChainId", &SIGNATURE_CHAIN_ID_EVM);
        p.set_str("signature", self.signature.to_hex());
    }
}

impl Endpoint for ApproveBuilder {
    type Response = Ack;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/fapi/v3/approveBuilder";
    const VENUE: Venue = Venue::Futures;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        self.write(&mut p);
        Ok(p)
    }
}

/// Contract builder fee update (`UpdateBuilder`).
#[derive(Debug, Clone)]
pub struct UpdateBuilder {
    /// Builder.
    pub builder: String,
    /// Maximum fee rate granted to the builder.
    pub max_fee_rate: String,
    /// Settlement chain.
    pub chain: AsterChain,
    /// Master wallet address.
    pub user: String,
    /// Request nonce: microsecond timestamp, strictly increasing per signer.
    pub nonce: u64,
    /// EIP-712 signature.
    pub signature: Signature,
}

impl UpdateBuilder {
    /// Build and sign the payload with the user's master wallet key.
    pub fn sign(
        signer: &dyn Signer,
        builder: impl Into<String>,
        max_fee_rate: impl Into<String>,
        chain: AsterChain,
        nonce: u64,
    ) -> Result<Self> {
        let builder = builder.into();
        let max_fee_rate = max_fee_rate.into();
        let user = signer.address().to_string();
        let data = auth::update_builder(&builder, &max_fee_rate, chain, &user, nonce);
        let signature = auth::sign_struct(signer, &Domain::aster_sign(SIGNATURE_CHAIN_ID_EVM), &data)?;
        Ok(Self {
            builder,
            max_fee_rate,
            chain,
            user,
            nonce,
            signature,
        })
    }
}

impl Endpoint for UpdateBuilder {
    type Response = Ack;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/fapi/v3/updateBuilder";
    const VENUE: Venue = Venue::Futures;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("builder", &self.builder);
        p.set("maxFeeRate", &self.max_fee_rate);
        p.set("asterChain", &self.chain.as_str());
        p.set("user", &self.user);
        p.set("nonce", &self.nonce);
        p.set("signatureChainId", &SIGNATURE_CHAIN_ID_EVM);
        p.set_str("signature", self.signature.to_hex());
        Ok(p)
    }
}

/// Contract builder revocation (`DelBuilder`).
#[derive(Debug, Clone)]
pub struct DelBuilder {
    /// Builder.
    pub builder: String,
    /// Settlement chain.
    pub chain: AsterChain,
    /// Master wallet address.
    pub user: String,
    /// Request nonce: microsecond timestamp, strictly increasing per signer.
    pub nonce: u64,
    /// EIP-712 signature.
    pub signature: Signature,
}

impl DelBuilder {
    /// Build and sign the payload with the user's master wallet key.
    pub fn sign(
        signer: &dyn Signer,
        builder: impl Into<String>,
        chain: AsterChain,
        nonce: u64,
    ) -> Result<Self> {
        let builder = builder.into();
        let user = signer.address().to_string();
        let data = auth::del_builder(&builder, chain, &user, nonce);
        let signature = auth::sign_struct(signer, &Domain::aster_sign(SIGNATURE_CHAIN_ID_EVM), &data)?;
        Ok(Self {
            builder,
            chain,
            user,
            nonce,
            signature,
        })
    }
}

impl Endpoint for DelBuilder {
    type Response = Ack;
    const METHOD: Method = Method::Delete;
    const PATH: &'static str = "/fapi/v3/builder";
    const VENUE: Venue = Venue::Futures;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("builder", &self.builder);
        p.set("asterChain", &self.chain.as_str());
        p.set("user", &self.user);
        p.set("nonce", &self.nonce);
        p.set("signatureChainId", &SIGNATURE_CHAIN_ID_EVM);
        p.set_str("signature", self.signature.to_hex());
        Ok(p)
    }
}

pub use crate::model::AgentInfo as AgentInfoPayload;
