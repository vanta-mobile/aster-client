//! Nonce generation.
//!
//! Aster V3 requires a **microsecond** timestamp that is strictly increasing per
//! agent address (the server keeps the last 100 nonces and rejects duplicates,
//! stale values, values more than 60s off its clock, or values that fall below
//! the retained minimum). A single process therefore needs a monotonic source;
//! multi-instance deployments must provide a shared one via [`NonceSource`].
//!
//! Two problems are easy to get wrong on a phone:
//!
//! - **Cold start.** [`MicrosecondNonce`] resets to the wall clock, so a device
//!   whose clock went backwards would reuse nonces. [`PersistentNonce`] keeps a
//!   high-water mark in a [`NonceStore`].
//! - **Wrong device clock.** The client rejects nonces more than 60s away from
//!   its own time; call [`Client::calibrate_time`](crate::Client::calibrate_time)
//!   once after startup to apply the offset returned by `/fapi/v3/time`.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Source of request nonces, shared by every client handle.
pub trait NonceSource: Send + Sync + 'static {
    /// Next strictly increasing nonce (microseconds since the epoch).
    fn next(&self) -> u64;

    /// Apply a clock offset in microseconds (device clock minus exchange
    /// clock). Applied to every nonce produced afterwards; the default
    /// implementation ignores it.
    fn set_time_offset(&self, _offset_micros: i64) {}
}

/// Wall-clock microseconds, forced to be strictly increasing.
///
/// Two calls in the same microsecond return `n` and `n + 1`.
#[derive(Debug, Default)]
pub struct MicrosecondNonce {
    last: AtomicU64,
    offset: AtomicI64,
}

impl MicrosecondNonce {
    /// Create a new value.
    pub fn new() -> Self {
        Self::default()
    }

    /// Raise the high-water mark, so later nonces stay above `value`
    /// (used by [`PersistentNonce`] on startup).
    pub fn seed(&self, value: u64) {
        self.last.fetch_max(value, Ordering::Relaxed);
    }
}

impl NonceSource for MicrosecondNonce {
    fn next(&self) -> u64 {
        let now = (now_micros() as i64 + self.offset.load(Ordering::Relaxed)).max(0) as u64;
        let mut last = self.last.load(Ordering::Relaxed);
        loop {
            let candidate = now.max(last + 1);
            match self.last.compare_exchange_weak(
                last,
                candidate,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return candidate,
                Err(current) => last = current,
            }
        }
    }

    fn set_time_offset(&self, offset_micros: i64) {
        self.offset.store(offset_micros, Ordering::Relaxed);
    }
}

impl NonceSource for std::sync::Arc<MicrosecondNonce> {
    fn next(&self) -> u64 {
        (**self).next()
    }

    fn set_time_offset(&self, offset_micros: i64) {
        (**self).set_time_offset(offset_micros);
    }
}

/// Persistent high-water mark behind [`PersistentNonce`].
///
/// Implement this over whatever the platform offers — a file, `NSUserDefaults`,
/// Android `SharedPreferences`, SQLite, a server lease.
pub trait NonceStore: Send + Sync + 'static {
    /// Last nonce that may have been used, if any.
    fn load(&self) -> Option<u64>;

    /// Record the highest nonce handed out so far.
    fn save(&self, high_water: u64);
}

/// A [`NonceStore`] that keeps the high-water mark in one text file.
///
/// Writes go through a temp file and a rename, so a crash mid-write cannot
/// leave a truncated value behind.
#[derive(Debug, Clone)]
pub struct FileNonceStore {
    path: PathBuf,
}

impl FileNonceStore {
    /// Create a new value.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Backing file.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl NonceStore for FileNonceStore {
    fn load(&self) -> Option<u64> {
        std::fs::read_to_string(&self.path)
            .ok()?
            .trim()
            .parse()
            .ok()
    }

    fn save(&self, high_water: u64) {
        let temp = self.path.with_extension("tmp");
        if std::fs::write(&temp, high_water.to_string()).is_ok() {
            let _ = std::fs::rename(&temp, &self.path);
        }
    }
}

/// Microsecond nonces that survive a restart.
///
/// Reads the [store](NonceStore) once, then flushes at most every
/// `flush_every` microseconds of progress (one byte-earlier than 1s by
/// default). Losing that window on a crash is harmless: the wall clock is
/// already past it. Call [`PersistentNonce::flush`] when the app goes to the
/// background.
#[derive(Debug)]
pub struct PersistentNonce<S: NonceStore> {
    inner: MicrosecondNonce,
    store: S,
    persisted: AtomicU64,
    flush_every: u64,
}

impl<S: NonceStore> PersistentNonce<S> {
    /// Create a new value, seeding from the store's high-water mark.
    pub fn new(store: S) -> Self {
        let inner = MicrosecondNonce::new();
        let persisted = store.load().unwrap_or_default();
        if persisted > 0 {
            inner.seed(persisted);
        }
        Self {
            inner,
            store,
            persisted: AtomicU64::new(persisted),
            flush_every: 1_000_000,
        }
    }

    /// Minimum gap between store writes, in microseconds of nonce progress.
    pub fn flush_every(mut self, micros: u64) -> Self {
        self.flush_every = micros;
        self
    }

    /// Persist the current high-water mark now.
    pub fn flush(&self) {
        let last = self.last();
        if last > self.persisted.load(Ordering::Relaxed) {
            self.store.save(last);
            self.persisted.store(last, Ordering::Relaxed);
        }
    }

    fn last(&self) -> u64 {
        self.inner.last.load(Ordering::Relaxed)
    }
}

impl<S: NonceStore> NonceSource for PersistentNonce<S> {
    fn next(&self) -> u64 {
        let nonce = self.inner.next();
        if nonce.saturating_sub(self.persisted.load(Ordering::Relaxed)) >= self.flush_every {
            self.store.save(nonce);
            self.persisted.store(nonce, Ordering::Relaxed);
        }
        nonce
    }

    fn set_time_offset(&self, offset_micros: i64) {
        self.inner.set_time_offset(offset_micros);
    }
}

impl<S: NonceStore> NonceSource for std::sync::Arc<PersistentNonce<S>> {
    fn next(&self) -> u64 {
        (**self).next()
    }

    fn set_time_offset(&self, offset_micros: i64) {
        (**self).set_time_offset(offset_micros);
    }
}

/// Current wall clock in microseconds since the Unix epoch.
pub fn now_micros() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strictly_increasing() {
        let source = MicrosecondNonce::new();
        let mut previous = source.next();
        for _ in 0..10_000 {
            let next = source.next();
            assert!(next > previous, "{next} <= {previous}");
            previous = next;
        }
    }

    #[test]
    fn offset_shifts_nonces_forward() {
        let source = MicrosecondNonce::new();
        let before = now_micros();
        source.set_time_offset(60_000_000);
        assert!(source.next() >= before + 60_000_000);
        source.set_time_offset(-60_000_000);
        assert!(source.next() >= before - 60_000_000);
    }
}
