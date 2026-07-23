//! Server-side reader for `/lifting` — in-process over the fitness
//! snapshot (`benjisponge::fitness`), no HTTP hop.
//!
//! The wire types are the lib's own API envelopes, so pages and the
//! public JSON endpoints can never drift. `LoadError` keeps its old
//! shape: `Rejected` messages are the filter validator's exact 400
//! strings and stay reader-visible; everything else renders generically.

use std::fmt;

use super::archive::eastern;
use super::archive::filters::parse_filters;
use super::archive::store::FitnessStore;

pub use super::archive::api::{
    Calendar, CalendarDay, Facets, Record, Set, SetPage, Workout, WorkoutDetail,
};

/// A rejected filter is safe to show to the reader. Snapshot failures are
/// logged by the page but deliberately rendered generically.
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
            Self::Unavailable(message) => {
                write!(formatter, "fitness archive unavailable: {message}")
            }
        }
    }
}

/// The full-log page's pair of reads. Both come from one snapshot, so the
/// facet counts and the filtered page can never disagree about versions.
pub async fn load(
    store: &FitnessStore,
    filters: &[(String, String)],
) -> (Result<Facets, LoadError>, Result<SetPage, LoadError>) {
    let snapshot = match store.snapshot().await {
        Ok(snapshot) => snapshot,
        Err(error) => {
            let message = error.to_string();
            return (
                Err(LoadError::Unavailable(message.clone())),
                Err(LoadError::Unavailable(message)),
            );
        }
    };
    let sets = match parse_filters(filters) {
        Ok(parsed) => Ok(snapshot.sets_page(&parsed)),
        Err(message) => Err(LoadError::Rejected(message)),
    };
    (Ok(snapshot.facets()), sets)
}

/// The landing view: archive-wide daily totals plus the newest workout.
pub async fn load_home(
    store: &FitnessStore,
) -> (
    Result<Calendar, LoadError>,
    Result<WorkoutDetail, LoadError>,
) {
    match store.snapshot().await {
        Ok(snapshot) => (Ok(snapshot.calendar()), Ok(snapshot.latest())),
        Err(error) => {
            let message = error.to_string();
            (
                Err(LoadError::Unavailable(message.clone())),
                Err(LoadError::Unavailable(message)),
            )
        }
    }
}

/// Resolve a canonical public path. Rejections mirror the API's 404s.
pub async fn load_workout_by_path(
    store: &FitnessStore,
    path: &str,
) -> Result<WorkoutDetail, LoadError> {
    let Some(instant) = eastern::parse_public_path(path) else {
        return Err(LoadError::NotFound("not found".to_string()));
    };
    let snapshot = store
        .snapshot()
        .await
        .map_err(|error| LoadError::Unavailable(error.to_string()))?;
    snapshot
        .by_path(&instant)
        .ok_or_else(|| LoadError::NotFound("not found".to_string()))
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
