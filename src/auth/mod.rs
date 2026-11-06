//! Authentication: EIP-712 hashing, signing keys, nonces and the documented
//! typed-data payloads.

/// eip712.
pub mod eip712;
/// request nonce: microsecond timestamp, strictly increasing per signer.
pub mod nonce;
/// agent wallet address.
pub mod signer;

pub use eip712::{Domain, FieldValue, Struct, keccak, message_digest, message_struct};
pub use nonce::{
    FileNonceStore, MicrosecondNonce, NonceSource, NonceStore, PersistentNonce, now_micros,
};
pub use signer::{EvmSigner, NoSigner, Signature, Signer, sign_message};

use alloy_primitives::Address;

use crate::error::Result;

/// `signatureChainId` value Aster expects for EVM wallets.
pub const SIGNATURE_CHAIN_ID_EVM: u64 = 56;
/// `signatureChainId` value Aster expects for Solana wallets.
pub const SIGNATURE_CHAIN_ID_SOLANA: u64 = 101;

/// Aster Chain identifier string used in typed payloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsterChain {
    /// Mainnet.
    Mainnet,
    /// Testnet.
    Testnet,
}

impl AsterChain {
    /// Wire representation of this value.
    pub fn as_str(self) -> &'static str {
        match self {
            AsterChain::Mainnet => "Mainnet",
            AsterChain::Testnet => "Testnet",
        }
    }
}

impl std::fmt::Display for AsterChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Sign a typed struct in the given domain.
pub fn sign_struct(
    signer: &dyn Signer,
    domain: &eip712::Domain,
    data: &Struct<'_>,
) -> Result<Signature> {
    signer.sign_digest(data.digest(domain))
}

/// Contract builder approval payload (`ApproveBuilder`).
///
/// Field order is fixed by the API: `Builder, MaxFeeRate, BuilderName,
/// AsterChain, User, Nonce`; only `Nonce` is `uint256`.
pub fn approve_builder<'a>(
    builder: &'a str,
    max_fee_rate: &'a str,
    builder_name: &'a str,
    chain: AsterChain,
    user: &'a str,
    nonce: u64,
) -> Struct<'a> {
    Struct::new("ApproveBuilder")
        .field("Builder", FieldValue::String(builder))
        .field("MaxFeeRate", FieldValue::String(max_fee_rate))
        .field("BuilderName", FieldValue::String(builder_name))
        .field("AsterChain", FieldValue::String(chain.as_str()))
        .field("User", FieldValue::String(user))
        .field("Nonce", FieldValue::Uint(nonce))
}

/// Contract builder fee update payload (`UpdateBuilder`).
pub fn update_builder<'a>(
    builder: &'a str,
    max_fee_rate: &'a str,
    chain: AsterChain,
    user: &'a str,
    nonce: u64,
) -> Struct<'a> {
    Struct::new("UpdateBuilder")
        .field("Builder", FieldValue::String(builder))
        .field("MaxFeeRate", FieldValue::String(max_fee_rate))
        .field("AsterChain", FieldValue::String(chain.as_str()))
        .field("User", FieldValue::String(user))
        .field("Nonce", FieldValue::Uint(nonce))
}

/// Contract builder revocation payload (`DelBuilder`).
pub fn del_builder<'a>(
    builder: &'a str,
    chain: AsterChain,
    user: &'a str,
    nonce: u64,
) -> Struct<'a> {
    Struct::new("DelBuilder")
        .field("Builder", FieldValue::String(builder))
        .field("AsterChain", FieldValue::String(chain.as_str()))
        .field("User", FieldValue::String(user))
        .field("Nonce", FieldValue::Uint(nonce))
}

/// Destination chain of a withdrawal; also selects the signing chain id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WithdrawChain {
    /// Ethereum.
    Ethereum,
    /// BNB Smart Chain.
    Bsc,
    /// Arbitrum One.
    Arbitrum,
}

impl WithdrawChain {
    /// chain id of the settlement network.
    pub fn chain_id(self) -> u64 {
        match self {
            WithdrawChain::Ethereum => 1,
            WithdrawChain::Bsc => 56,
            WithdrawChain::Arbitrum => 42_161,
        }
    }

    /// Wire representation of this value.
    pub fn as_str(self) -> &'static str {
        match self {
            WithdrawChain::Ethereum => "ETH",
            WithdrawChain::Bsc => "BSC",
            WithdrawChain::Arbitrum => "Arbitrum",
        }
    }
}

/// Withdrawal typed payload (`Action`) for `POST .../aster/user-withdraw`.
///
/// The signing domain must be [`eip712::Domain::withdraw`] with the destination chain id —
/// *not* the 1666/714 used by trading endpoints.
#[allow(clippy::too_many_arguments)]
pub fn withdraw_action<'a>(
    destination: Address,
    chain: WithdrawChain,
    token: &'a str,
    amount: &'a str,
    fee: &'a str,
    nonce: u64,
    aster_chain: AsterChain,
) -> Struct<'a> {
    Struct::new("Action")
        .field("type", FieldValue::String("Withdraw"))
        .field("destination", FieldValue::Address(destination))
        .field("destination Chain", FieldValue::String(chain.as_str()))
        .field("token", FieldValue::String(token))
        .field("amount", FieldValue::String(amount))
        .field("fee", FieldValue::String(fee))
        .field("nonce", FieldValue::Uint(nonce))
        .field("aster chain", FieldValue::String(aster_chain.as_str()))
}
