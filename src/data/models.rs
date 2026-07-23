//! Toasty models — the schema source of truth for the Postgres database.
//!
//! Every model lives in this one module so `toasty::models!` can register
//! them with a single glob. Schema changes go through the migrations CLI
//! (`cargo run --bin migrate -- migration generate/apply`), never
//! `push_schema`, once a database holds real data.
//!
//! D1's STRICT/CHECK constraints have no equivalent here: application-level
//! validation (the import endpoints) is the only line of defense.

/// A Slay the Spire 2 run, minus the original `.run` payload.
///
/// `raw` deliberately lives in [`SpireRunRaw`]: toasty hydrates whole rows,
/// and dragging ~100 KB of JSON per run into every list read would swamp
/// the container. Splitting the table makes `raw` write-only by
/// construction.
#[derive(Debug, toasty::Model)]
#[table = "spire_runs"]
pub struct SpireRun {
    #[key]
    pub id: String,
    pub date: String,
    #[index]
    pub start_time: i64,
    pub character: String,
    pub win: bool,
    pub abandoned: bool,
    pub ascension: i64,
    pub acts: i64,
    pub floors: i64,
    pub killed_by: Option<String>,
    pub kill_kind: Option<String>,
    pub run_time: i64,
    pub seed: String,
    pub game_mode: String,
    pub build_id: String,
    pub added_at: i64,
}

/// The whole original `.run` file, kept so future redesigns never need a
/// re-scrape. Written by import, read by nothing.
#[derive(Debug, toasty::Model)]
#[table = "spire_run_raws"]
pub struct SpireRunRaw {
    #[key]
    pub id: String,
    pub raw: String,
}

#[derive(Debug, toasty::Model)]
#[table = "spire_meta"]
pub struct SpireMeta {
    #[key]
    pub k: String,
    pub v: i64,
}

/// A lifting workout. `started_at_utc` is the Strong-export source instant
/// and the identity anchor; `started_at_local`/`eastern_offset_minutes` are
/// its America/New_York projection, derived server-side at import.
#[derive(Debug, toasty::Model)]
#[table = "workouts"]
pub struct Workout {
    #[key]
    pub id: String,
    pub title: String,
    pub raw_title: String,
    #[index]
    pub started_at_utc: String,
    pub started_at_local: String,
    pub eastern_offset_minutes: i64,
    pub duration_seconds: i64,
    pub duration_suspicious: bool,
    pub notes: Option<String>,
    pub description: Option<String>,
    pub source: String,
    pub imported_at: i64,
}

#[derive(Debug, toasty::Model)]
#[table = "exercises"]
pub struct Exercise {
    #[key]
    pub name: String,
}

/// One taxonomy tag on an exercise; an exercise carries several per facet.
#[derive(Debug, toasty::Model)]
#[table = "exercise_tags"]
#[key(exercise_name, kind, value)]
pub struct ExerciseTag {
    pub exercise_name: String,
    pub kind: String,
    pub value: String,
}

/// One performed set. There is deliberately no stored records table: badges
/// are derived from the full set history (`benjisponge::records`), so this
/// stays the only source of truth a future manual-logging write path needs.
#[derive(Debug, toasty::Model)]
#[table = "sets"]
#[unique(workout_id, ordinal)]
pub struct LiftSet {
    #[key]
    pub id: String,
    #[index]
    pub workout_id: String,
    pub exercise_name: String,
    pub raw_exercise_name: String,
    pub ordinal: i64,
    pub exercise_note: Option<String>,
    pub superset_id: Option<i64>,
    pub weight_milli: Option<i64>,
    pub weight_unit: String,
    pub reps: Option<i64>,
    pub effort_hundredths: Option<i64>,
    pub distance_milli: Option<i64>,
    pub set_time_seconds: Option<i64>,
    pub set_type: String,
    pub incomplete: bool,
}

#[derive(Debug, toasty::Model)]
#[table = "fitness_meta"]
pub struct FitnessMeta {
    #[key]
    pub k: String,
    pub v: i64,
}
