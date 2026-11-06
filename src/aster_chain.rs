//! Aster-Chain API (`/aster-chain/v3`, host `chainapi.*`).
//!
//! A third documented surface next to futures (`/fapi/v3`) and spot (`/api/v3`).
//! It owns account privacy status, address transfers, staking, deposit
//! addresses and the withdraw/transfer flows.
//!
//! Authentication follows the same V3 model as the other surfaces: `USER_DATA`
//! requests carry `nonce` + `signer` + `signature` ([`Security::Signer`]),
//! `TRADE`/`WITHDRAW` requests carry `nonce` + `user` + `signature`
//! ([`Security::User`]).
//!
//! Withdrawals are the exception: they carry the **wallet** signature
//! (`userNonce` + `userSignature`, see [`crate::funding::SignedWithdraw`])
//! and no request-level signature, matching the futures/spot withdraw flows.
//!
//! ```no_run
//! use aster_client::{Client, Environment, aster_chain};
//!
//! # async fn demo() -> aster_client::Result<()> {
//! let client = Client::new(Environment::Mainnet)
//!     .signer(aster_client::auth::EvmSigner::from_hex("0x…")?)
//!     .with_user("0x…".parse().unwrap());
//!
//! let status = client.send(aster_chain::AccountStatus).await?;
//! println!("{}", status.status);
//! # Ok(()) }
//! ```

use rust_decimal::Decimal;

use crate::client::{Endpoint, Method, Security, Venue};
use crate::error::{Error, Result};
use crate::funding::SignedWithdraw;
use crate::model::*;
use crate::params::Params;

// ---------------------------------------------------------------- account ---

/// `GET /aster-chain/v3/account/status`.
#[derive(Debug, Clone, Copy, Default)]
pub struct AccountStatus;

impl Endpoint for AccountStatus {
    type Response = ChainAccountStatus;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/aster-chain/v3/account/status";
    const VENUE: Venue = Venue::Chain;
    const SECURITY: Security = Security::Signer;
}

/// `POST /aster-chain/v3/account/modify-status`.
///
/// Switches the account between `PUBLIC` and `PRIVATE`. A `PRIVATE` account
/// cannot use batch transfer.
#[derive(Debug, Clone)]
pub struct ModifyAccountStatus {
    /// Target status: `PUBLIC` or `PRIVATE`.
    pub status: String,
}

impl Endpoint for ModifyAccountStatus {
    type Response = ChainAccountStatus;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/aster-chain/v3/account/modify-status";
    const VENUE: Venue = Venue::Chain;
    const SECURITY: Security = Security::User;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set_str("status", self.status.clone());
        Ok(p)
    }
}

// --------------------------------------------------------------- transfer ---

/// `POST /aster-chain/v3/transfer`.
#[derive(Debug, Clone)]
pub struct Transfer {
    /// Asset to transfer.
    pub asset: String,
    /// Amount.
    pub amount: Decimal,
    /// Destination Aster-Chain address.
    pub to_address: String,
    /// Optional idempotency key (20–64 chars).
    pub client_tran_id: Option<String>,
}

impl Endpoint for Transfer {
    type Response = ChainTransferResult;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/aster-chain/v3/transfer";
    const VENUE: Venue = Venue::Chain;
    const SECURITY: Security = Security::User;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set_str("asset", self.asset.clone());
        p.set("amount", &self.amount);
        p.set_str("toAddress", self.to_address.clone());
        if let Some(id) = &self.client_tran_id {
            p.set_str("clientTranId", id.clone());
        }
        Ok(p)
    }
}

/// `POST /aster-chain/v3/batch-transfer`.
///
/// At most 10 items. `client_tran_id` is required and makes the call
/// idempotent: resubmitting it returns the existing batch. Not available while
/// the account is `PRIVATE`.
#[derive(Debug, Clone)]
pub struct BatchTransfer {
    /// Idempotency key (20–64 chars), required.
    pub client_tran_id: String,
    /// Destinations, at most 10.
    pub items: Vec<BatchTransferItem>,
}

impl Endpoint for BatchTransfer {
    type Response = BatchTransferResult;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/aster-chain/v3/batch-transfer";
    const VENUE: Venue = Venue::Chain;
    const SECURITY: Security = Security::User;

    fn params(&self) -> Result<Params> {
        let items = serde_json::to_string(&self.items)
            .map_err(|_| Error::InvalidParam("items are not serializable"))?;
        let mut p = Params::new();
        p.set_str("clientTranId", self.client_tran_id.clone());
        p.set_str("items", items);
        Ok(p)
    }

