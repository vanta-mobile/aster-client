//! Funding: withdrawals (`userSignature`), fees, history and the EVM deposit
//! contract calldata.
//!
//! These settlements do **not** use the `AsterSignTransaction` domain: the
//! withdrawal signature lives in the `Aster` domain whose `chainId` is the
//! destination chain (1 / 56 / 42161). See [`crate::auth::Domain::withdraw`].

use alloy_primitives::Address;

use crate::auth::{self, AsterChain, Domain, Signature, Signer, WithdrawChain};
use crate::client::{Endpoint, Method, Security, Venue};
use crate::error::Result;
use crate::model::*;
use crate::params::Params;

/// `GET /api/v3/aster/withdraw/estimateFee`.
#[derive(Debug, Clone)]
pub struct EstimateWithdrawFee {
    /// Settlement chain.
    pub chain: WithdrawChain,
    /// Asset.
    pub asset: String,
}

impl Endpoint for EstimateWithdrawFee {
    type Response = WithdrawEstimateFee;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/api/v3/aster/withdraw/estimateFee";
    const VENUE: Venue = Venue::Spot;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        p.set("chainId", &self.chain.chain_id());
        p.set("asset", &self.asset);
        Ok(p)
    }
}

/// What the user wants to withdraw. Sign it with the **account wallet** key.
#[derive(Debug, Clone)]
pub struct WithdrawIntent {
    /// Settlement chain.
    pub chain: WithdrawChain,
    /// Asset.
    pub asset: String,
    /// Amount.
    pub amount: rust_decimal::Decimal,
    /// Fee amount.
    pub fee: rust_decimal::Decimal,
    /// Withdrawal destination address.
    pub receiver: Address,
}

/// A withdrawal with its EIP-712 signature, ready to submit.
#[derive(Debug, Clone)]
pub struct SignedWithdraw {
    /// Withdrawal intent being signed.
    pub intent: WithdrawIntent,
    /// Request nonce: microsecond timestamp, strictly increasing per signer.
    pub nonce: u64,
    /// EIP-712 signature.
    pub signature: Signature,
}

impl WithdrawIntent {
    /// Freeze the intent with a microsecond nonce and sign the `Action` struct.
    ///
    /// `fee` must be the value returned by [`EstimateWithdrawFee`] in the same
    /// intent — the server rejects stale fees.
    pub fn sign(&self, signer: &dyn Signer, nonce: Option<u64>) -> Result<SignedWithdraw> {
        let nonce = nonce.unwrap_or_else(auth::now_micros);
        let domain = Domain::withdraw(self.chain.chain_id());
        let amount = self.amount.to_string();
        let fee = self.fee.to_string();
        let data = auth::withdraw_action(
            self.receiver,
            self.chain,
            &self.asset,
            &amount,
            &fee,
            nonce,
            AsterChain::Mainnet,
        );
        let signature = auth::sign_struct(signer, &domain, &data)?;
        Ok(SignedWithdraw {
            intent: self.clone(),
            nonce,
            signature,
        })
    }
}

impl SignedWithdraw {
    pub(crate) fn write(&self, p: &mut Params) {
        p.set("chainId", &self.intent.chain.chain_id());
        p.set("asset", &self.intent.asset);
        p.set("amount", &self.intent.amount);
        p.set("fee", &self.intent.fee);
        p.set("receiver", &self.intent.receiver);
        p.set("userNonce", &self.nonce);
        p.set_str("userSignature", self.signature.to_hex());
    }
}

/// `POST /fapi/v3/aster/user-withdraw` (futures balance).
#[derive(Debug, Clone)]
pub struct FuturesWithdraw(pub SignedWithdraw);

impl Endpoint for FuturesWithdraw {
    type Response = WithdrawResult;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/fapi/v3/aster/user-withdraw";
    const VENUE: Venue = Venue::Futures;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        self.0.write(&mut p);
        Ok(p)
    }
}

/// `POST /api/v3/aster/user-withdraw` (spot balance).
#[derive(Debug, Clone)]
pub struct SpotWithdraw(pub SignedWithdraw);

impl Endpoint for SpotWithdraw {
    type Response = WithdrawResult;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/api/v3/aster/user-withdraw";
    const VENUE: Venue = Venue::Spot;

    fn params(&self) -> Result<Params> {
        let mut p = Params::new();
        self.0.write(&mut p);
        Ok(p)
    }
}

/// `POST /fapi/v3/aster/deposit-withdraw-history`.
///
/// The provided docs list the response shape but no filter parameters; this
/// endpoint therefore sends none.
#[derive(Debug, Clone, Copy, Default)]
pub struct DepositWithdrawHistory;

impl Endpoint for DepositWithdrawHistory {
    type Response = Vec<DepositWithdrawRecord>;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/fapi/v3/aster/deposit-withdraw-history";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;
}

