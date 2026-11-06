//! Signing keys and signatures.

use alloy_primitives::{Address, B256};
use k256::ecdsa::{RecoveryId, Signature as K256Signature, SigningKey, VerifyingKey};

use crate::auth::eip712;
use crate::error::{Error, Result};

/// A 65-byte `r || s || v` signature with `v ∈ {27, 28}` (the encoding Aster
/// expects, matching `eth_account.signature.hex()`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Signature([u8; 65]);

impl Signature {
    /// Parse from raw bytes.
    pub fn from_bytes(bytes: [u8; 65]) -> Result<Self> {
        match bytes[64] {
            27 | 28 => Ok(Self(bytes)),
            v => Err(Error::Signing(format!(
                "invalid recovery byte {v}, expected 27 or 28"
            ))),
        }
    }

    /// Parse `0x`-prefixed or bare hex.
    pub fn from_hex(hex_str: &str) -> Result<Self> {
        let raw = hex::decode(hex_str.trim_start_matches("0x"))
            .map_err(|e| Error::Signing(format!("invalid signature hex: {e}")))?;
        let bytes: [u8; 65] = raw.try_into().map_err(|v: Vec<u8>| {
            Error::Signing(format!("signature must be 65 bytes, got {}", v.len()))
        })?;
        Self::from_bytes(bytes)
    }

    /// Raw 65-byte signature (`r || s || v`).
    pub fn as_bytes(&self) -> &[u8; 65] {
        &self.0
    }

    /// `0x`-prefixed lowercase hex.
    pub fn to_hex(&self) -> String {
        format!("0x{}", hex::encode(self.0))
    }
}

impl std::fmt::Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_hex())
    }
}

/// Anything able to produce EIP-712 signatures for an Aster account.
///
/// Implement this for KMS/HSM-backed keys: only the 32-byte digest leaves the
/// process. [`EvmSigner`] is the in-memory reference implementation.
pub trait Signer: Send + Sync {
    /// Address whose private key signs (an agent signer or the master wallet).
    fn address(&self) -> Address;

    /// Sign a prehashed EIP-712 digest.
    fn sign_digest(&self, digest: B256) -> Result<Signature>;
}

/// Client state without a key: read-only endpoints work, signed ones return
/// [`Error::MissingCredentials`].
#[derive(Debug, Clone, Copy, Default)]
pub struct NoSigner;

impl Signer for NoSigner {
    fn address(&self) -> Address {
        Address::ZERO
    }

    fn sign_digest(&self, _digest: B256) -> Result<Signature> {
        Err(Error::MissingCredentials)
    }
}

/// In-memory secp256k1 signer.
#[derive(Clone)]
pub struct EvmSigner {
    key: SigningKey,
    address: Address,
}

impl EvmSigner {
    /// `0x`-prefixed or bare 32-byte private key.
    pub fn from_hex(private_key: &str) -> Result<Self> {
        let raw = hex::decode(private_key.trim_start_matches("0x"))
            .map_err(|e| Error::Signing(format!("invalid private key hex: {e}")))?;
        Self::from_bytes(&raw)
    }

    /// Parse from raw bytes.
    pub fn from_bytes(private_key: &[u8]) -> Result<Self> {
        let key = SigningKey::from_slice(private_key)
            .map_err(|e| Error::Signing(format!("invalid secp256k1 private key: {e}")))?;
        let address = address_of(key.verifying_key());
        Ok(Self { key, address })
    }

    /// Address of the signing key.
    /// Address derived from the key.
    pub fn address(&self) -> Address {
        self.address
    }

    /// Alias of [`EvmSigner::address`], kept for callers that avoid the trait.
    pub fn public_address(&self) -> Address {
        self.address
    }
}

impl Signer for EvmSigner {
    fn address(&self) -> Address {
        self.address
    }

    fn sign_digest(&self, digest: B256) -> Result<Signature> {
        let (signature, recovery_id) = self
            .key
            .sign_prehash_recoverable(digest.as_slice())
            .map_err(|e| Error::Signing(format!("secp256k1 sign failed: {e}")))?;
        let mut out = [0u8; 65];
        out[..64].copy_from_slice(&signature.to_bytes());
        out[64] = recovery_id.to_byte() + 27;
        Signature::from_bytes(out)
    }
}

impl std::fmt::Debug for EvmSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EvmSigner")
            .field("address", &self.address)
            .finish_non_exhaustive()
    }
}

/// Convenience: sign a `Message(string msg)` payload in the given domain.
pub fn sign_message(
    signer: &dyn Signer,
    domain: &eip712::Domain,
    payload: &str,
) -> Result<Signature> {
    signer.sign_digest(eip712::message_digest(domain, payload))
}

fn address_of(key: &VerifyingKey) -> Address {
    let point = key.to_encoded_point(false);
    let hash = eip712::keccak(&point.as_bytes()[1..]);
    Address::from_slice(&hash[12..])
}

// Keep the k256 signature type name out of the public surface.
const _: fn(K256Signature, RecoveryId) = |_, _| {};

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &str = "0x0123456789012345678901234567890123456789012345678901234567890123";

    #[test]
    fn derives_documented_address() {
        let signer = EvmSigner::from_hex(KEY).unwrap();
        assert_eq!(
            signer.address().to_string().to_lowercase(),
            "0x14791697260e4c9a71f18484c9f997b308e59325"
        );
    }

    #[test]
    fn rejects_out_of_range_recovery_byte() {
        let mut bytes = [0u8; 65];
        bytes[64] = 0;
        assert!(Signature::from_bytes(bytes).is_err());
    }
}
