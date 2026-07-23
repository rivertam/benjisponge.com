//! Spire toasty models — the schema source of truth for the spire tables.
//!
//! This file lives with its interest but is compiled as part of the LIB
//! crate: `src/data.rs` pulls it in via `#[path]` as
//! `benjisponge::data::spire_models`, because the migrations CLI
//! (`src/bin/migrate.rs`) and `toasty::models!` registration need every
//! model in the shared lib. Schema changes go through
//! `cargo run --bin migrate -- migration generate/apply`.

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
