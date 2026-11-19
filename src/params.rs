//! Request parameter collection, canonical form encoding and signing payload.
//!
//! The one hard rule of Aster V3 signing: **the signed string must be byte
//! identical to the string that is sent**. Every request is therefore built as
//! a [`Params`](crate::params::Params) map, encoded once with
//! [`Params::encode`](crate::params::Params::encode) (keys sorted by ASCII,
//! `application/x-www-form-urlencoded`), signed, and then sent with
//! `&signature=...` appended — the encoded prefix is never reordered or
//! re-encoded afterwards.

/// Something that can be written as a form parameter.
///
/// `Option<T>` is skipped entirely when `None`, which is how optional request
/// parameters are expressed. Booleans are sent lowercase, matching the wire
/// examples (`reduceOnly=true`).
pub trait Param {
    /// Write this value under `key`; `None` writes nothing.
    fn write(&self, key: &'static str, out: &mut Params);
}

impl Param for str {
    fn write(&self, key: &'static str, out: &mut Params) {
        out.0.push((key, self.to_owned()));
    }
}

impl Param for String {
    fn write(&self, key: &'static str, out: &mut Params) {
        out.0.push((key, self.clone()));
    }
}

impl<T: Param + ?Sized> Param for &T {
    fn write(&self, key: &'static str, out: &mut Params) {
        (*self).write(key, out);
    }
}

impl<T: Param> Param for Option<T> {
    fn write(&self, key: &'static str, out: &mut Params) {
        if let Some(value) = self {
            value.write(key, out);
        }
    }
}

impl Param for alloy_primitives::Address {
    fn write(&self, key: &'static str, out: &mut Params) {
        out.0.push((key, format!("{self:#x}")));
    }
}

impl Param for bool {
    fn write(&self, key: &'static str, out: &mut Params) {
        out.0
            .push((key, if *self { "true" } else { "false" }.to_owned()));
    }
}

impl Param for rust_decimal::Decimal {
    fn write(&self, key: &'static str, out: &mut Params) {
        // Decimal's Display never emits an exponent, so prices keep the exact
        // digits the caller supplied.
        out.0.push((key, self.to_string()));
    }
}

macro_rules! int_params {
    ($($t:ty),*) => {$(
        impl Param for $t {
            fn write(&self, key: &'static str, out: &mut Params) {
                out.0.push((key, self.to_string()));
            }
        }
    )*};
}
int_params!(i8, i16, i32, i64, u8, u16, u32, u64, usize);

/// Ordered request parameters. Duplicate keys are allowed (the exchange takes
/// the last one); ordering is normalised to ASCII ascending by [`Params::encode`],
/// unless built with [`Params::in_order`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Params(Vec<(&'static str, String)>, bool);

impl Params {
    /// Create a new value.
    pub fn new() -> Self {
        Self::default()
    }

    /// Parameters that keep their insertion order when encoded.
    ///
    /// For endpoints whose signature covers a documented field order rather
    /// than the ASCII-sorted canonical string (`registerAndApproveAgent`):
    /// the encoded string is both the signed `msg` and the request body.
    pub fn in_order() -> Self {
        Self(Vec::new(), true)
    }

    /// Set a typed parameter. `None` values are dropped.
    pub fn set<P: Param + ?Sized>(&mut self, key: &'static str, value: &P) {
        value.write(key, self);
    }

    /// Set a raw string parameter.
    pub fn set_str(&mut self, key: &'static str, value: impl Into<String>) {
        self.0.push((key, value.into()));
    }

    /// Number of parameters.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether nothing was collected.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Value of a parameter, if present.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0
            .iter()
            .rev()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v.as_str())
    }

    /// Canonical `(key, value)` pairs, in the same order as [`Params::encode`].
    ///
    /// Useful for debugging and for tests that want to assert the wire
    /// parameters without dealing with percent-encoding.
    pub fn pairs(&self) -> Vec<(&'static str, String)> {
        let mut entries: Vec<_> = self.0.clone();
        if !self.1 {
            entries.sort_by(|a, b| a.0.cmp(b.0));
        }
        entries
    }

    /// Consume the parameters as pairs (used to build JSON list parameters).
    pub fn into_pairs(self) -> Vec<(&'static str, String)> {
        self.0
    }

    /// Canonical string: keys sorted by ASCII (insertion order for
    /// [`Params::in_order`]), percent-encoded (`A-Za-z0-9-._~` literal, space
    /// as `+`, everything else `%XX` uppercase).
    ///
    /// This is exactly the string that must be signed.
    pub fn encode(&self) -> String {
        let mut entries: Vec<&(&'static str, String)> = self.0.iter().collect();
        if !self.1 {
            entries.sort_by(|a, b| a.0.cmp(b.0));
        }
        let mut out = String::with_capacity(64 * entries.len());
        for (key, value) in entries {
            if !out.is_empty() {
                out.push('&');
            }
            encode_component(key, &mut out);
            out.push('=');
            encode_component(value, &mut out);
        }
        out
    }

    /// The canonical string with the signature appended: this is the exact
    /// request body/query that is sent.
    pub fn encode_signed(&self, signature: &str) -> String {
        let mut s = self.encode();
        if !s.is_empty() {
            s.push('&');
        }
        s.push_str("signature=");
        s.push_str(signature);
        s
    }
}

fn encode_component(input: &str, out: &mut String) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(byte as char)
            }
            b' ' => out.push('+'),
            other => {
                out.push('%');
                out.push(HEX[(other >> 4) as usize] as char);
                out.push(HEX[(other & 0xf) as usize] as char);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorts_ascii_and_percent_encodes() {
        let mut p = Params::new();
        p.set("symbol", &"AAPLUSDT");
        p.set("side", &"BUY");
        p.set("quoteOrderQty", &rust_decimal::Decimal::new(10025, 2));
        p.set("note", &"a b&c~");
        p.set::<Option<&str>>("reduceOnly", &None);

        assert_eq!(
            p.encode(),
            "note=a+b%26c~&quoteOrderQty=100.25&side=BUY&symbol=AAPLUSDT"
        );
        assert_eq!(
            p.encode_signed("0xdead"),
            "note=a+b%26c~&quoteOrderQty=100.25&side=BUY&symbol=AAPLUSDT&signature=0xdead"
        );
    }

    #[test]
    fn encodes_non_ascii_and_reserved_bytes() {
        let mut p = Params::new();
        p.set("ipWhitelist", &"192.168.1.1 10.0.0.0/24");
        assert_eq!(p.encode(), "ipWhitelist=192.168.1.1+10.0.0.0%2F24");

        let mut p = Params::new();
        p.set("msg", &"é/:");
        assert_eq!(p.encode(), "msg=%C3%A9%2F%3A");
    }
}
