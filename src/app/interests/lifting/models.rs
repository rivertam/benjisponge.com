//! Fitness toasty models — the schema source of truth for the archive.
//!
//! This file lives with its interest but is compiled as part of the LIB
//! crate: `src/data.rs` pulls it in via `#[path]` as
//! `benjisponge::data::fitness_models`, because the migrations CLI
//! (`src/bin/migrate.rs`) and `toasty::models!` registration need every
//! model in the shared lib. Schema changes go through
//! `cargo run --bin migrate -- migration generate/apply`.
//!
//! D1's STRICT/CHECK constraints have no equivalent here: the import
//! validation in `archive/validate.rs` is the only line of defense.

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
/// are derived from the full set history (`archive/records.rs`), so this
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
