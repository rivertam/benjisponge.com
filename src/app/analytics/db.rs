//! Analytics writes through Toasty.

use anyhow::{Context, anyhow};
use sha2::{Digest, Sha256};
use toasty::Executor;
use uuid::Uuid;

use benjisponge::data::analytics_models::{AnalyticsEvent, AnalyticsIdentity};

use super::input::ValidatedEvent;

pub fn hash_identifier(namespace: &str, value: &str) -> String {
    let mut hash = Sha256::new();
    hash.update(namespace);
    hash.update([0]);
    hash.update(value);
    hash.finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Resolve a hardened Topcoat cookie to the stable anonymous visitor chosen
/// for it. A newly-issued cookie is aliased to the tab bootstrap nonce's
/// deterministic fallback, so concurrent first-load beacons converge even
/// before either response has installed its cookie.
pub async fn resolve_visitor(
    executor: &mut dyn Executor,
    token_hash: &str,
    bootstrap_id: Option<&str>,
    now: i64,
) -> anyhow::Result<String> {
    let candidate = bootstrap_id.map_or_else(
        || token_hash.to_string(),
        |bootstrap_id| hash_identifier("analytics-bootstrap-visitor", bootstrap_id),
    );
    let rows = toasty::sql::query(
        "INSERT INTO analytics_visitor_aliases AS existing
             (token_hash, visitor_id, created_at)
         VALUES ($1, $2, $3)
         ON CONFLICT (token_hash) DO UPDATE
         SET visitor_id = existing.visitor_id
         RETURNING visitor_id",
    )
    .bind(token_hash.to_string())
    .bind(candidate)
    .bind(now)
    .exec(&mut *executor)
    .await
    .context("analytics visitor resolution failed")?;
    one_text(&rows, "analytics visitor resolution")
}

/// Insert an event under a server-owned, rolling thirty-minute session.
///
/// Pageviews and outbound clicks insert exactly once. Engagement uses one
/// stable event id per document; later lifecycle flushes atomically raise its
/// cumulative measurements instead of creating duplicate samples.
pub async fn insert_event(
    executor: &mut dyn Executor,
    visitor_hash: &str,
    event: ValidatedEvent,
    occurred_at: i64,
) -> anyhow::Result<bool> {
    if AnalyticsEvent::filter_by_id(event.id.clone())
        .first()
        .exec(&mut *executor)
        .await?
        .is_some()
    {
        update_engagement(executor, visitor_hash, &event, occurred_at).await?;
        return Ok(false);
    }

    let session_id = server_session(executor, visitor_hash, occurred_at).await?;
    let engagement_retry = (event.kind == "engagement").then(|| EngagementUpdate {
        id: event.id.clone(),
        page_path: event.path.clone(),
        engagement_seconds: event.engagement_seconds,
        scroll_percent: event.scroll_percent,
        lcp_milliseconds: event.lcp_milliseconds,
        cls_thousandths: event.cls_thousandths,
        navigation_milliseconds: event.navigation_milliseconds,
    });
    let insert = toasty::create!(AnalyticsEvent {
        id: event.id.clone(),
        visitor_id: visitor_hash,
        session_id,
        occurred_at,
        kind: event.kind,
        page_path: event.path,
        referrer_kind: event.referrer_kind,
        referrer_host: event.referrer_host,
        referrer_path: event.referrer_path,
        country_code: event.country_code,
        timezone: event.timezone,
        language: event.language,
        device_kind: event.device_kind,
        browser: event.browser,
        operating_system: event.operating_system,
        viewport_kind: event.viewport_kind,
        navigation_kind: event.navigation_kind,
        local_hour: event.local_hour,
        local_weekday: event.local_weekday,
        engagement_seconds: event.engagement_seconds,
        scroll_percent: event.scroll_percent,
        lcp_milliseconds: event.lcp_milliseconds,
        cls_thousandths: event.cls_thousandths,
        navigation_milliseconds: event.navigation_milliseconds,
        target_host: event.target_host,
        utm_source: event.utm_source,
        utm_medium: event.utm_medium,
        utm_campaign: event.utm_campaign,
    })
    .exec(&mut *executor)
    .await;
    match insert {
        Ok(_) => Ok(true),
        Err(error) => {
            let exists = AnalyticsEvent::filter_by_id(event.id)
                .first()
                .exec(&mut *executor)
                .await?
                .is_some();
            if !exists {
                return Err(error.into());
            }
            if let Some(update) = engagement_retry {
                update_engagement_values(executor, visitor_hash, &update, occurred_at).await?;
            }
            Ok(false)
        }
    }
}

const SESSION_IDLE_SECONDS: i64 = 30 * 60;

async fn server_session(
    executor: &mut dyn Executor,
    visitor_hash: &str,
    occurred_at: i64,
) -> anyhow::Result<String> {
    let candidate = hash_identifier("analytics-session", &Uuid::new_v4().to_string());
    let rows = toasty::sql::query(
        "INSERT INTO analytics_sessions (visitor_id, session_id, last_seen_at)
         VALUES ($1, $2, $3)
         ON CONFLICT (visitor_id) DO UPDATE
         SET session_id = CASE
                 WHEN analytics_sessions.last_seen_at
                      < EXCLUDED.last_seen_at - $4
                 THEN EXCLUDED.session_id
                 ELSE analytics_sessions.session_id
             END,
             last_seen_at = GREATEST(
                 analytics_sessions.last_seen_at,
                 EXCLUDED.last_seen_at
             )
         RETURNING session_id",
    )
    .bind(visitor_hash.to_string())
    .bind(candidate)
    .bind(occurred_at)
    .bind(SESSION_IDLE_SECONDS)
    .exec(&mut *executor)
    .await
    .context("analytics session upsert failed")?;
    one_text(&rows, "analytics session upsert")
}

struct EngagementUpdate {
    id: String,
    page_path: String,
    engagement_seconds: Option<i64>,
    scroll_percent: Option<i64>,
    lcp_milliseconds: Option<i64>,
    cls_thousandths: Option<i64>,
    navigation_milliseconds: Option<i64>,
}

async fn update_engagement(
    executor: &mut dyn Executor,
    visitor_hash: &str,
    event: &ValidatedEvent,
    occurred_at: i64,
) -> anyhow::Result<()> {
    if event.kind != "engagement" {
        return Ok(());
    }
    update_engagement_values(
        executor,
        visitor_hash,
        &EngagementUpdate {
            id: event.id.clone(),
            page_path: event.path.clone(),
            engagement_seconds: event.engagement_seconds,
            scroll_percent: event.scroll_percent,
            lcp_milliseconds: event.lcp_milliseconds,
            cls_thousandths: event.cls_thousandths,
            navigation_milliseconds: event.navigation_milliseconds,
        },
        occurred_at,
    )
    .await
}

async fn update_engagement_values(
    executor: &mut dyn Executor,
    visitor_hash: &str,
    update: &EngagementUpdate,
    occurred_at: i64,
) -> anyhow::Result<()> {
    // Keep the cumulative maximum and its activity cursor in one statement:
    // a retry after a process/database failure must never observe one without
    // the other. The final idle predicate also prevents an old document from
    // resurrecting a session that already expired or rotated.
    toasty::sql::statement(
        "WITH advanced AS (
             UPDATE analytics_events
             SET engagement_seconds = GREATEST(
                     COALESCE(engagement_seconds, 0),
                     $4
                 ),
                 scroll_percent = CASE WHEN $5 < 0 THEN scroll_percent
                     ELSE GREATEST(COALESCE(scroll_percent, 0), $5) END,
                 lcp_milliseconds = CASE WHEN $6 < 0 THEN lcp_milliseconds
                     ELSE GREATEST(COALESCE(lcp_milliseconds, 0), $6) END,
                 cls_thousandths = CASE WHEN $7 < 0 THEN cls_thousandths
                     ELSE GREATEST(COALESCE(cls_thousandths, 0), $7) END,
                 navigation_milliseconds = CASE
                     WHEN $8 < 0 THEN navigation_milliseconds
                     ELSE GREATEST(COALESCE(navigation_milliseconds, 0), $8)
                 END
             WHERE id = $1 AND visitor_id = $2
               AND kind = 'engagement' AND page_path = $3
               AND (
                   engagement_seconds IS NULL OR engagement_seconds < $4
                   OR ($5 >= 0 AND (scroll_percent IS NULL OR scroll_percent < $5))
                   OR ($6 >= 0 AND (
                       lcp_milliseconds IS NULL OR lcp_milliseconds < $6
                   ))
                   OR ($7 >= 0 AND (
                       cls_thousandths IS NULL OR cls_thousandths < $7
                   ))
                   OR ($8 >= 0 AND (
                       navigation_milliseconds IS NULL
                       OR navigation_milliseconds < $8
                   ))
               )
             RETURNING session_id
         )
         UPDATE analytics_sessions AS current_session
         SET last_seen_at = GREATEST(current_session.last_seen_at, $9)
         FROM advanced
         WHERE current_session.visitor_id = $2
           AND current_session.session_id = advanced.session_id
           AND current_session.last_seen_at >= $9 - $10",
    )
    .bind(update.id.clone())
    .bind(visitor_hash.to_string())
    .bind(update.page_path.clone())
    .bind(update.engagement_seconds.unwrap_or(0))
    .bind(update.scroll_percent.unwrap_or(-1))
    .bind(update.lcp_milliseconds.unwrap_or(-1))
    .bind(update.cls_thousandths.unwrap_or(-1))
    .bind(update.navigation_milliseconds.unwrap_or(-1))
    .bind(occurred_at)
    .bind(SESSION_IDLE_SECONDS)
    .exec(&mut *executor)
    .await
    .context("analytics engagement update failed")?;
    Ok(())
}

