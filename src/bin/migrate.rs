//! Migrations CLI for the site's Postgres database.
//!
//! Built on the toasty-cli library (there is no prebuilt binary). Artifacts
//! live under `toasty/` at the repo root (`history.toml`, `migrations/`,
//! `snapshots/`) and are committed; applied migrations are tracked in the
//! database's `__toasty_migrations` table.
//!
//! Usage (POSTGRES_URL selects the target database — prod from `.env` via
//! `just migrate`, the local docker database from `scripts/dev.sh`):
//!
//! ```sh
//! cargo run --bin migrate -- migration generate --name add_spire_tables
//! cargo run --bin migrate -- migration apply
//! ```

use toasty_cli::{Config, ToastyCli};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let url = std::env::var(benjisponge::data::POSTGRES_URL_VAR).map_err(|_| {
        anyhow::anyhow!(
            "{} must be set (prod: `just migrate ...`; local dev database: scripts/dev.sh)",
            benjisponge::data::POSTGRES_URL_VAR
        )
    })?;

    // Creates a default Toasty.toml on first run; migration artifacts live
    // under toasty/ and are committed.
    let config = Config::load_or_default(std::path::Path::new("."))?;
    let db = benjisponge::data::connect(&url).await?;
    ToastyCli::with_config(db, config).parse_and_run().await?;
    Ok(())
}
