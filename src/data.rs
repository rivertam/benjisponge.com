//! Database access: toasty over Postgres.
//!
//! `Data` is the shareable handle the site puts in topcoat's app context.
//! It connects lazily so a missing/unreachable database never prevents the
//! binary from starting or serving non-data pages — readers treat a failed
//! `db()` like any other data-source error (stale cache or error card).

use std::sync::Arc;

use toasty::Db;
use tokio::sync::OnceCell;

pub mod fitness;
pub mod models;
pub mod spire;

pub const POSTGRES_URL_VAR: &str = "POSTGRES_URL";

#[derive(Clone)]
pub struct Data {
    url: Option<Arc<str>>,
    cell: Arc<OnceCell<Db>>,
}

#[derive(Debug)]
pub enum DataError {
    /// POSTGRES_URL is not configured for this process.
    Unconfigured,
    Connect(toasty::Error),
}

impl std::fmt::Display for DataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataError::Unconfigured => write!(f, "{POSTGRES_URL_VAR} is not set"),
            DataError::Connect(error) => write!(f, "database connect failed: {error}"),
        }
    }
}

impl std::error::Error for DataError {}

impl Data {
    pub fn from_env() -> Self {
        Self::new(std::env::var(POSTGRES_URL_VAR).ok())
    }

    pub fn new(url: Option<String>) -> Self {
        Data {
            url: url.filter(|value| !value.trim().is_empty()).map(Arc::from),
            cell: Arc::new(OnceCell::new()),
        }
    }

    /// A cheap clone of the shared `Db` (toasty statements borrow it
    /// mutably), connecting on first use.
    pub async fn db(&self) -> Result<Db, DataError> {
        let Some(url) = self.url.as_deref() else {
            return Err(DataError::Unconfigured);
        };
        let db = self
            .cell
            .get_or_try_init(|| connect(url))
            .await
            .map_err(DataError::Connect)?;
        Ok(db.clone())
    }
}

pub async fn connect(url: &str) -> toasty::Result<Db> {
    Db::builder()
        .models(toasty::models!(crate::*))
        .pool_pre_ping(true)
        .connect(url)
        .await
}
