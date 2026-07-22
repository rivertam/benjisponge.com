//! Server-side reader for `/lifting` and its D1-backed fitness archive.
//!
//! The Worker owns filtering and validation. The site forwards the normalized
//! `/lifting/log` query as repeated key/value pairs, then renders the typed JSON
//! response as HTML. Keeping the API boundary here makes the page useful with
//! no browser runtime at all.

use std::{fmt, sync::OnceLock, time::Duration};

use reqwest::StatusCode;
use serde::Deserialize;

use crate::util::urlencode;

#[derive(Debug, Deserialize)]
pub struct Facets {
    pub summary: Summary,
    pub exercises: Vec<Facet>,
}

#[derive(Debug, Deserialize)]
pub struct Summary {
    pub sets: u64,
    pub workouts: u64,
    pub min_date: Option<String>,
    pub max_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Facet {
    pub value: String,
    pub count: u64,
}

#[derive(Debug, Deserialize)]
pub struct SetPage {
    pub page: usize,
    pub per_page: usize,
    pub total_sets: u64,
    pub total_workouts: u64,
    pub workouts: Vec<Workout>,
}

#[derive(Debug, Deserialize)]
pub struct Workout {
    /// The Worker-generated, canonical public path segment. The immutable ID
    /// stays internal to the archive so importing and URL presentation can
    /// evolve independently.
    pub path: String,
    pub title: String,
    pub started_at_local: String,
    pub ended_at_local: String,
    pub eastern_offset_minutes: i32,
    pub end_eastern_offset_minutes: i32,
    pub duration_seconds: u64,
    pub duration_suspicious: bool,
    pub notes: Option<String>,
    pub description: Option<String>,
    pub sets: Vec<Set>,
}

#[derive(Debug, Deserialize)]
pub struct Set {
    pub ordinal: u32,
    pub exercise_name: String,
    pub exercise_note: Option<String>,
    pub superset_id: Option<u64>,
    pub weight_milli: Option<u64>,
    pub weight_unit: String,
    pub reps: Option<u64>,
    pub effort_hundredths: Option<u64>,
    pub distance_milli: Option<u64>,
    pub set_time_seconds: Option<u64>,
    pub set_type: String,
    pub records: Vec<Record>,
}

#[derive(Debug, Deserialize)]
pub struct Record {
    pub level: String,
    pub kind: String,
}

#[derive(Debug, Deserialize)]
pub struct Calendar {
    pub days: Vec<CalendarDay>,
}

#[derive(Debug, Deserialize)]
pub struct CalendarDay {
    pub date: String,
    pub volume_points: u32,
}

#[derive(Debug, Deserialize)]
pub struct WorkoutDetail {
    pub workout: Option<Workout>,
    pub newer_workout_path: Option<String>,
    pub older_workout_path: Option<String>,
}

/// A rejected filter is safe to show to the reader. Transport, upstream, and
/// JSON failures are logged by the page but deliberately rendered generically.
#[derive(Debug)]
pub enum LoadError {
    Rejected(String),
    NotFound(String),
    Unavailable(String),
}

impl LoadError {
    pub fn rejected_message(&self) -> Option<&str> {
        match self {
            Self::Rejected(message) => Some(message),
            Self::NotFound(_) | Self::Unavailable(_) => None,
        }
    }

    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound(_))
    }
}

impl fmt::Display for LoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rejected(message) => write!(formatter, "fitness filter rejected: {message}"),
            Self::NotFound(message) => write!(formatter, "fitness resource not found: {message}"),
            Self::Unavailable(message) => write!(formatter, "fitness API unavailable: {message}"),
        }
    }
}

/// Both reads are independent D1 queries, so issue them concurrently.
pub async fn load(
    filters: &[(String, String)],
) -> (Result<Facets, LoadError>, Result<SetPage, LoadError>) {
    tokio::join!(fetch_facets(), fetch_sets(filters))
}

/// The landing view has no filters: it needs one current workout and the
/// archive-wide daily totals for its heatmap.
pub async fn load_home() -> (
    Result<Calendar, LoadError>,
    Result<WorkoutDetail, LoadError>,
) {
    tokio::join!(fetch_calendar(), fetch_latest_workout())
}

/// Resolve a current, Worker-owned public path. Do not infer an importer ID
/// from it: the path includes the Eastern UTC offset and is intentionally an
/// API-level mapping rather than a client-side convention.
pub async fn load_workout_by_path(path: &str) -> Result<WorkoutDetail, LoadError> {
    fetch_public_workout(path).await
}

async fn fetch_facets() -> Result<Facets, LoadError> {
    fetch("facets", &[]).await
}

async fn fetch_sets(filters: &[(String, String)]) -> Result<SetPage, LoadError> {
    fetch("sets", filters).await
}