fn one_text(rows: &[toasty::stmt::Value], operation: &str) -> anyhow::Result<String> {
    rows.first()
        .and_then(toasty::stmt::Value::as_record)
        .and_then(|row| row.first())
        .and_then(toasty::stmt::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("{operation} returned no text row"))
}

/// The private ledger is one row per visitor. A later submission edits the
/// visitor's entry without disclosing whether one already existed.
pub async fn upsert_identity(
    executor: &mut dyn Executor,
    visitor_hash: &str,
    display_name: String,
    note: Option<String>,
    now: i64,
) -> toasty::Result<()> {
    let visitor_id = visitor_hash.to_string();
    match AnalyticsIdentity::filter_by_visitor_id(visitor_id.clone())
        .first()
        .exec(&mut *executor)
        .await?
    {
        Some(mut identity) => {
            let updated_at = now.max(identity.updated_at);
            toasty::update!(identity {
                display_name: display_name,
                note: note,
                updated_at: updated_at,
            })
            .exec(&mut *executor)
            .await?;
        }
        None => {
            let create = toasty::create!(AnalyticsIdentity {
                visitor_id: visitor_id.clone(),
                display_name: display_name.clone(),
                note: note.clone(),
                first_submitted_at: now,
                updated_at: now,
            })
            .exec(&mut *executor)
            .await;
            if let Err(error) = create {
                // A double-submit can race on the key. If the row appeared,
                // finish as an update; otherwise preserve the original error.
                let Some(mut identity) = AnalyticsIdentity::filter_by_visitor_id(visitor_id)
                    .first()
                    .exec(&mut *executor)
                    .await?
                else {
                    return Err(error);
                };
                let updated_at = now.max(identity.updated_at);
                toasty::update!(identity {
                    display_name: display_name,
                    note: note,
                    updated_at: updated_at,
                })
                .exec(&mut *executor)
                .await?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_session_identifiers_are_one_way_and_namespaced() {
        let raw = "64ec3b75-05af-49de-8d4e-75c2bd4ee4d4";
        let session = hash_identifier("analytics-session", raw);
        assert_eq!(session.len(), 64);
        assert!(!session.contains(raw));
        assert_ne!(session, hash_identifier("analytics-bootstrap-visitor", raw));
    }
}