    fn weight(&self) -> u32 {
        50
    }
}

/// `GET /aster-chain/v3/batch-transfer/history`.
///
/// Sending only `client_tran_id` returns that batch regardless of age;
/// otherwise results are filtered by creation time (default last 7 days, at
/// most 31 days).
#[derive(Debug, Clone, Default)]
pub struct BatchTransferHistoryQuery {
    /// Return one batch by its idempotency key.
    pub client_tran_id: Option<String>,
    /// Filter by batch status: `I`/`P`/`S`/`F`/`M`.
    pub status: Option<String>,
    /// Inclusive start timestamp in milliseconds.
    pub start_time: Option<i64>,
    /// Exclusive end timestamp in milliseconds.
    pub end_time: Option<i64>,
    /// Page number, starting at 1.
    pub page: Option<u32>,
    /// Page size, default 20, max 100.
    pub limit: Option<u32>,
}

impl Endpoint for BatchTransferHistoryQuery {
    type Response = BatchTransferHistory;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/aster-chain/v3/batch-transfer/history";
    const VENUE: Venue = Venue::Chain;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        if let Some(v) = &self.client_tran_id {
            p.set_str("clientTranId", v.clone());
        }
        if let Some(v) = &self.status {
            p.set_str("status", v.clone());
        }
        if let Some(v) = self.start_time {
            p.set("startTime", &v);
        }
        if let Some(v) = self.end_time {
            p.set("endTime", &v);
        }
        if let Some(v) = self.page {
            p.set("page", &v);
        }
        if let Some(v) = self.limit {
            p.set("limit", &v);
        }
        Ok(p)
    }

    fn weight(&self) -> u32 {
        5
    }
}

// ---------------------------------------------------------------- staking ---

/// `GET /aster-chain/v3/staking/stakeAccountStatus`.
#[derive(Debug, Clone, Copy, Default)]
pub struct StakingAccountStatus;

impl Endpoint for StakingAccountStatus {
    type Response = crate::model::StakingAccountStatus;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/aster-chain/v3/staking/stakeAccountStatus";
    const VENUE: Venue = Venue::Chain;
    const SECURITY: Security = Security::Signer;
}

/// `GET /aster-chain/v3/staking/myStaking`.
#[derive(Debug, Clone, Copy, Default)]
pub struct MyStaking;

impl Endpoint for MyStaking {
    type Response = crate::model::MyStaking;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/aster-chain/v3/staking/myStaking";
    const VENUE: Venue = Venue::Chain;
    const SECURITY: Security = Security::Signer;
}

/// `GET /aster-chain/v3/staking/claimableRewards`.
#[derive(Debug, Clone, Copy, Default)]
pub struct ClaimableRewards;

impl Endpoint for ClaimableRewards {
    type Response = crate::model::ClaimableRewards;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/aster-chain/v3/staking/claimableRewards";
    const VENUE: Venue = Venue::Chain;
    const SECURITY: Security = Security::Signer;
}

/// `POST /aster-chain/v3/staking/create`.
#[derive(Debug, Clone)]
pub struct CreateStaking {
    /// Validator to delegate to.
    pub validator_address: String,
    /// Amount to stake.
    pub stake_amount: Decimal,
    /// Lock period code, e.g. `90D`.
    pub period_code: String,
}

impl Endpoint for CreateStaking {
    type Response = StakingActionResult;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/aster-chain/v3/staking/create";
    const VENUE: Venue = Venue::Chain;
    const SECURITY: Security = Security::User;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set_str("validatorAddress", self.validator_address.clone());
        p.set("stakeAmount", &self.stake_amount);
        p.set_str("periodCode", self.period_code.clone());
        Ok(p)
    }
}

/// `POST /aster-chain/v3/staking/deposit`.
#[derive(Debug, Clone)]
pub struct DepositStake {
    /// Validator to top up.
    pub validator_address: String,
    /// Additional amount to stake.
    pub stake_amount: Decimal,
}

impl Endpoint for DepositStake {
    type Response = StakingActionResult;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/aster-chain/v3/staking/deposit";
    const VENUE: Venue = Venue::Chain;
    const SECURITY: Security = Security::User;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set_str("validatorAddress", self.validator_address.clone());
        p.set("stakeAmount", &self.stake_amount);
        Ok(p)
    }
}

/// `POST /aster-chain/v3/staking/updateLockPeriod`.
#[derive(Debug, Clone)]
pub struct UpdateLockPeriod {
    /// New lock period code.
    pub period_code: String,
}

