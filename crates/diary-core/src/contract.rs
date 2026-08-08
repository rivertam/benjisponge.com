//! The wire half of the queue: what an entry looks like on
//! `POST /api/diary/entries` and what each response means to the flusher.
//! The server parses with these exact envelopes; the worker serializes and
//! classifies with them. Domain acceptance lives in [`crate::entry`].

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::entry::{ComposedEntry, DiaryEntry, SavedRef};

/// The queue-replay endpoint. The worker flushes here and nowhere else.
pub const API_PATH: &str = "/api/diary/entries";

/// The current diary wire generation. Readers accept the frozen body-only
/// predecessors during deploy skew, reject an unknown generation distinctly,
/// and strictly parse the full selected envelope before use.
pub const CURRENT_WIRE_VERSION: u16 = 2;

/// The body-only generation shipped immediately before reply relationships.
/// Its private DTOs below stay frozen: decoding an old envelope straight
/// into today's optional-field domain type would accidentally accept a new
/// field under an old version number.
const BODY_ONLY_WIRE_VERSION: u16 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ComposedEntryV1 {
    written_at: i64,
    body: String,
}

impl From<ComposedEntryV1> for ComposedEntry {
    fn from(entry: ComposedEntryV1) -> Self {
        Self::new(entry.written_at, entry.body)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DiaryEntryV1 {
    id: String,
    written_at: i64,
    body: String,
}

impl From<DiaryEntryV1> for DiaryEntry {
    fn from(entry: DiaryEntryV1) -> Self {
        Self::from_parts(entry.id, entry.written_at, entry.body)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PushWireV1 {
    version: u16,
    entry: ComposedEntryV1,
}

/// Versioned push envelope emitted by current workers.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PushWire {
    pub version: u16,
    pub entry: ComposedEntry,
}

impl PushWire {
    pub fn new(entry: ComposedEntry) -> Self {
        Self {
            version: CURRENT_WIRE_VERSION,
            entry,
        }
    }
}

/// One serialized argument across the JavaScript→wasm enqueue seam. Adding
/// entry behavior changes [`EntryContent`](crate::entry::EntryContent), not
/// the exported wasm function's positional signature.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComposeCommand {
    pub version: u16,
    pub entry: ComposedEntry,
    pub enqueued_at_ms: i64,
}

impl ComposeCommand {
    pub fn new(entry: ComposedEntry, enqueued_at_ms: i64) -> Self {
        Self {
            version: CURRENT_WIRE_VERSION,
            entry,
            enqueued_at_ms,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WireError {
    Malformed,
    UnsupportedVersion(u16),
}

impl std::fmt::Display for WireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Malformed => write!(f, "malformed diary wire value"),
            Self::UnsupportedVersion(version) => {
                write!(f, "unsupported diary wire version {version}")
            }
        }
    }
}

impl std::error::Error for WireError {}

#[derive(Deserialize)]
struct VersionProbe {
    #[serde(default)]
    version: Option<u16>,
}

fn probe_version(body: &[u8]) -> Result<Option<u16>, WireError> {
    serde_json::from_slice::<VersionProbe>(body)
        .map(|probe| probe.version)
        .map_err(|_| WireError::Malformed)
}

/// Deserialize one selected wire generation exactly. Serde cannot combine
/// `deny_unknown_fields` with our nested flattened lifecycle wrappers: each
/// wrapper mistakes its parent's fields for unknowns. Comparing the parsed
/// JSON value with that generation's value serialized back out gives the
/// wire boundary the same guarantee recursively—nothing may be consumed and
/// discarded.
fn decode_exact<T>(body: &[u8]) -> Result<T, WireError>
where
    T: DeserializeOwned + Serialize,
{
    let source: serde_json::Value =
        serde_json::from_slice(body).map_err(|_| WireError::Malformed)?;
    let parsed: T = serde_json::from_slice(body).map_err(|_| WireError::Malformed)?;
    let canonical = serde_json::to_value(&parsed).map_err(|_| WireError::Malformed)?;
    if source == canonical {
        Ok(parsed)
    } else {
        Err(WireError::Malformed)
    }
}

/// Decode a push after probing its generation. The unversioned flat shape
/// is the base PR's predecessor and remains readable so an old worker can
/// finish its outbox after a server deploy.
pub fn decode_push(body: &[u8]) -> Result<ComposedEntry, WireError> {
    match probe_version(body)? {
        None => decode_exact::<ComposedEntryV1>(body).map(Into::into),
        Some(BODY_ONLY_WIRE_VERSION) => {
            let wire: PushWireV1 = decode_exact(body)?;
            Ok(wire.entry.into())
        }
        Some(CURRENT_WIRE_VERSION) => {
            let wire: PushWire = decode_exact(body)?;
            Ok(wire.entry)
        }
        Some(version) => Err(WireError::UnsupportedVersion(version)),
    }
}

/// Decode the page's single serialized enqueue command. Only the current
/// page/wasm pair may compose new local rows; predecessor commands are not a
/// durable replay wire and must never carry current semantics under an old
/// generation number.
pub fn decode_compose_command(body: &str) -> Result<ComposeCommand, WireError> {
    let bytes = body.as_bytes();
    let version = probe_version(bytes)?.ok_or(WireError::Malformed)?;
    if version != CURRENT_WIRE_VERSION {
        return Err(WireError::UnsupportedVersion(version));
    }
    decode_exact(bytes)
}

/// What one replay attempt means for the queue.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SendOutcome {
    /// The server holds the entry — a fresh insert or a deduped replay —
    /// under the identity it reported back.
    Saved(SavedRef),
    /// 401/404: signed out or the wrong account. Retrying cannot fix it
    /// (and burns Background Sync's bounded attempt budget), so the flush
    /// stops quietly and the page shows the sign-in banner.
    Auth,
    /// Network failure, 403, 5xx, or a 200 that is not our JSON (a captive
    /// portal) — never the entry's fault; stop and retry later.
    Retry,
    /// 409/413/415/422: the entry itself can never succeed. It is
    /// marked failed and kept on the page for manual copy — queued diary
    /// text is never silently dropped.
    Rejected(u16),
}

/// Classify one response from the replay endpoint. `body` is only consulted
/// for 2xx, where anything but our full saved JSON — status, id, and
/// written_at, the shape `api_write_entry` always returns — means something
/// other than the site answered.
pub fn classify_response(status: u16, body: &str) -> SendOutcome {
    #[derive(Deserialize)]
    struct SavedResponse {
        status: String,
        id: String,
        written_at: i64,
    }

    match status {
        200..=299 => match serde_json::from_str::<SavedResponse>(body) {
            Ok(response) if response.status == "saved" => SendOutcome::Saved(SavedRef {
                id: response.id,
                written_at: response.written_at,
            }),
            _ => SendOutcome::Retry,
        },
        401 | 404 => SendOutcome::Auth,
        // A generated request cannot be malformed. During a rolling deploy,
        // however, an older server answers a newer strict envelope with 400.
        // Preserve the queued text for retry instead of making skew fatal.
        400 => SendOutcome::Retry,
        409 | 413 | 415 | 422 => SendOutcome::Rejected(status),
        _ => SendOutcome::Retry,
    }
}

/// The reason a permanent rejection leaves on the queued entry — the exact
/// text the page has always rendered next to failed entries.
pub fn rejection_reason(status: u16) -> String {
    format!("rejected (HTTP {status})")
}

/// The snapshot endpoint the worker pulls the mirror from.
pub const SNAPSHOT_PATH: &str = "/api/diary/snapshot";

/// The token mint for direct client→SurrealDB sync (flag-gated; 404 when
/// the flag is off).
pub const TOKEN_PATH: &str = "/api/diary/token";
/// Direct sync's request-side generation fence. Requiring it prevents a
/// worker from before [`DirectTokenGrant`] existed from receiving a token.
pub const WIRE_VERSION_HEADER: &str = "x-diary-wire-version";

/// The cookie-gated grant that arms one direct database sync pass. Exact
/// generation agreement is required because direct mode bypasses the
/// versioned HTTP push/snapshot envelopes entirely.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DirectTokenGrant {
    pub token: String,
    pub ns: String,
    pub db: String,
    pub version: u16,
}