async fn fetch_calendar() -> Result<Calendar, LoadError> {
    fetch("calendar", &[]).await
}

async fn fetch_latest_workout() -> Result<WorkoutDetail, LoadError> {
    fetch("workouts/latest", &[]).await
}

async fn fetch_public_workout(path: &str) -> Result<WorkoutDetail, LoadError> {
    fetch(&format!("workouts/by-path/{}", urlencode(path)), &[]).await
}

async fn fetch<T: for<'de> Deserialize<'de>>(
    path: &str,
    query: &[(String, String)],
) -> Result<T, LoadError> {
    let url = format!("{}/api/fitness/{path}", origin().trim_end_matches('/'));
    let response = client()
        .get(&url)
        .query(query)
        .send()
        .await
        .map_err(|error| LoadError::Unavailable(error.to_string()))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| LoadError::Unavailable(error.to_string()))?;

    if !status.is_success() {
        let message = serde_json::from_str::<ApiError>(&body)
            .map(|error| error.error)
            .unwrap_or_else(|_| {
                format!(
                    "{} {}",
                    status.as_u16(),
                    status.canonical_reason().unwrap_or("error")
                )
            });
        return if status == StatusCode::NOT_FOUND {
            Err(LoadError::NotFound(message))
        } else if status.is_client_error() {
            Err(LoadError::Rejected(message))
        } else {
            Err(LoadError::Unavailable(message))
        };
    }

    serde_json::from_str(&body).map_err(|error| LoadError::Unavailable(error.to_string()))
}

#[derive(Deserialize)]
struct ApiError {
    error: String,
}

/// `just dev` points this at its local Worker. The production container gets
/// `SITE_ORIGIN`; a direct binary run falls back to the public archive.
fn origin() -> String {
    std::env::var("FITNESS_DATA_ORIGIN")
        .or_else(|_| std::env::var("SITE_ORIGIN"))
        .unwrap_or_else(|_| "https://benjisponge.com".to_string())
}

fn client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(6))
            .user_agent("benjisponge-site (+https://benjisponge.com/lifting)")
            .build()
            .expect("reqwest client")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_public_api_shape_and_preserves_nulls() {
        let page: SetPage = serde_json::from_str(
            r#"{
              "version": 4, "page": 1, "per_page": 10,
              "total_sets": 1, "total_workouts": 1,
              "workouts": [{
                "id": "w1", "path": "2026-07-21T17-03-00-04-00",
                "title": "Leg day", "raw_title": "Leg day",
                "started_at_local": "2026-07-21 17:03:00",
                "ended_at_local": "2026-07-21 18:03:00",
                "eastern_offset_minutes": -240,
                "end_eastern_offset_minutes": -240,
                "duration_seconds": 3600, "duration_suspicious": false,
                "notes": null, "description": "hard",
                "sets": [{
                  "id": "s1", "ordinal": 1, "exercise_name": "Squat",
                  "raw_exercise_name": "Squat", "exercise_note": null,
                  "superset_id": null, "weight_milli": 102500, "weight_unit": "lbs", "reps": 5,
                  "effort_hundredths": null, "distance_milli": null,
                  "set_time_seconds": null, "set_type": "NORMAL_SET",
                  "records": [{"level": "gold", "kind": "volume"}]
                }]
              }]
            }"#,
        )
        .unwrap();

        assert_eq!(page.per_page, 10);
        assert_eq!(page.workouts[0].path, "2026-07-21T17-03-00-04-00");
        assert_eq!(page.workouts[0].eastern_offset_minutes, -240);
        assert_eq!(page.workouts[0].sets[0].weight_milli, Some(102_500));
        assert_eq!(page.workouts[0].sets[0].weight_unit, "lbs");
        assert_eq!(page.workouts[0].sets[0].effort_hundredths, None);
        assert_eq!(page.workouts[0].sets[0].records[0].kind, "volume");
    }

    #[test]
    fn parses_calendar_and_linkable_workout_envelopes() {
        let calendar: Calendar = serde_json::from_str(
            r#"{"version":4,"days":[{"date":"2026-07-21","volume_points":42}]}"#,
        )
        .unwrap();
        assert_eq!(calendar.days[0].date, "2026-07-21");
        assert_eq!(calendar.days[0].volume_points, 42);

        let detail: WorkoutDetail = serde_json::from_str(
            r#"{
              "version":4,
              "workout":null,
              "newer_workout_path":null,
              "older_workout_path":"2026-07-18T16-19-36-04-00"
            }"#,
        )
        .unwrap();
        assert!(detail.workout.is_none());
        assert_eq!(
            detail.older_workout_path.as_deref(),
            Some("2026-07-18T16-19-36-04-00")
        );
    }
}
