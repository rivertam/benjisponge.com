//! Shared data-layer library.
//!
//! The site binary (`src/main.rs`) keeps its rendering modules private; this
//! crate holds the logic that must be shared across the server, the sync
//! CLIs, and the migrations CLI as the data layer moves in-process
//! (TypeScript Worker + D1 -> toasty + Postgres).

pub mod auth;
pub mod data;
pub mod eastern;
pub mod fitness;
pub mod records;
pub mod scoring;
