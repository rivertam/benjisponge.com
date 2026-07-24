//! Public aggregate queries.
//!
//! Every query is fixed SQL with one bound cutoff. In particular, this module
//! never touches the private identity ledger; that table has no public read
//! path.

use anyhow::{Context, anyhow};
use toasty::{Db, Executor, stmt::Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Window {
    Week,
    Month,
    Quarter,
    Year,
}

impl Window {
    pub const ALL: [Self; 4] = [Self::Week, Self::Month, Self::Quarter, Self::Year];

    #[cfg(test)]
    pub fn parse(value: Option<&str>) -> Self {
        match value {
            Some("7d") => Self::Week,
            Some("90d") => Self::Quarter,
            Some("365d") => Self::Year,
            _ => Self::Month,
        }
    }

    pub const fn slug(self) -> &'static str {
        match self {
            Self::Week => "7d",
            Self::Month => "30d",
            Self::Quarter => "90d",
            Self::Year => "365d",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Week => "7 days",
            Self::Month => "30 days",
            Self::Quarter => "90 days",
            Self::Year => "1 year",
        }
    }

    pub const fn days(self) -> i64 {
        match self {
            Self::Week => 7,
            Self::Month => 30,
            Self::Quarter => 90,
            Self::Year => 365,
        }
    }
}

#[derive(Debug, Default)]
pub struct Dashboard {
    pub overview: Overview,
    pub performance: Performance,
    pub days: Vec<Day>,
    pub pages: Vec<Page>,
    pub channels: Vec<Count>,
    pub referrers: Vec<Cohort>,
    pub countries: Vec<Cohort>,
    pub technology: Vec<Technology>,
    pub hourly: [[i64; 24]; 7],
    pub journeys: Vec<Journey>,
    pub outbound: Vec<Cohort>,
    pub campaigns: Vec<Campaign>,
}

#[derive(Debug, Default)]
pub struct Overview {
    pub pageviews: i64,
    pub visitors: i64,
    pub sessions: i64,
    pub engaged_seconds: i64,
    pub outbound_clicks: i64,
    pub returning_percent: i64,
    pub single_page_percent: i64,
}

#[derive(Debug, Default)]
pub struct Performance {
    pub attention_seconds: i64,
    pub scroll_percent: i64,
    pub finish_percent: i64,
    pub lcp_milliseconds: i64,
    pub cls_thousandths: i64,
    pub navigation_milliseconds: i64,
    pub samples: i64,
}

#[derive(Debug)]
pub struct Day {
    pub date: String,
    pub views: i64,
    pub visitors: i64,
    pub engaged_seconds: i64,
}

#[derive(Debug)]
pub struct Page {
    pub path: String,
    pub views: i64,
    pub visitors: i64,
    pub engaged_seconds: i64,
    pub scroll_percent: i64,
}

#[derive(Debug)]
pub struct Count {
    pub label: String,
    pub count: i64,
}

#[derive(Debug)]
pub struct Cohort {
    pub label: String,
    pub views: i64,
    pub visitors: i64,
}

#[derive(Debug)]
pub struct Technology {
    pub dimension: String,
    pub label: String,
    pub views: i64,
    pub visitors: i64,
}

#[derive(Debug)]
pub struct Journey {
    pub from: String,
    pub to: String,
    pub trips: i64,
    pub visitors: i64,
}

#[derive(Debug)]
pub struct Campaign {
    pub source: String,
    pub campaign: String,
    pub views: i64,
    pub visitors: i64,
}

