//! The Slay the Spire 2 run log, read from this site's own Worker API
//! (`GET /api/spire/runs`, backed by Cloudflare D1). The sync CLI
//! (`src/bin/spire_sync.rs`) is the write path and does all the prettifying,
//! so a [`Run`] arrives display-ready. Responses are cached in-process for a
//! minute; a failed fetch serves the last good copy, or an empty log flagged
//! `live: false` — run data must never 500 a page.

use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde::Deserialize;

/// One synced run — the fields the site renders. The API serves more (seed,
/// acts, build id); serde drops what the pages don't use.
#[derive(Clone, Debug, Deserialize)]
pub struct Run {
    /// The run file's stem — also its `start_time`, as a string.
    pub id: String,
    /// `YYYY-MM-DD`, US Eastern, stamped by the sync CLI.
    pub date: String,
    /// Epoch seconds; the log's sort key.
    pub start_time: i64,
    /// Prettified, e.g. "Necrobinder"; co-op runs join with " + ".
    pub character: String,
    pub win: bool,
    pub abandoned: bool,
    pub ascension: u32,
    pub floors: u32,
    /// Prettified killer, e.g. "Knowledge Demon"; `None` on wins/abandons.
    pub killed_by: Option<String>,
    /// "boss" | "elite" | "monster" | "event", when `killed_by` is set.
    pub kill_kind: Option<String>,
    /// Seconds of play.
    pub run_time: u64,
    /// "standard" for almost everything; "daily" runs get a marker.
    pub game_mode: String,
}

impl Run {
    /// The run-log table's result cell: "won", "died to X", "abandoned".
    pub fn result_label(&self) -> String {
        if self.win {
            "won".to_string()
        } else if self.abandoned {
            "abandoned".to_string()
        } else {
            match &self.killed_by {
                Some(killer) => format!("died to {killer}"),
                None => "died".to_string(),
            }
        }
    }
}

/// What a page gets: the runs (newest first) and whether they are real.
pub struct RunLog {
    pub runs: Arc<Vec<Run>>,
    /// False only when the API was unreachable AND no cached copy existed —
    /// the one case pages should admit the log is missing rather than empty.
    pub live: bool,
}

/// "7534 s" → "2h 05m"; sub-hour runs render as "48m".
pub fn fmt_duration(secs: u64) -> String {
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    if hours > 0 {
        format!("{hours}h {minutes:02}m")
    } else {
        format!("{minutes}m")
    }
}

/// Where the run API lives. `SPIRE_DATA_ORIGIN` wins (point dev at a local
/// Worker), then `SITE_ORIGIN` (set in the production container), then the
/// real site — so `just dev` renders production data with zero setup.
fn origin() -> String {
    std::env::var("SPIRE_DATA_ORIGIN")
        .or_else(|_| std::env::var("SITE_ORIGIN"))
        .unwrap_or_else(|_| "https://benjisponge.com".to_string())
}

fn client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(4))
            .user_agent("benjisponge-site (+https://benjisponge.com/spire)")
            .build()
            .expect("reqwest client")
    })
}

static CACHE: Mutex<Option<(Instant, Arc<Vec<Run>>)>> = Mutex::new(None);
const TTL: Duration = Duration::from_secs(60);
pub const SPIRE_CACHE_REFRESH_HEADER: &str = "x-spire-cache-refresh";

/// Fetch the run log, through the cache. `force_refresh` is set by the Worker
/// on a data-versioned edge-cache miss: that request must refresh before its
/// rendered page can be stored under the new version. Ordinary page requests
/// still share one upstream fetch per minute.
pub async fn load(force_refresh: bool) -> RunLog {
    if !force_refresh
        && let Some((at, runs)) = CACHE.lock().unwrap().clone()
        && at.elapsed() < TTL
    {
        return RunLog { runs, live: true };
    }
    match fetch().await {
        Ok(runs) => {
            let runs = Arc::new(runs);
            *CACHE.lock().unwrap() = Some((Instant::now(), Arc::clone(&runs)));
            RunLog { runs, live: true }
        }
        Err(err) => {
            eprintln!("spire run fetch failed: {err}");
            // A stale copy beats an empty page; the TTL stays expired, so the
            // next request retries the fetch.
            match CACHE.lock().unwrap().clone() {
                Some((_, runs)) => RunLog { runs, live: true },
                None => RunLog {
                    runs: Arc::new(Vec::new()),
                    live: false,
                },
            }
        }
    }
}

