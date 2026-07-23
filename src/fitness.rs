//! The fitness archive engine — the Rust replacement for the old
//! D1-backed Worker implementation (`fitness.ts`, since deleted).
//!
//! Reads are served from an in-memory [`snapshot`] of the whole archive
//! rather than per-request SQL: records are derived-not-stored (a
//! `has_record` filter that participates in counts and pagination has no
//! SQL expression once the `set_records` table is gone), and every
//! SQLite-ism the old Worker relied on (ASCII-only NOCASE, byte-order
//! sorts, NULL-excluding comparisons) has an exact pure-Rust mirror here
//! but no faithful Postgres collation. The database is read in full only
//! when the data version changes — an import, a few times a week.
//!
//! Error messages, envelope key order, and filter semantics are contract:
//! the golden fixtures under `tests/fixtures/api` capture the Worker's
//! originals verbatim.

pub mod api;
pub mod filters;
pub mod import;
pub mod snapshot;
pub mod store;
mod validate;