impl DirectTokenGrant {
    pub fn new(token: impl Into<String>, ns: impl Into<String>, db: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            ns: ns.into(),
            db: db.into(),
            version: CURRENT_WIRE_VERSION,
        }
    }

    pub fn supports_current(&self) -> bool {
        self.version == CURRENT_WIRE_VERSION
    }
}

/// The versioned snapshot wire shape.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotWire {
    pub version: u16,
    pub entries: Vec<DiaryEntry>,
}

impl SnapshotWire {
    pub fn new(entries: Vec<DiaryEntry>) -> Self {
        Self {
            version: CURRENT_WIRE_VERSION,
            entries,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SnapshotWireV1 {
    version: u16,
    entries: Vec<DiaryEntryV1>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct UnversionedSnapshotWire {
    entries: Vec<DiaryEntryV1>,
}

pub fn decode_snapshot(body: &[u8]) -> Result<Vec<DiaryEntry>, WireError> {
    match probe_version(body)? {
        None => decode_exact::<UnversionedSnapshotWire>(body)
            .map(|wire| wire.entries.into_iter().map(DiaryEntry::from).collect()),
        Some(BODY_ONLY_WIRE_VERSION) => decode_exact::<SnapshotWireV1>(body)
            .map(|wire| wire.entries.into_iter().map(DiaryEntry::from).collect()),
        Some(CURRENT_WIRE_VERSION) => decode_exact::<SnapshotWire>(body).map(|wire| wire.entries),
        Some(version) => Err(WireError::UnsupportedVersion(version)),
    }
}

/// What one pull attempt means for the mirror.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PullOutcome {
    /// A strictly parsed snapshot from OUR server. The only outcome that
    /// may touch the mirror.
    Data(Vec<DiaryEntry>),
    /// 401/404: signed out or the wrong account. No mirror changes.
    Auth,
    /// Anything else — network failure, 5xx, or a 200 whose body is not our
    /// exact snapshot JSON (a captive portal). No mirror changes. This
    /// classification is load-bearing: a pull that read hotel-wifi HTML as
    /// "zero entries" would delete every synced row on the device.
    Retry,
}

/// Classify one response from the snapshot endpoint, mirroring
/// [`classify_response`]'s rules for the push side.
pub fn classify_pull(status: u16, body: &str) -> PullOutcome {
    match status {
        200..=299 => match decode_snapshot(body.as_bytes()) {
            Ok(entries) => PullOutcome::Data(entries),
            Err(_) => PullOutcome::Retry,
        },
        401 | 404 => PullOutcome::Auth,
        _ => PullOutcome::Retry,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_entries_require_canonical_fields_and_types() {
        let entry: ComposedEntry =
            serde_json::from_slice(br#"{"written_at": 1753640000, "body": "Dear diary,"}"#)
                .unwrap();
        assert_eq!(entry.written_at, 1_753_640_000);
        assert_eq!(entry.body, "Dear diary,");
        assert_eq!(entry.reply_to, None);
        for bad in [
            &br#"{"written_at": 1753640000.5, "body": "x"}"#[..],
            br#"{"written_at": "1753640000", "body": "x"}"#,
            br#"{"body": "x"}"#,
            br#"{"written_at": 1753640000}"#,
            b"not json",
        ] {
            assert!(
                serde_json::from_slice::<ComposedEntry>(bad).is_err(),
                "accepted {:?}",
                String::from_utf8_lossy(bad)
            );
        }
        // A generation is exact: accepting an unknown business field would
        // acknowledge and then silently discard newer entry semantics.
        assert_eq!(
            decode_push(br#"{"written_at":1753640000,"body":"x","extra":true}"#),
            Err(WireError::Malformed)
        );
    }

    /// What the worker sends is exactly what the server parses: the wire
    /// struct round-trips through its own serialization.
    #[test]
    fn wire_entries_round_trip() {
        let parent = "2026-07-27T14-30-45-04-00";
        let entry = ComposedEntry::new(1_753_640_000, "Dear diary,\n\nIt me.")
            .with_reply_to(Some(parent.to_string()));
        let json = serde_json::to_vec(&PushWire::new(entry.clone())).unwrap();
        let back = decode_push(&json).unwrap();
        assert_eq!(back, entry);
        assert_eq!(back.reply_to.as_deref(), Some(parent));

        let top_level =
            serde_json::to_string(&PushWire::new(ComposedEntry::new(1, "plain"))).unwrap();
        assert!(
            !top_level.contains("reply_to"),
            "an absent relationship stays off the current wire"
        );
    }

    #[test]
    fn version_probes_distinguish_skew_from_malformed_json() {
        let future = br#"{"version":99,"entry":{"written_at":1,"body":"x"}}"#;
        assert_eq!(decode_push(future), Err(WireError::UnsupportedVersion(99)));
        assert_eq!(decode_push(b"not json"), Err(WireError::Malformed));
        // Known generations are strict all the way through the flattened
        // canonical value; a semantic field must bump the version.
        assert_eq!(
            decode_push(br#"{"version":1,"entry":{"written_at":1,"body":"x","extra":true}}"#),
            Err(WireError::Malformed)
        );
        // Both body-only generations translate into canonical top-level
        // entries, but neither may consume and discard the reply field.
        for legacy in [
            &br#"{"written_at":1,"body":"unversioned"}"#[..],
            br#"{"version":1,"entry":{"written_at":1,"body":"v1"}}"#,
        ] {
            assert_eq!(decode_push(legacy).unwrap().reply_to, None);
        }
        for mislabeled_reply in [
            &br#"{"written_at":1,"body":"old","reply_to":"2026-07-27T14-30-45-04-00"}"#[..],
            br#"{"version":1,"entry":{"written_at":1,"body":"old","reply_to":"2026-07-27T14-30-45-04-00"}}"#,
        ] {
            assert_eq!(decode_push(mislabeled_reply), Err(WireError::Malformed));
        }
    }

    #[test]
    fn compose_command_is_one_strict_versioned_value() {
        let command = ComposeCommand::new(
            ComposedEntry::new(12, "queued")
                .with_reply_to(Some("2026-07-27T14-30-45-04-00".to_string())),
            34_000,
        );
        let json = serde_json::to_string(&command).unwrap();
        assert_eq!(decode_compose_command(&json).unwrap(), command);
        assert_eq!(
            decode_compose_command(
                r#"{"version":1,"entry":{"written_at":12,"body":"queued"},"enqueued_at_ms":34000}"#
            ),
            Err(WireError::UnsupportedVersion(1))
        );
        assert_eq!(
            decode_compose_command(
                r#"{"version":2,"entry":{"written_at":12,"body":"queued"},"enqueued_at_ms":34000,"extra":true}"#
            ),
            Err(WireError::Malformed)
        );
        assert_eq!(
            decode_compose_command(
                r#"{"version":2,"entry":{"written_at":12,"body":"queued","extra":true},"enqueued_at_ms":34000}"#
            ),
            Err(WireError::Malformed)
        );
        assert_eq!(
            decode_compose_command(
                r#"{"version":99,"entry":{"written_at":12,"body":"queued"},"enqueued_at_ms":34000}"#
            ),
            Err(WireError::UnsupportedVersion(99))
        );
    }

    #[test]
    fn snapshots_carry_canonical_entries_and_accept_the_legacy_envelope() {
        let top_level = DiaryEntry::from_parts("one", 1, "first");
        let reply = DiaryEntry::new(
            "two",
            ComposedEntry::new(2, "second")
                .with_reply_to(Some("2026-07-27T14-30-45-04-00".to_string())),
        );
        let entries = vec![top_level.clone(), reply];
        let current = serde_json::to_vec(&SnapshotWire::new(entries.clone())).unwrap();
        assert_eq!(decode_snapshot(&current).unwrap(), entries);

        let legacy = br#"{"entries":[{"id":"one","written_at":1,"body":"first"}]}"#;
        assert_eq!(decode_snapshot(legacy).unwrap(), vec![top_level.clone()]);
        let v1 = br#"{"version":1,"entries":[{"id":"one","written_at":1,"body":"first"}]}"#;
        assert_eq!(decode_snapshot(v1).unwrap(), vec![top_level]);

        for mislabeled_reply in [
            &br#"{"entries":[{"id":"two","written_at":2,"body":"second","reply_to":"2026-07-27T14-30-45-04-00"}]}"#[..],
            br#"{"version":1,"entries":[{"id":"two","written_at":2,"body":"second","reply_to":"2026-07-27T14-30-45-04-00"}]}"#,
        ] {
            assert_eq!(
                decode_snapshot(mislabeled_reply),
                Err(WireError::Malformed)
            );
        }
        assert_eq!(
            decode_snapshot(
                br#"{"version":1,"entries":[{"id":"one","written_at":1,"body":"first","extra":true}]}"#
            ),
            Err(WireError::Malformed)
        );
        assert_eq!(
            decode_snapshot(br#"{"version":99,"entries":[]}"#),
            Err(WireError::UnsupportedVersion(99))
        );
    }

    #[test]
    fn responses_classify_like_the_old_worker() {
        // A save carries the server-assigned identity out of the response —
        // the page renders the delivered message from these fields.
        assert_eq!(
            classify_response(
                200,
                r#"{"status":"saved","id":"x","written_at":1,"deduped":false}"#
            ),
            SendOutcome::Saved(SavedRef {
                id: "x".to_string(),
                written_at: 1,
            })
        );
        // A 200 that is not our JSON is a captive portal, not a save; our
        // server never sends "saved" without the identity fields, so a
        // partial shape classifies the same way (the retry re-asks and the
        // dedupe absorbs it).
        assert_eq!(
            classify_response(200, "<html>hotel wifi</html>"),
            SendOutcome::Retry
        );
        assert_eq!(classify_response(200, ""), SendOutcome::Retry);
        assert_eq!(
            classify_response(200, r#"{"status":"saved"}"#),
            SendOutcome::Retry
        );
        assert_eq!(classify_response(401, ""), SendOutcome::Auth);
        assert_eq!(classify_response(404, ""), SendOutcome::Auth);
        assert_eq!(classify_response(400, ""), SendOutcome::Retry);
        for permanent in [409, 413, 415, 422] {
            assert_eq!(
                classify_response(permanent, ""),
                SendOutcome::Rejected(permanent),
                "status {permanent}"
            );
        }
        // 403 is a same-origin/config failure and 5xx an outage — never the
        // entry's fault, always retried later.
        for retryable in [403, 500, 502, 503] {
            assert_eq!(classify_response(retryable, ""), SendOutcome::Retry);
        }
        assert_eq!(rejection_reason(422), "rejected (HTTP 422)");
    }

    #[test]
    fn direct_grants_fence_unversioned_database_sync() {
        let grant = DirectTokenGrant::new("token", "namespace", "database");
        assert!(grant.supports_current());
        let json = serde_json::to_string(&grant).unwrap();
        assert!(json.contains(&format!(r#""version":{CURRENT_WIRE_VERSION}"#)));
        let future = json.replace(
            &format!(r#""version":{CURRENT_WIRE_VERSION}"#),
            r#""version":99"#,
        );
        let future: DirectTokenGrant = serde_json::from_str(&future).unwrap();
        assert!(!future.supports_current());
    }
}