impl Endpoint for UpdateLockPeriod {
    type Response = StakingActionResult;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/aster-chain/v3/staking/updateLockPeriod";
    const VENUE: Venue = Venue::Chain;
    const SECURITY: Security = Security::User;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set_str("periodCode", self.period_code.clone());
        Ok(p)
    }
}

/// `POST /aster-chain/v3/staking/claimRewards`.
#[derive(Debug, Clone)]
pub struct ClaimRewards {
    /// Amount to claim.
    pub requested_amount: Decimal,
}

impl Endpoint for ClaimRewards {
    type Response = StakingActionResult;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/aster-chain/v3/staking/claimRewards";
    const VENUE: Venue = Venue::Chain;
    const SECURITY: Security = Security::User;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("requestedAmount", &self.requested_amount);
        Ok(p)
    }
}

/// `GET /aster-chain/v3/staking/getLockedAster` — public, no credentials.
///
/// Reports the platform-wide volume locked under one lock period.
#[derive(Debug, Clone, Copy, Default)]
pub struct LockedAster;

impl Endpoint for LockedAster {
    type Response = LockedAsterPeriod;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/aster-chain/v3/staking/getLockedAster";
    const VENUE: Venue = Venue::Chain;
}

// ---------------------------------------------------------------- deposit ---

/// `GET /aster-chain/v3/spot/user-deposit-address`.
#[derive(Debug, Clone, Default)]
pub struct UserDepositAddress {
    /// Optional network filter.
    pub network: Option<String>,
}

impl Endpoint for UserDepositAddress {
    type Response = DepositAddress;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/aster-chain/v3/spot/user-deposit-address";
    const VENUE: Venue = Venue::Chain;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        if let Some(v) = &self.network {
            p.set_str("network", v.clone());
        }
        Ok(p)
    }
}

// ------------------------------------------------------------ withdrawals ---

/// One withdraw/transfer request: the wallet-signed action plus an optional
/// signature scheme selector.
#[derive(Debug, Clone)]
pub struct WithdrawRequest {
    /// Wallet-signed withdrawal, built with
    /// [`crate::funding::WithdrawIntent::sign`].
    pub withdrawal: SignedWithdraw,
    /// Optional `signatureType` selector.
    pub signature_type: Option<String>,
}

impl WithdrawRequest {
    fn write(&self, p: &mut Params) {
        self.withdrawal.write(p);
        if let Some(v) = &self.signature_type {
            p.set_str("signatureType", v.clone());
        }
    }
}

/// `POST /aster-chain/v3/perp/user-withdraw` — EVM destination.
#[derive(Debug, Clone)]
pub struct PerpWithdraw(pub WithdrawRequest);

impl Endpoint for PerpWithdraw {
    type Response = WithdrawResult;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/aster-chain/v3/perp/user-withdraw";
    const VENUE: Venue = Venue::Chain;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        self.0.write(&mut p);
        Ok(p)
    }
}

/// `POST /aster-chain/v3/perp/user-solana-withdraw`.
#[derive(Debug, Clone)]
pub struct PerpSolanaWithdraw(pub WithdrawRequest);

impl Endpoint for PerpSolanaWithdraw {
    type Response = WithdrawResult;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/aster-chain/v3/perp/user-solana-withdraw";
    const VENUE: Venue = Venue::Chain;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        self.0.write(&mut p);
        Ok(p)
    }
}

/// `POST /aster-chain/v3/spot/user-withdraw` — EVM destination.
#[derive(Debug, Clone)]
pub struct SpotWithdraw(pub WithdrawRequest);

impl Endpoint for SpotWithdraw {
    type Response = WithdrawResult;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/aster-chain/v3/spot/user-withdraw";
    const VENUE: Venue = Venue::Chain;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        self.0.write(&mut p);
        Ok(p)
    }
}

/// `POST /aster-chain/v3/spot/user-solana-withdraw`.
#[derive(Debug, Clone)]
pub struct SpotSolanaWithdraw(pub WithdrawRequest);

impl Endpoint for SpotSolanaWithdraw {
    type Response = WithdrawResult;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/aster-chain/v3/spot/user-solana-withdraw";
    const VENUE: Venue = Venue::Chain;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        self.0.write(&mut p);
        Ok(p)
    }
}

/// `GET /aster-chain/v3/perp/user-withdraw-info`.
///
/// Withdraw limits per asset and chain for the current account.
#[derive(Debug, Clone, Copy, Default)]
pub struct UserWithdrawInfo;

impl Endpoint for UserWithdrawInfo {
    type Response = crate::model::UserWithdrawInfo;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/aster-chain/v3/perp/user-withdraw-info";
    const VENUE: Venue = Venue::Chain;
    const SECURITY: Security = Security::Signer;
}