pub async fn load(db: &Db, cutoff: i64) -> anyhow::Result<Dashboard> {
    // One repeatable-read transaction gives the entire public page a coherent
    // snapshot while checking out (and pre-pinging) only one pooled
    // connection. A slow query degrades to the dashboard's standby state.
    let mut handle = db.clone();
    let mut transaction = handle
        .transaction()
        .await
        .context("analytics snapshot transaction failed")?;
    toasty::sql::statement("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
        .exec(&mut transaction)
        .await
        .context("analytics snapshot configuration failed")?;
    toasty::sql::statement("SET LOCAL statement_timeout = '3s'")
        .exec(&mut transaction)
        .await
        .context("analytics statement timeout failed")?;

    let overview = load_overview(&mut transaction, cutoff).await?;
    let performance = load_performance(&mut transaction, cutoff).await?;
    let days = load_days(&mut transaction, cutoff).await?;
    let pages = load_pages(&mut transaction, cutoff).await?;
    let channels = load_counts(
        &mut transaction,
        cutoff,
        "WITH arrivals AS (
             SELECT event.*
             FROM analytics_events event
             WHERE event.occurred_at >= $1 AND event.kind = 'pageview'
               AND NOT EXISTS (
                   SELECT 1
                   FROM analytics_events earlier
                   WHERE earlier.kind = 'pageview'
                     AND earlier.visitor_id = event.visitor_id
                     AND earlier.session_id = event.session_id
                     AND (earlier.occurred_at, earlier.id)
                         < (event.occurred_at, event.id)
               )
         )
         SELECT referrer_kind, COUNT(*)::bigint
         FROM arrivals
         GROUP BY referrer_kind
         ORDER BY COUNT(*) DESC, referrer_kind",
    )
    .await?;
    let referrers = load_cohorts(
        &mut transaction,
        cutoff,
        "WITH arrivals AS (
             SELECT event.*
             FROM analytics_events event
             WHERE event.occurred_at >= $1 AND event.kind = 'pageview'
               AND NOT EXISTS (
                   SELECT 1
                   FROM analytics_events earlier
                   WHERE earlier.kind = 'pageview'
                     AND earlier.visitor_id = event.visitor_id
                     AND earlier.session_id = event.session_id
                     AND (earlier.occurred_at, earlier.id)
                         < (event.occurred_at, event.id)
               )
         )
         SELECT referrer_host, COUNT(*)::bigint,
                COUNT(DISTINCT visitor_id)::bigint
         FROM arrivals
         WHERE referrer_host IS NOT NULL
         GROUP BY referrer_host
         HAVING COUNT(DISTINCT visitor_id) >= 3
         ORDER BY COUNT(*) DESC, referrer_host
         LIMIT 12",
    )
    .await?;
    let countries = load_cohorts(
        &mut transaction,
        cutoff,
        "SELECT country_code, COUNT(*)::bigint,
                COUNT(DISTINCT visitor_id)::bigint
         FROM analytics_events
         WHERE occurred_at >= $1 AND kind = 'pageview'
           AND country_code IS NOT NULL
         GROUP BY country_code
         HAVING COUNT(DISTINCT visitor_id) >= 3
         ORDER BY COUNT(*) DESC, country_code
         LIMIT 30",
    )
    .await?;
    let technology = load_technology(&mut transaction, cutoff).await?;
    let hourly = load_hourly(&mut transaction, cutoff).await?;
    let journeys = load_journeys(&mut transaction, cutoff).await?;
    let outbound = load_cohorts(
        &mut transaction,
        cutoff,
        "SELECT target_host, COUNT(*)::bigint,
                COUNT(DISTINCT visitor_id)::bigint
         FROM analytics_events
         WHERE occurred_at >= $1 AND kind = 'outbound'
           AND target_host IS NOT NULL
         GROUP BY target_host
         HAVING COUNT(DISTINCT visitor_id) >= 3
         ORDER BY COUNT(*) DESC, target_host
         LIMIT 10",
    )
    .await?;
    let campaigns = load_campaigns(&mut transaction, cutoff).await?;

    let dashboard = Dashboard {
        overview,
        performance,
        days,
        pages,
        channels,
        referrers,
        countries,
        technology,
        hourly,
        journeys,
        outbound,
        campaigns,
    };
    transaction
        .commit()
        .await
        .context("analytics snapshot commit failed")?;
    Ok(dashboard)
}

