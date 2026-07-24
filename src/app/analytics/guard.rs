//! Small in-process abuse guard for the public write endpoints.
//!
//! The database remains authoritative and event IDs remain idempotent. This
//! guard merely bounds accidental loops and low-effort write floods before
//! they consume a connection. Keys are one-way hashes and never leave memory.

use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use sha2::{Digest, Sha256};
use topcoat::router::HeaderMap;

#[derive(Clone, Default)]
pub struct AnalyticsGuard {
    state: Arc<Mutex<State>>,
}

#[derive(Default)]
struct State {
    buckets: HashMap<String, VecDeque<Instant>>,
    requests: u64,
}

#[derive(Clone, Copy)]
pub enum WriteKind {
    Event,
    Identity,
}

impl AnalyticsGuard {
    pub fn allow(&self, headers: &HeaderMap, fallback_visitor: &str, kind: WriteKind) -> bool {
        let (limit, window, namespace) = match kind {
            WriteKind::Event => (90, Duration::from_secs(60), "event"),
            WriteKind::Identity => (4, Duration::from_secs(60 * 60), "identity"),
        };
        let source = headers
            .get("cf-connecting-ip")
            .and_then(|value| value.to_str().ok())
            .filter(|value| !value.is_empty() && value.len() <= 64)
            .unwrap_or(fallback_visitor);
        let key = digest_key(namespace, source);
        let now = Instant::now();

        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        state.requests += 1;
        if state.requests.is_multiple_of(256) {
            state.buckets.retain(|_, entries| {
                entries
                    .back()
                    .is_some_and(|seen| now.duration_since(*seen) <= Duration::from_secs(60 * 60))
            });
        }

        let entries = state.buckets.entry(key).or_default();
        while entries
            .front()
            .is_some_and(|seen| now.duration_since(*seen) > window)
        {
            entries.pop_front();
        }
        if entries.len() >= limit {
            return false;
        }
        entries.push_back(now);
        true
    }
}

fn digest_key(namespace: &str, source: &str) -> String {
    let mut hash = Sha256::new();
    hash.update(namespace);
    hash.update([0]);
    hash.update(source);
    hash.finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_writes_are_bounded_without_storing_the_source() {
        let guard = AnalyticsGuard::default();
        let headers = HeaderMap::new();
        for _ in 0..4 {
            assert!(guard.allow(&headers, "visitor-a", WriteKind::Identity));
        }
        assert!(!guard.allow(&headers, "visitor-a", WriteKind::Identity));
        assert!(guard.allow(&headers, "visitor-b", WriteKind::Identity));
    }

    #[test]
    fn keys_are_scoped_by_action() {
        assert_ne!(
            digest_key("event", "203.0.113.7"),
            digest_key("identity", "203.0.113.7")
        );
    }
}
