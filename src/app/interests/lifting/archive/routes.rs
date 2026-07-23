//! `/api/fitness/*` — the public fitness archive and private bounded
//! import path, ported from `deploy/src/fitness.ts`. Bodies, error
//! messages, status codes, and headers are contract (golden fixtures in
//! `tests/fixtures/api`). Public GET reads carry `Access-Control-Allow-
//! Origin: *`; import responses never did and still don't.

use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{Bytes, StatusCode, headers, path_param, route, uri},
};

use benjisponge::auth::bearer_authorized;
use benjisponge::data::Data;

use super::db;
use super::eastern;
use super::filters::parse_filters;
use super::import::{BODY_LIMIT_BYTES, parse_import_payload};
use super::store::FitnessStore;

pub const FITNESS_SYNC_TOKEN_VAR: &str = "FITNESS_SYNC_TOKEN";

type PublicResponse = (StatusCode, [(&'static str, &'static str); 3], String);
type PrivateResponse = (StatusCode, [(&'static str, &'static str); 2], String);

const PUBLIC_HEADERS: [(&str, &str); 3] = [
    ("Content-Type", "application/json; charset=utf-8"),
    ("Cache-Control", "no-store"),
    ("Access-Control-Allow-Origin", "*"),
];

const PRIVATE_HEADERS: [(&str, &str); 2] = [
    ("Content-Type", "application/json; charset=utf-8"),
    ("Cache-Control", "no-store"),
];

fn public(status: StatusCode, body: String) -> PublicResponse {
    (status, PUBLIC_HEADERS, body)
}

fn public_error(status: StatusCode, message: &str) -> PublicResponse {
    public(status, serde_json::json!({ "error": message }).to_string())
}

fn private(status: StatusCode, body: String) -> PrivateResponse {
    (status, PRIVATE_HEADERS, body)
}

fn private_error(status: StatusCode, message: &str) -> PrivateResponse {
    private(status, serde_json::json!({ "error": message }).to_string())
}

fn log_failure(path: &str, error: impl std::fmt::Display) {
    eprintln!(
        "{}",
        serde_json::json!({
            "message": "fitness api failed",
            "path": path,
            "error": error.to_string(),
        })
    );
}

fn to_body<T: Serialize>(payload: &T) -> String {
    serde_json::to_string(payload).expect("api payloads are plain data")
}

/// `url.search !== ""` — a bare trailing `?` produces an empty search in
/// the Worker too, so only a non-empty query trips the rejection.
fn has_query(cx: &Cx) -> bool {
    uri(cx).query().is_some_and(|query| !query.is_empty())
}

fn query_pairs(cx: &Cx) -> Vec<(String, String)> {
    form_urlencoded::parse(uri(cx).query().unwrap_or("").as_bytes())
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect()
}

#[route(GET "/api/fitness/sets")]
async fn list_sets(cx: &Cx) -> Result<PublicResponse> {
    let filters = match parse_filters(&query_pairs(cx)) {
        Ok(filters) => filters,
        Err(message) => return Ok(public_error(StatusCode::BAD_REQUEST, &message)),
    };
    Ok(match app_context::<FitnessStore>(cx).snapshot().await {
        Ok(snapshot) => public(StatusCode::OK, to_body(&snapshot.sets_page(&filters))),
        Err(error) => {
            log_failure("/api/fitness/sets", error);
            public_error(StatusCode::INTERNAL_SERVER_ERROR, "internal error")
        }
    })
}

#[route(GET "/api/fitness/facets")]
async fn list_facets(cx: &Cx) -> Result<PublicResponse> {
    if has_query(cx) {
        return Ok(public_error(
            StatusCode::BAD_REQUEST,
            "facets does not accept filters",
        ));
    }
    Ok(match app_context::<FitnessStore>(cx).snapshot().await {
        Ok(snapshot) => public(StatusCode::OK, to_body(&snapshot.facets())),
        Err(error) => {
            log_failure("/api/fitness/facets", error);
            public_error(StatusCode::INTERNAL_SERVER_ERROR, "internal error")
        }
    })
}

#[route(GET "/api/fitness/calendar")]
async fn list_calendar(cx: &Cx) -> Result<PublicResponse> {
    if has_query(cx) {
        return Ok(public_error(
            StatusCode::BAD_REQUEST,
            "calendar does not accept filters",
        ));
    }
    Ok(match app_context::<FitnessStore>(cx).snapshot().await {
        Ok(snapshot) => public(StatusCode::OK, to_body(&snapshot.calendar())),
        Err(error) => {
            log_failure("/api/fitness/calendar", error);
            public_error(StatusCode::INTERNAL_SERVER_ERROR, "internal error")
        }
    })
}

#[route(GET "/api/fitness/workouts/latest")]
async fn latest_workout(cx: &Cx) -> Result<PublicResponse> {
    if has_query(cx) {
        return Ok(public_error(
            StatusCode::BAD_REQUEST,
            "latest workout does not accept filters",
        ));
    }
    Ok(match app_context::<FitnessStore>(cx).snapshot().await {
        Ok(snapshot) => public(StatusCode::OK, to_body(&snapshot.latest())),
        Err(error) => {
            log_failure("/api/fitness/workouts/latest", error);
            public_error(StatusCode::INTERNAL_SERVER_ERROR, "internal error")
        }
    })
}

#[path_param]
struct PublicWorkoutPath(str);