/// `POST /fapi/v3/aster/user-withdraw-info`.
#[derive(Debug, Clone, Copy, Default)]
pub struct WithdrawInfo;

impl Endpoint for WithdrawInfo {
    type Response = Vec<DepositWithdrawRecord>;
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/fapi/v3/aster/user-withdraw-info";
    const VENUE: Venue = Venue::Futures;
    const SECURITY: Security = Security::Signer;
}

/// EVM deposit vaults and calldata for `depositFor(...)`.
///
/// A deposit is: `approve(vault, amount)` for ERC-20s, then
/// `depositFor(currency, user, amount, broker)` with `msg.value = 0`
/// (native coin: `currency` is the placeholder and `msg.value == amount`).
pub mod deposit {
    use super::*;

    /// Placeholder `currency` address for native ETH/BNB.
    /// Native ETH/BNB placeholder used as `currency`.
    pub const NATIVE_CURRENCY: Address = Address::new(hex_literal_address(
        "fdAE1bA7C826aBDc4c99903c8056f82a1A04a615",
    ));

    /// `broker = 1000` credits the spot account; any other value credits futures.
    pub const BROKER_SPOT: u64 = 1000;

    /// Chain the deposit is credited on.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum DepositChain {
        /// Ethereum.
        Ethereum,
        /// BNB Smart Chain.
        Bsc,
        /// Arbitrum One.
        Arbitrum,
    }

    impl DepositChain {
        /// chain id of the settlement network.
        pub fn chain_id(self) -> u64 {
            match self {
                DepositChain::Ethereum => 1,
                DepositChain::Bsc => 56,
                DepositChain::Arbitrum => 42_161,
            }
        }

        /// Vault address on this chain.
        pub fn vault(self) -> Address {
            match self {
                DepositChain::Ethereum => Address::new(hex_literal_address(
                    "604DD02d620633Ae427888d41bfd15e38483736E",
                )),
                DepositChain::Bsc => Address::new(hex_literal_address(
                    "128463A60784c4D3f46c23Af3f65Ed859Ba87974",
                )),
                DepositChain::Arbitrum => Address::new(hex_literal_address(
                    "9E36CB86a159d479cEd94Fa05036f235Ac40E1d5",
                )),
            }
        }
    }

    /// `depositFor(address currency, address forAddress, uint256 amount, uint256 broker)`.
    pub fn deposit_for_calldata(
        currency: Address,
        for_address: Address,
        amount: u128,
        broker: u64,
    ) -> Vec<u8> {
        let mut data = selector(b"depositFor(address,address,uint256,uint256)").to_vec();
        data.extend_from_slice(&address_word(currency));
        data.extend_from_slice(&address_word(for_address));
        data.extend_from_slice(&uint256(amount));
        data.extend_from_slice(&uint256(broker as u128));
        data
    }

    /// `approve(address spender, uint256 amount)`.
    pub fn approve_calldata(spender: Address, amount: u128) -> Vec<u8> {
        let mut data = selector(b"approve(address,uint256)").to_vec();
        data.extend_from_slice(&address_word(spender));
        data.extend_from_slice(&uint256(amount));
        data
    }

    fn uint256(value: u128) -> [u8; 32] {
        let mut out = [0u8; 32];
        out[16..].copy_from_slice(&value.to_be_bytes());
        out
    }

    fn selector(signature: &[u8]) -> [u8; 4] {
        let hash = auth::keccak(signature);
        [hash[0], hash[1], hash[2], hash[3]]
    }

    fn address_word(address: Address) -> [u8; 32] {
        let mut out = [0u8; 32];
        out[12..].copy_from_slice(address.as_slice());
        out
    }
}

const fn hex_literal_address(hex: &str) -> [u8; 20] {
    const fn nibble(c: u8) -> u8 {
        match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'f' => c - b'a' + 10,
            b'A'..=b'F' => c - b'A' + 10,
            _ => 0,
        }
    }
    let bytes = hex.as_bytes();
    let mut out = [0u8; 20];
    let mut i = 0;
    while i < 20 {
        out[i] = (nibble(bytes[i * 2]) << 4) | nibble(bytes[i * 2 + 1]);
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deposit_calldata_matches_abi_shape() {
        let currency = deposit::NATIVE_CURRENCY;
        let data = deposit::deposit_for_calldata(
            currency,
            Address::ZERO,
            1_000_000_u128,
            deposit::BROKER_SPOT,
        );
        assert_eq!(data.len(), 4 + 4 * 32);
        assert_eq!(
            &data[..4],
            &auth::keccak(b"depositFor(address,address,uint256,uint256)").0[..4]
        );
        assert_eq!(&data[4..36][12..], currency.as_slice());
        assert_eq!(&data[92..100], &1_000_000u64.to_be_bytes());
        assert_eq!(&data[124..132], &1000u64.to_be_bytes());
    }
}
