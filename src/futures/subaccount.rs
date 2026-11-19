//! Sub-account management: create, list and transfer (`/fapi/v3/*SubAccount*`).
//!
//! Like agent registration these are **pre-signed**: the signed `msg` is the
//! request body itself, in the documented field order (built with
//! [`Params::in_order`](crate::params::Params::in_order)), and the signature is a trailing parameter.
//!
//! Field order, chain ids and the double signature follow the App's working
//! implementation (`interface/lib/features/subaccounts/data/subaccount_management_repository.dart`):
//!
//! * **create** — the child signs `body` in the trading domain (1666 / 714); the
//!   owner signs `body&childSignature=0x…`, optionally in another domain that
//!   is then declared as `signatureChainId` (wallets refuse chain 1666);
//! * **transfer** — `user` is the owner, `fromAccountAddress` names the source
//!   (owner or child), signed by the owner key.

use alloy_primitives::Address;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::auth::{self, Domain, Signature, Signer, now_micros};
use crate::client::{Endpoint, Method, Security, Venue};
use crate::error::Result;
use crate::model::Ack;
use crate::params::Params;

/// `POST /fapi/v3/createSubAccount` (owner + child double signature).
#[derive(Debug, Clone)]
pub struct CreateSubAccount {
    params: Params,
}

impl CreateSubAccount {
    /// Build and sign.
    ///
    /// * `signer`: sent as `signer` — the docs say the owner's agent, the App
    ///   sends the owner address itself;
    /// * `child_chain_id`: trading domain of the environment (1666 / 714);
    /// * `owner_chain_id`: domain the owner signs in. When it differs from
    ///   `child_chain_id` it is sent as `signatureChainId` (the App uses 56).
    pub fn sign(
        owner: &dyn Signer,
        child: &dyn Signer,
        signer: Address,
        name: impl Into<String>,
        child_chain_id: u64,
        owner_chain_id: u64,
    ) -> Result<Self> {
        let mut p = Params::in_order();
        p.set_str("subAccountName", name.into());
        p.set_str("subSourceAddr", child.address().to_string());
        p.set("nonce", &now_micros());
        p.set_str("user", owner.address().to_string());
        p.set_str("signer", signer.to_string());
        if owner_chain_id != child_chain_id {
            p.set("signatureChainId", &owner_chain_id);
        }
        let body = p.encode();
        let child_signature =
            auth::sign_message(child, &Domain::aster_sign(child_chain_id), &body)?;
        p.set_str("childSignature", child_signature.to_hex());
        let owner_signature =
            auth::sign_message(owner, &Domain::aster_sign(owner_chain_id), &p.encode())?;
        p.set_str("signature", owner_signature.to_hex());
        Ok(Self { params: p })
    }
}

impl Endpoint for CreateSubAccount {
    type Response = Ack;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/fapi/v3/createSubAccount";
    const VENUE: Venue = Venue::Futures;

    fn params(&self) -> Result<Params> {
        Ok(self.params.clone())
    }

    fn weight(&self) -> u32 {
        5
    }
}

/// `GET /fapi/v3/getSubAccountList` — signed by the owner's agent.
#[derive(Debug, Clone, Copy, Default)]
pub struct SubAccountList;

impl Endpoint for SubAccountList {
    type Response = Vec<SubAccountInfo>;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/fapi/v3/getSubAccountList";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;

    fn weight(&self) -> u32 {
        5
    }
}

/// One row of [`SubAccountList`]; the owner itself is listed with `parent_account = true`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubAccountInfo {
    /// Aster account id.
    #[serde(default)]
    pub account_id: Option<i64>,
    /// Wallet address (when returned).
    #[serde(default, alias = "address", alias = "subSourceAddr")]
    pub account_address: Option<String>,
    /// Name; empty for the owner row.
    #[serde(default)]
    pub sub_account_name: Option<String>,
    /// Whether this row is the owner.
    #[serde(default)]
    pub parent_account: Option<bool>,
    /// Status (`NORMAL` / `FROZEN`), when returned.
    #[serde(default)]
    pub status: Option<String>,
}

/// Transfer direction between account types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubAccountTransferKind {
    /// Futures → futures.
    FutureFuture,
    /// Futures → spot.
    FutureSpot,
    /// Spot → futures.
    SpotFuture,
    /// Spot → spot.
    SpotSpot,
}

impl SubAccountTransferKind {
    /// Wire value.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FutureFuture => "FUTURE_FUTURE",
            Self::FutureSpot => "FUTURE_SPOT",
            Self::SpotFuture => "SPOT_FUTURE",
            Self::SpotSpot => "SPOT_SPOT",
        }
    }
}