#[route(GET "/api/fitness/workouts/by-path/{public_workout_path}")]
async fn workout_by_path(cx: &Cx) -> Result<PublicResponse> {
    if has_query(cx) {
        return Ok(public_error(
            StatusCode::BAD_REQUEST,
            "workout does not accept filters",
        ));
    }
    let segment = path_param::<PublicWorkoutPath>(cx);
    let Some(instant) = eastern::parse_public_path(segment) else {
        return Ok(public_error(StatusCode::NOT_FOUND, "not found"));
    };
    Ok(match app_context::<FitnessStore>(cx).snapshot().await {
        Ok(snapshot) => match snapshot.by_path(&instant) {
            Some(detail) => public(StatusCode::OK, to_body(&detail)),
            None => public_error(StatusCode::NOT_FOUND, "not found"),
        },
        Err(error) => {
            log_failure("/api/fitness/workouts/by-path", error);
            public_error(StatusCode::INTERNAL_SERVER_ERROR, "internal error")
        }
    })
}

#[route(GET "/api/fitness/ids")]
async fn list_ids(cx: &Cx) -> Result<PublicResponse> {
    if has_query(cx) {
        return Ok(public_error(
            StatusCode::BAD_REQUEST,
            "ids does not accept filters",
        ));
    }
    Ok(match app_context::<FitnessStore>(cx).snapshot().await {
        Ok(snapshot) => public(StatusCode::OK, to_body(&snapshot.ids())),
        Err(error) => {
            log_failure("/api/fitness/ids", error);
            public_error(StatusCode::INTERNAL_SERVER_ERROR, "internal error")
        }
    })
}

/// The Worker's `readJson` Content-Length handling: `Number(header)` must
/// be a non-negative integer, and anything over the limit is a 413 before
/// the body is even considered.
fn content_length_error(cx: &Cx) -> Option<PrivateResponse> {
    let declared = headers(cx)
        .get("content-length")
        .and_then(|value| value.to_str().ok())?;
    let parsed: Option<f64> = {
        let trimmed = declared.trim();
        if trimmed.is_empty() {
            Some(0.0)
        } else {
            trimmed.parse().ok()
        }
    };
    let Some(length) = parsed.filter(|n| n.is_finite() && n.fract() == 0.0 && *n >= 0.0) else {
        return Some(private_error(StatusCode::BAD_REQUEST, "bad Content-Length"));
    };
    if length > BODY_LIMIT_BYTES as f64 {
        return Some(private_error(
            StatusCode::PAYLOAD_TOO_LARGE,
            &format!("body exceeds {BODY_LIMIT_BYTES} bytes"),
        ));
    }
    None
}

#[route(POST "/api/fitness/import")]
async fn import_chunk(cx: &Cx, body: Bytes) -> Result<PrivateResponse> {
    let authorization = headers(cx)
        .get("authorization")
        .and_then(|value| value.to_str().ok());
    let expected = std::env::var(FITNESS_SYNC_TOKEN_VAR).ok();
    if !bearer_authorized(authorization, expected.as_deref()) {
        return Ok(private_error(StatusCode::UNAUTHORIZED, "unauthorized"));
    }

    let media_type = headers(cx)
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .map(|value| value.trim().to_ascii_lowercase());
    if media_type.as_deref() != Some("application/json") {
        return Ok(private_error(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Content-Type must be application/json",
        ));
    }

    if let Some(response) = content_length_error(cx) {
        return Ok(response);
    }
    if body.len() > BODY_LIMIT_BYTES {
        return Ok(private_error(
            StatusCode::PAYLOAD_TOO_LARGE,
            &format!("body exceeds {BODY_LIMIT_BYTES} bytes"),
        ));
    }
    let Ok(decoded) = serde_json::from_slice::<serde_json::Value>(&body) else {
        return Ok(private_error(StatusCode::BAD_REQUEST, "body must be JSON"));
    };
    let payload = match parse_import_payload(&decoded) {
        Ok(payload) => payload,
        Err(message) => return Ok(private_error(StatusCode::BAD_REQUEST, &message)),
    };

    let imported_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or(0);
    let data = app_context::<Data>(cx);
    let outcome = async {
        let handle = data.db().await?;
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(
            db::apply_import(&handle, &payload, imported_at).await?,
        )
    }
    .await;

    Ok(match outcome {
        Ok(outcome) => {
            if outcome.mutated
                && let Err(error) = app_context::<FitnessStore>(cx).rebuild().await
            {
                // The commit landed; a rebuild failure only delays
                // freshness until the next read's version check.
                log_failure("/api/fitness/import", error);
            }
            private(
                StatusCode::OK,
                to_body(&super::api::ImportReceipt {
                    received: outcome.received,
                    added: outcome.added,
                    skipped: outcome.skipped,
                    version: outcome.version,
                }),
            )
        }
        Err(error) => {
            log_failure("/api/fitness/import", error);
            private_error(StatusCode::INTERNAL_SERVER_ERROR, "internal error")
        }
    })
}

/// Unmatched `/api/fitness/*` paths mirror the Worker's fallthrough:
/// JSON 404s, with CORS only on GET (`isPublicRead`).
#[route(GET "/api/fitness/{*rest}")]
async fn unknown_get() -> Result<PublicResponse> {
    Ok(public_error(StatusCode::NOT_FOUND, "not found"))
}

#[route(POST "/api/fitness/{*rest}")]
async fn unknown_post() -> Result<PrivateResponse> {
    Ok(private_error(StatusCode::NOT_FOUND, "not found"))
}