async fn fetch() -> Result<Vec<Run>, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!("{}/api/spire/runs", origin());
    let body = client()
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    Ok(parse(&body)?)
}

#[derive(Deserialize)]
struct ApiResponse {
    runs: Vec<Run>,
}

/// Parse the API payload: newest first, and any run whose date isn't
/// `YYYY-MM-DD` is dropped (the feed's RFC 2822 conversion indexes into that
/// shape, and one bad row must not take out `/feed.xml`).
fn parse(body: &str) -> serde_json::Result<Vec<Run>> {
    let api: ApiResponse = serde_json::from_str(body)?;
    let mut runs: Vec<Run> = api
        .runs
        .into_iter()
        .filter(|r| {
            let ok = iso_date(&r.date);
            if !ok {
                eprintln!(
                    "spire run {} has malformed date {:?}; dropped",
                    r.id, r.date
                );
            }
            ok
        })
        .collect();
    runs.sort_unstable_by_key(|r| std::cmp::Reverse(r.start_time));
    Ok(runs)
}

fn iso_date(date: &str) -> bool {
    let bytes = date.as_bytes();
    date.len() == 10
        && bytes.iter().enumerate().all(|(i, b)| match i {
            4 | 7 => *b == b'-',
            _ => b.is_ascii_digit(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
      "version": 3,
      "count": 2,
      "runs": [
        {"id": "1772889032", "date": "2026-03-07", "start_time": 1772889032,
         "character": "Silent", "win": false, "abandoned": false,
         "ascension": 0, "acts": 2, "floors": 21,
         "killed_by": "Knowledge Demon", "kill_kind": "boss",
         "run_time": 3600, "seed": "AAAA", "game_mode": "standard",
         "build_id": "v0.98.2"},
        {"id": "1784587453", "date": "2026-07-20", "start_time": 1784587453,
         "character": "Necrobinder", "win": true, "abandoned": false,
         "ascension": 3, "acts": 3, "floors": 34,
         "killed_by": null, "kill_kind": null,
         "run_time": 7534, "seed": "6CCBAWXEKT", "game_mode": "standard",
         "build_id": "v0.107.1"}
      ]
    }"#;

    #[test]
    fn parses_and_sorts_newest_first() {
        let runs = parse(SAMPLE).unwrap();
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].id, "1784587453");
        assert!(runs[0].win);
        assert_eq!(runs[1].killed_by.as_deref(), Some("Knowledge Demon"));
    }

    #[test]
    fn malformed_dates_are_dropped_not_fatal() {
        let bad = SAMPLE.replace("2026-03-07", "March 7th");
        let runs = parse(&bad).unwrap();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].id, "1784587453");
    }

    #[test]
    fn result_labels() {
        let runs = parse(SAMPLE).unwrap();
        assert_eq!(runs[0].result_label(), "won");
        assert_eq!(runs[1].result_label(), "died to Knowledge Demon");
        let mut abandoned = runs[1].clone();
        abandoned.abandoned = true;
        abandoned.killed_by = None;
        assert_eq!(abandoned.result_label(), "abandoned");
    }

    #[test]
    fn durations_read_like_a_clock() {
        assert_eq!(fmt_duration(7534), "2h 05m");
        assert_eq!(fmt_duration(2891), "48m");
        assert_eq!(fmt_duration(59), "0m");
        assert_eq!(fmt_duration(3600), "1h 00m");
    }
}