/// `POST /fapi/v3/subAccountTransfer` — owner-signed, **not idempotent**
/// (resubmitting moves the funds again).
#[derive(Debug, Clone)]
pub struct SubAccountTransfer {
    params: Params,
}

impl SubAccountTransfer {
    /// Build and sign with the owner key. `from` is the source account (owner
    /// or one of its children); `chain_id` is the trading domain (1666 / 714).
    pub fn sign(
        owner: &dyn Signer,
        from: Address,
        to: Address,
        asset: impl Into<String>,
        amount: Decimal,
        kind: SubAccountTransferKind,
        chain_id: u64,
    ) -> Result<Self> {
        let mut p = Params::in_order();
        p.set_str("toAccountAddress", to.to_string());
        p.set_str("asset", asset.into());
        p.set("amount", &amount);
        p.set_str("kindType", kind.as_str());
        p.set("nonce", &now_micros());
        p.set_str("user", owner.address().to_string());
        p.set_str("fromAccountAddress", from.to_string());
        let signature: Signature =
            auth::sign_message(owner, &Domain::aster_sign(chain_id), &p.encode())?;
        p.set_str("signature", signature.to_hex());
        Ok(Self { params: p })
    }
}

impl Endpoint for SubAccountTransfer {
    type Response = Ack;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/fapi/v3/subAccountTransfer";
    const VENUE: Venue = Venue::Futures;

    fn params(&self) -> Result<Params> {
        Ok(self.params.clone())
    }

    fn weight(&self) -> u32 {
        5
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::EvmSigner;

    fn key(byte: u8) -> EvmSigner {
        EvmSigner::from_bytes(&[byte; 32]).unwrap()
    }

    fn split(body: &str, field: &str) -> (String, String) {
        let marker = format!("&{field}=");
        let (head, tail) = body.rsplit_once(&marker).unwrap();
        (head.to_owned(), tail.to_owned())
    }

    #[test]
    fn create_is_double_signed_over_the_documented_body() {
        let (owner, child) = (key(1), key(2));
        let request =
            CreateSubAccount::sign(&owner, &child, owner.address(), "e2e sub", 1666, 56).unwrap();
        let body = request.params().unwrap().encode();
        let (signed_by_owner, owner_sig) = split(&body, "signature");
        let (signed_by_child, child_sig) = split(&signed_by_owner, "childSignature");

        let keys: Vec<&str> = signed_by_child
            .split('&')
            .map(|kv| kv.split_once('=').unwrap().0)
            .collect();
        assert_eq!(
            keys,
            [
                "subAccountName",
                "subSourceAddr",
                "nonce",
                "user",
                "signer",
                "signatureChainId"
            ]
        );
        assert!(signed_by_child.starts_with("subAccountName=e2e+sub&subSourceAddr="));
        assert!(signed_by_child.ends_with("&signatureChainId=56"));

        let expected_child =
            auth::sign_message(&child, &Domain::aster_sign(1666), &signed_by_child).unwrap();
        assert_eq!(child_sig, expected_child.to_hex());
        let expected_owner =
            auth::sign_message(&owner, &Domain::aster_sign(56), &signed_by_owner).unwrap();
        assert_eq!(owner_sig, expected_owner.to_hex());
    }

    #[test]
    fn create_in_the_trading_domain_omits_signature_chain_id() {
        let owner = key(1);
        let request =
            CreateSubAccount::sign(&owner, &key(2), owner.address(), "s", 1666, 1666).unwrap();
        assert!(
            !request
                .params()
                .unwrap()
                .encode()
                .contains("signatureChainId")
        );
    }

    #[test]
    fn transfer_body_is_the_signed_message_in_app_order() {
        let owner = key(1);
        let child = key(2).address();
        let request = SubAccountTransfer::sign(
            &owner,
            owner.address(),
            child,
            "USDT",
            Decimal::new(6, 0),
            SubAccountTransferKind::FutureFuture,
            1666,
        )
        .unwrap();
        let body = request.params().unwrap().encode();
        let (msg, sig) = split(&body, "signature");
        let keys: Vec<&str> = msg
            .split('&')
            .map(|kv| kv.split_once('=').unwrap().0)
            .collect();
        assert_eq!(
            keys,
            [
                "toAccountAddress",
                "asset",
                "amount",
                "kindType",
                "nonce",
                "user",
                "fromAccountAddress"
            ]
        );
        assert!(msg.contains("&amount=6&kindType=FUTURE_FUTURE&"));
        let expected = auth::sign_message(&owner, &Domain::aster_sign(1666), &msg).unwrap();
        assert_eq!(sig, expected.to_hex());
    }
}