/// `GET /aster-chain/v3/perp/deposit-withdraw-history`.
#[derive(Debug, Clone, Default)]
pub struct DepositWithdrawHistory {
    /// Optional chain filter.
    pub chain_id: Option<i64>,
}

impl Endpoint for DepositWithdrawHistory {
    type Response = Vec<DepositWithdrawRecord>;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/aster-chain/v3/perp/deposit-withdraw-history";
    const VENUE: Venue = Venue::Chain;
    const SECURITY: Security = Security::Signer;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        if let Some(v) = self.chain_id {
            p.set_str("chainId", v.to_string());
        }
        Ok(p)
    }
}

/// `GET /aster-chain/v3/withdraw/estimateFee`.
///
/// Public: no credentials. `gas_cost` is the fee to state in the matching
/// withdrawal intent — the server rejects a stale fee.
#[derive(Debug, Clone)]
pub struct EstimateWithdrawFee {
    /// Destination chain id.
    pub chain_id: i64,
    /// Asset.
    pub asset: String,
}

impl Endpoint for EstimateWithdrawFee {
    type Response = ChainWithdrawEstimateFee;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/aster-chain/v3/withdraw/estimateFee";
    const VENUE: Venue = Venue::Chain;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("chainId", &self.chain_id);
        p.set_str("asset", self.asset.clone());
        Ok(p)
    }
}

// -------------------------------------------------------------- transfers ---

/// `POST /aster-chain/v3/perp/wallet/transfer`.
#[derive(Debug, Clone)]
pub struct PerpWalletTransfer {
    /// Asset.
    pub asset: String,
    /// Amount.
    pub amount: Decimal,
    /// Idempotency key.
    pub client_tran_id: String,
    /// Transfer direction.
    pub kind_type: TransferKind,
}

impl Endpoint for PerpWalletTransfer {
    type Response = WalletTransferResult;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/aster-chain/v3/perp/wallet/transfer";
    const VENUE: Venue = Venue::Chain;
    const SECURITY: Security = Security::User;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set_str("asset", self.asset.clone());
        p.set("amount", &self.amount);
        p.set_str("clientTranId", self.client_tran_id.clone());
        p.set("kindType", &self.kind_type);
        Ok(p)
    }
}

/// `POST /aster-chain/v3/spot/wallet/transfer`.
#[derive(Debug, Clone)]
pub struct SpotWalletTransfer {
    /// Asset.
    pub asset: String,
    /// Amount.
    pub amount: Decimal,
    /// Idempotency key.
    pub client_tran_id: String,
    /// Transfer direction.
    pub kind_type: TransferKind,
}

impl Endpoint for SpotWalletTransfer {
    type Response = WalletTransferResult;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/aster-chain/v3/spot/wallet/transfer";
    const VENUE: Venue = Venue::Chain;
    const SECURITY: Security = Security::User;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set_str("asset", self.asset.clone());
        p.set("amount", &self.amount);
        p.set_str("clientTranId", self.client_tran_id.clone());
        p.set("kindType", &self.kind_type);
        Ok(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_paths_and_venues() {
        assert_eq!(
            <AccountStatus as Endpoint>::PATH,
            "/aster-chain/v3/account/status"
        );
        assert_eq!(<PerpWithdraw as Endpoint>::VENUE, Venue::Chain);
        assert_eq!(
            <EstimateWithdrawFee as Endpoint>::PATH.split('/').nth(1),
            Some("aster-chain")
        );
    }

    #[test]
    fn batch_transfer_serializes_items_as_json() {
        let body = BatchTransfer {
            client_tran_id: "vanta-0001".into(),
            items: vec![BatchTransferItem {
                to_address: "0xabc".into(),
                asset: "USDT".into(),
                amount: "10.5".into(),
                transfer_chain_id: None,
            }],
        };
        let p = body.params().unwrap();
        let raw = p.get("items").unwrap();
        assert!(
            raw.starts_with('['),
            "items must be a JSON array, got {raw}"
        );
        assert!(
            raw.contains("\"toAddress\":\"0xabc\""),
            "camelCase keys: {raw}"
        );
        assert!(
            !raw.contains("transferChainId"),
            "None must be skipped: {raw}"
        );
        assert_eq!(p.get("clientTranId"), Some("vanta-0001"));
    }

    #[test]
    fn estimate_fee_is_public() {
        assert_eq!(
            <EstimateWithdrawFee as Endpoint>::SECURITY,
            Security::None,
            "estimateFee needs no credentials"
        );
    }
}