async fn query(
    executor: &mut dyn Executor,
    sql: &'static str,
    cutoff: i64,
) -> anyhow::Result<Vec<Value>> {
    toasty::sql::query(sql)
        .bind(cutoff)
        .exec(executor)
        .await
        .context("analytics aggregate query failed")
}

async fn load_overview(executor: &mut dyn Executor, cutoff: i64) -> anyhow::Result<Overview> {
    let rows = query(
        executor,
        "WITH filtered AS (
             SELECT visitor_id, session_id, kind, engagement_seconds
             FROM analytics_events
             WHERE occurred_at >= $1
         ),
         visitors AS (
             SELECT visitor_id, COUNT(DISTINCT session_id) AS sessions
             FROM filtered WHERE kind = 'pageview' GROUP BY visitor_id
         ),
         sessions AS (
             SELECT visitor_id, session_id, COUNT(*) AS views
             FROM filtered
             WHERE kind = 'pageview'
             GROUP BY visitor_id, session_id
         )
         SELECT
             COUNT(*) FILTER (WHERE kind = 'pageview')::bigint,
             COUNT(DISTINCT visitor_id) FILTER (WHERE kind = 'pageview')::bigint,
             COUNT(DISTINCT (visitor_id, session_id))
                 FILTER (WHERE kind = 'pageview')::bigint,
             COALESCE(SUM(engagement_seconds)
                 FILTER (WHERE kind = 'engagement'), 0)::bigint,
             COUNT(*) FILTER (WHERE kind = 'outbound')::bigint,
             COALESCE((SELECT ROUND(
                 100.0 * COUNT(*) FILTER (WHERE sessions > 1)
                 / NULLIF(COUNT(*), 0)
             )::bigint FROM visitors), 0)::bigint,
             COALESCE((SELECT ROUND(
                 100.0 * COUNT(*) FILTER (WHERE views = 1) / NULLIF(COUNT(*), 0)
             )::bigint FROM sessions), 0)::bigint
         FROM filtered",
        cutoff,
    )
    .await?;
    let row = only_row(&rows, 7)?;
    Ok(Overview {
        pageviews: integer(&row[0])?,
        visitors: integer(&row[1])?,
        sessions: integer(&row[2])?,
        engaged_seconds: integer(&row[3])?,
        outbound_clicks: integer(&row[4])?,
        returning_percent: integer(&row[5])?,
        single_page_percent: integer(&row[6])?,
    })
}

async fn load_performance(executor: &mut dyn Executor, cutoff: i64) -> anyhow::Result<Performance> {
    let rows = query(
        executor,
        "SELECT
             COALESCE(ROUND(AVG(engagement_seconds))::bigint, 0),
             COALESCE(ROUND(AVG(scroll_percent))::bigint, 0),
             COALESCE(ROUND(
                 100.0 * COUNT(*) FILTER (WHERE scroll_percent >= 90)
                 / NULLIF(COUNT(*) FILTER (WHERE scroll_percent IS NOT NULL), 0)
             )::bigint, 0),
             COALESCE(ROUND(AVG(lcp_milliseconds))::bigint, 0),
             COALESCE(ROUND(AVG(cls_thousandths))::bigint, 0),
             COALESCE(ROUND(AVG(navigation_milliseconds))::bigint, 0),
             COUNT(*)::bigint
         FROM analytics_events
         WHERE occurred_at >= $1 AND kind = 'engagement'",
        cutoff,
    )
    .await?;
    let row = only_row(&rows, 7)?;
    Ok(Performance {
        attention_seconds: integer(&row[0])?,
        scroll_percent: integer(&row[1])?,
        finish_percent: integer(&row[2])?,
        lcp_milliseconds: integer(&row[3])?,
        cls_thousandths: integer(&row[4])?,
        navigation_milliseconds: integer(&row[5])?,
        samples: integer(&row[6])?,
    })
}

