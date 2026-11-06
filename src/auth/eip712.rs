//! EIP-712 typed-data hashing as used by Aster V3.
//!
//! Two domains exist:
//! * `AsterSignTransaction` / version `1` — every trading and account request,
//!   with `chainId` 1666 (mainnet) or 714 (testnet).
//! * `Aster` / version `1` — withdrawals, with `chainId` equal to the
//!   *destination* chain (1 ETH, 56 BSC, 42161 Arbitrum).
//!
//! Only flat structs of `string` / `address` / `uint256` / `bool` fields are
//! needed by the API. Dynamic `string` values are hashed into 32 bytes, so the
//! struct hash is a plain 32-byte concatenation — no head/tail offsets.

use alloy_primitives::{Address, B256};
use sha3::{Digest as _, Keccak256};

/// `keccak256` of arbitrary bytes.
pub fn keccak(bytes: &[u8]) -> B256 {
    B256::from_slice(&Keccak256::digest(bytes))
}

/// EIP-712 domain separator inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Domain {
    /// Domain name.
    pub name: &'static str,
    /// Domain version.
    pub version: &'static str,
    /// Chain id of the settlement network.
    pub chain_id: u64,
    /// Verifying contract (zero address for Aster).
    pub verifying_contract: Address,
}

impl Domain {
    /// Domain for signed requests (`AsterSignTransaction`, zero contract).
    pub fn aster_sign(chain_id: u64) -> Self {
        Self {
            name: "AsterSignTransaction",
            version: "1",
            chain_id,
            verifying_contract: Address::ZERO,
        }
    }

    /// Domain for withdrawals (`Aster`, zero contract, destination chain id).
    pub fn withdraw(chain_id: u64) -> Self {
        Self {
            name: "Aster",
            version: "1",
            chain_id,
            verifying_contract: Address::ZERO,
        }
    }

    /// EIP-712 domain separator.
    pub fn separator(&self) -> B256 {
        const DOMAIN_TYPE: &str =
            "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)";
        let mut buf = Vec::with_capacity(160);
        buf.extend_from_slice(keccak(DOMAIN_TYPE.as_bytes()).as_slice());
        buf.extend_from_slice(keccak(self.name.as_bytes()).as_slice());
        buf.extend_from_slice(keccak(self.version.as_bytes()).as_slice());
        buf.extend_from_slice(&uint256_bytes(self.chain_id as u128));
        buf.extend_from_slice(&address_bytes(self.verifying_contract));
        keccak(&buf)
    }
}

/// A single member of a typed struct.
#[derive(Debug, Clone, Copy)]
pub enum FieldValue<'a> {
    /// Dynamic `string`.
    String(&'a str),
    /// `address`.
    Address(Address),
    /// `uint256`.
    Uint(u64),
    /// `bool`.
    Bool(bool),
}

impl FieldValue<'_> {
    /// ABI type name of this member.
    pub fn type_name(&self) -> &'static str {
        match self {
            FieldValue::String(_) => "string",
            FieldValue::Address(_) => "address",
            FieldValue::Uint(_) => "uint256",
            FieldValue::Bool(_) => "bool",
        }
    }

    fn encode(&self) -> [u8; 32] {
        match self {
            FieldValue::String(s) => keccak(s.as_bytes()).0,
            FieldValue::Address(a) => address_bytes(*a),
            FieldValue::Uint(n) => uint256_bytes(*n as u128),
            FieldValue::Bool(b) => uint256_bytes(*b as u128),
        }
    }
}

/// A flat EIP-712 struct with its members in declaration order.
#[derive(Debug, Clone)]
pub struct Struct<'a> {
    /// Primary type name.
    pub primary_type: &'static str,
    /// Members in declaration order.
    pub fields: Vec<(&'static str, FieldValue<'a>)>,
}

impl<'a> Struct<'a> {
    /// Create a new value.
    pub fn new(primary_type: &'static str) -> Self {
        Self {
            primary_type,
            fields: Vec::new(),
        }
    }

    /// Append a struct member.
    pub fn field(mut self, name: &'static str, value: FieldValue<'a>) -> Self {
        self.fields.push((name, value));
        self
    }

    /// `keccak256("Primary(type name,...)")`.
    pub fn type_hash(&self) -> B256 {
        let mut encoded = String::with_capacity(64);
        encoded.push_str(self.primary_type);
        encoded.push('(');
        for (i, (name, value)) in self.fields.iter().enumerate() {
            if i > 0 {
                encoded.push(',');
            }
            encoded.push_str(value.type_name());
            encoded.push(' ');
            encoded.push_str(name);
        }
        encoded.push(')');
        keccak(encoded.as_bytes())
    }

    /// EIP-712 struct hash.
    pub fn struct_hash(&self) -> B256 {
        let mut buf = Vec::with_capacity(32 * (self.fields.len() + 1));
        buf.extend_from_slice(self.type_hash().as_slice());
        for (_, value) in &self.fields {
            buf.extend_from_slice(&value.encode());
        }
        keccak(&buf)
    }

    /// `keccak256("\x19\x01" || domainSeparator || structHash)`.
    pub fn digest(&self, domain: &Domain) -> B256 {
        let mut buf = [0u8; 66];
        buf[0] = 0x19;
        buf[1] = 0x01;
        buf[2..34].copy_from_slice(domain.separator().as_slice());
        buf[34..].copy_from_slice(self.struct_hash().as_slice());
        keccak(&buf)
    }
}

/// `Message(string msg)` — the payload signed for ordinary REST requests.
pub fn message_struct(msg: &str) -> Struct<'_> {
    Struct::new("Message").field("msg", FieldValue::String(msg))
}

/// Digest for a plain request payload.
pub fn message_digest(domain: &Domain, payload: &str) -> B256 {
    message_struct(payload).digest(domain)
}

fn address_bytes(address: Address) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[12..].copy_from_slice(address.as_slice());
    out
}

fn uint256_bytes(value: u128) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[16..].copy_from_slice(&value.to_be_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_hash_lists_fields_in_declaration_order() {
        let s = Struct::new("ApproveBuilder")
            .field("Builder", FieldValue::String("0xabc"))
            .field("MaxFeeRate", FieldValue::String("0.001"))
            .field("BuilderName", FieldValue::String("acme"))
            .field("AsterChain", FieldValue::String("Mainnet"))
            .field(
                "User",
                FieldValue::String("0x0000000000000000000000000000000000000000"),
            )
            .field("Nonce", FieldValue::Uint(1));
        assert_eq!(
            s.type_hash(),
            keccak(
                b"ApproveBuilder(string Builder,string MaxFeeRate,string BuilderName,string AsterChain,string User,uint256 Nonce)"
            )
        );
    }

    #[test]
    fn type_hash_keeps_embedded_spaces() {
        let s = Struct::new("Action")
            .field("type", FieldValue::String("Withdraw"))
            .field("destination Chain", FieldValue::String("BSC"));
        assert_eq!(
            s.type_hash(),
            keccak(b"Action(string type,string destination Chain)")
        );
    }
}