async fn load_days(executor: &mut dyn Executor, cutoff: i64) -> anyhow::Result<Vec<Day>> {
    let rows = query(
        executor,
        "WITH daily AS (
             SELECT (to_timestamp(occurred_at) AT TIME ZONE 'UTC')::date AS day,
                    COUNT(*) FILTER (WHERE kind = 'pageview')::bigint AS views,
                    COUNT(DISTINCT visitor_id)
                        FILTER (WHERE kind = 'pageview')::bigint AS visitors,
                    COALESCE(SUM(engagement_seconds)
                        FILTER (WHERE kind = 'engagement'), 0)::bigint AS engaged
             FROM analytics_events
             WHERE occurred_at >= $1
             GROUP BY 1
         )
         SELECT to_char(series.day, 'YYYY-MM-DD'),
                COALESCE(daily.views, 0)::bigint,
                COALESCE(daily.visitors, 0)::bigint,
                COALESCE(daily.engaged, 0)::bigint
         FROM generate_series(
             (to_timestamp($1) AT TIME ZONE 'UTC')::date,
             (CURRENT_TIMESTAMP AT TIME ZONE 'UTC')::date,
             interval '1 day'
         ) AS series(day)
         LEFT JOIN daily ON daily.day = series.day
         ORDER BY series.day",
        cutoff,
    )
    .await?;
    rows.iter()
        .map(|value| {
            let row = row(value, 4)?;
            Ok(Day {
                date: text(&row[0])?.to_string(),
                views: integer(&row[1])?,
                visitors: integer(&row[2])?,
                engaged_seconds: integer(&row[3])?,
            })
        })
        .collect()
}

async fn load_pages(executor: &mut dyn Executor, cutoff: i64) -> anyhow::Result<Vec<Page>> {
    let rows = query(
        executor,
        "SELECT page_path,
                COUNT(*) FILTER (WHERE kind = 'pageview')::bigint AS views,
                COUNT(DISTINCT visitor_id)
                    FILTER (WHERE kind = 'pageview')::bigint AS visitors,
                COALESCE(SUM(engagement_seconds)
                    FILTER (WHERE kind = 'engagement'), 0)::bigint,
                COALESCE(ROUND(AVG(scroll_percent)
                    FILTER (WHERE kind = 'engagement'))::bigint, 0)
         FROM analytics_events
         WHERE occurred_at >= $1
         GROUP BY page_path
         HAVING COUNT(*) FILTER (WHERE kind = 'pageview') > 0
         ORDER BY views DESC, page_path",
        cutoff,
    )
    .await?;
    let mut pages: Vec<Page> = rows
        .iter()
        .map(|value| {
            let row = row(value, 5)?;
            Ok(Page {
                path: text(&row[0])?.to_string(),
                views: integer(&row[1])?,
                visitors: integer(&row[2])?,
                engaged_seconds: integer(&row[3])?,
                scroll_percent: integer(&row[4])?,
            })
        })
        .collect::<anyhow::Result<_>>()?;
    let fixed_routes = crate::content::routes::site_routes();
    pages.retain(|page| fixed_routes.iter().any(|route| route == &page.path) || page.visitors >= 3);
    pages.truncate(12);
    Ok(pages)
}

async fn load_counts(
    executor: &mut dyn Executor,
    cutoff: i64,
    sql: &'static str,
) -> anyhow::Result<Vec<Count>> {
    let rows = query(executor, sql, cutoff).await?;
    rows.iter()
        .map(|value| {
            let row = row(value, 2)?;
            Ok(Count {
                label: text(&row[0])?.to_string(),
                count: integer(&row[1])?,
            })
        })
        .collect()
}

async fn load_cohorts(
    executor: &mut dyn Executor,
    cutoff: i64,
    sql: &'static str,
) -> anyhow::Result<Vec<Cohort>> {
    let rows = query(executor, sql, cutoff).await?;
    rows.iter()
        .map(|value| {
            let row = row(value, 3)?;
            Ok(Cohort {
                label: text(&row[0])?.to_string(),
                views: integer(&row[1])?,
                visitors: integer(&row[2])?,
            })
        })
        .collect()
}

async fn load_technology(
    executor: &mut dyn Executor,
    cutoff: i64,
) -> anyhow::Result<Vec<Technology>> {
    let rows = query(
        executor,
        "SELECT dimension, label, views, visitors
         FROM (
             SELECT 'device'::text AS dimension, device_kind::text AS label,
                    COUNT(*)::bigint AS views,
                    COUNT(DISTINCT visitor_id)::bigint AS visitors
             FROM analytics_events
             WHERE occurred_at >= $1 AND kind = 'pageview'
             GROUP BY device_kind
             UNION ALL
             SELECT 'browser', browser, COUNT(*)::bigint,
                    COUNT(DISTINCT visitor_id)::bigint
             FROM analytics_events
             WHERE occurred_at >= $1 AND kind = 'pageview'
             GROUP BY browser
             UNION ALL
             SELECT 'os', operating_system, COUNT(*)::bigint,
                    COUNT(DISTINCT visitor_id)::bigint
             FROM analytics_events
             WHERE occurred_at >= $1 AND kind = 'pageview'
             GROUP BY operating_system
         ) technology
         WHERE visitors >= 3
         ORDER BY dimension, views DESC, label",
        cutoff,
    )
    .await?;
    rows.iter()
        .map(|value| {
            let row = row(value, 4)?;
            Ok(Technology {
                dimension: text(&row[0])?.to_string(),
                label: text(&row[1])?.to_string(),
                views: integer(&row[2])?,
                visitors: integer(&row[3])?,
            })
        })
        .collect()
}

async fn load_hourly(executor: &mut dyn Executor, cutoff: i64) -> anyhow::Result<[[i64; 24]; 7]> {
    let rows = query(
        executor,
        "SELECT local_weekday::bigint, local_hour::bigint, COUNT(*)::bigint
         FROM analytics_events
         WHERE occurred_at >= $1 AND kind = 'pageview'
           AND local_weekday IS NOT NULL AND local_hour IS NOT NULL
         GROUP BY local_weekday, local_hour
         ORDER BY local_weekday, local_hour",
        cutoff,
    )
    .await?;
    let mut grid = [[0; 24]; 7];
    for value in &rows {
        let row = row(value, 3)?;
        let weekday = usize::try_from(integer(&row[0])?)?;
        let hour = usize::try_from(integer(&row[1])?)?;
        if let Some(cell) = grid.get_mut(weekday).and_then(|day| day.get_mut(hour)) {
            *cell = integer(&row[2])?;
        }
    }
    Ok(grid)
}

async fn load_journeys(executor: &mut dyn Executor, cutoff: i64) -> anyhow::Result<Vec<Journey>> {
    let rows = query(
        executor,
        "SELECT referrer_path, page_path, COUNT(*)::bigint,
                COUNT(DISTINCT visitor_id)::bigint
         FROM analytics_events
         WHERE occurred_at >= $1 AND kind = 'pageview'
           AND referrer_kind = 'internal' AND referrer_path IS NOT NULL
           AND referrer_path <> page_path
         GROUP BY referrer_path, page_path
         HAVING COUNT(DISTINCT visitor_id) >= 3
         ORDER BY COUNT(*) DESC, referrer_path, page_path
         LIMIT 10",
        cutoff,
    )
    .await?;
    rows.iter()
        .map(|value| {
            let row = row(value, 4)?;
            Ok(Journey {
                from: text(&row[0])?.to_string(),
                to: text(&row[1])?.to_string(),
                trips: integer(&row[2])?,
                visitors: integer(&row[3])?,
            })
        })
        .collect()
}

async fn load_campaigns(executor: &mut dyn Executor, cutoff: i64) -> anyhow::Result<Vec<Campaign>> {
    let rows = query(
        executor,
        "WITH arrivals AS (
             SELECT event.*
             FROM analytics_events event
             WHERE event.occurred_at >= $1 AND event.kind = 'pageview'
               AND NOT EXISTS (
                   SELECT 1
                   FROM analytics_events earlier
                   WHERE earlier.kind = 'pageview'
                     AND earlier.visitor_id = event.visitor_id
                     AND earlier.session_id = event.session_id
                     AND (earlier.occurred_at, earlier.id)
                         < (event.occurred_at, event.id)
               )
         )
         SELECT utm_source, COALESCE(utm_campaign, '(uncategorized)'),
                COUNT(*)::bigint, COUNT(DISTINCT visitor_id)::bigint
         FROM arrivals
         WHERE utm_source IS NOT NULL
         GROUP BY utm_source, COALESCE(utm_campaign, '(uncategorized)')
         HAVING COUNT(DISTINCT visitor_id) >= 3
         ORDER BY COUNT(*) DESC, utm_source,
                  COALESCE(utm_campaign, '(uncategorized)')
         LIMIT 10",
        cutoff,
    )
    .await?;
    rows.iter()
        .map(|value| {
            let row = row(value, 4)?;
            Ok(Campaign {
                source: text(&row[0])?.to_string(),
                campaign: text(&row[1])?.to_string(),
                views: integer(&row[2])?,
                visitors: integer(&row[3])?,
            })
        })
        .collect()
}

fn only_row(rows: &[Value], fields: usize) -> anyhow::Result<&[Value]> {
    if rows.len() != 1 {
        return Err(anyhow!(
            "analytics query expected one row, received {}",
            rows.len()
        ));
    }
    row(&rows[0], fields)
}

fn row(value: &Value, fields: usize) -> anyhow::Result<&[Value]> {
    let record = value
        .as_record()
        .ok_or_else(|| anyhow!("analytics query returned a non-record row"))?;
    if record.len() != fields {
        return Err(anyhow!(
            "analytics query expected {fields} fields, received {}",
            record.len()
        ));
    }
    Ok(record.as_slice())
}

fn text(value: &Value) -> anyhow::Result<&str> {
    value
        .as_str()
        .ok_or_else(|| anyhow!("analytics query expected text, received {value:?}"))
}

fn integer(value: &Value) -> anyhow::Result<i64> {
    match value {
        Value::I64(value) => Ok(*value),
        Value::I32(value) => Ok(i64::from(*value)),
        Value::I16(value) => Ok(i64::from(*value)),
        Value::I8(value) => Ok(i64::from(*value)),
        Value::U64(value) => i64::try_from(*value).context("analytics integer overflow"),
        Value::U32(value) => Ok(i64::from(*value)),
        value => Err(anyhow!(
            "analytics query expected an integer, received {value:?}"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_parser_is_bounded_to_known_ranges() {
        assert_eq!(Window::parse(Some("7d")), Window::Week);
        assert_eq!(Window::parse(Some("365d")), Window::Year);
        assert_eq!(Window::parse(Some("all")), Window::Month);
        assert_eq!(Window::parse(None), Window::Month);
    }

    #[test]
    fn public_queries_do_not_name_the_private_table() {
        let source = include_str!("dashboard.rs");
        let private_table = ["analytics", "_identities"].concat();
        assert!(
            !source.contains(&private_table),
            "public dashboard source must not access private identities"
        );
    }

    #[test]
    fn public_cohorts_require_three_visitors() {
        let source = include_str!("dashboard.rs");
        assert!(
            source
                .matches("HAVING COUNT(DISTINCT visitor_id) >= 3")
                .count()
                >= 4
        );
    }
}
