//! RSS 2.0 feed at `/feed.xml`, generated from the logbook registry — every
//! entry, long or short, becomes an `<item>`, so publishing to the log
//! publishes to the feed. Slay the Spire 2 victories (and only victories —
//! deaths stay on `/spire`) join the feed at render time from the synced run
//! database. Not a page: it renders no shell and stays out of
//! `site_routes()` (the 404 index is for pages).

use topcoat::{
    Result,
    context::Cx,
    router::{headers, route},
};

use crate::content::logbook::{Entry, LOG};
use crate::content::spire_runs::{self, Run, SPIRE_CACHE_REFRESH_HEADER, fmt_duration};

/// Where absolute links point. `SITE_ORIGIN` overrides the default at
/// runtime; the default is a placeholder until the real domain is wired up.
fn origin() -> String {
    std::env::var("SITE_ORIGIN").unwrap_or_else(|_| "https://benjisponge.com".to_string())
}

#[route(GET "/feed.xml")]
async fn feed(cx: &Cx) -> Result<([(&'static str, &'static str); 1], String)> {
    let log = spire_runs::load(headers(cx).contains_key(SPIRE_CACHE_REFRESH_HEADER)).await;
    Ok((
        [("Content-Type", "application/rss+xml; charset=utf-8")],
        rss_xml(&origin(), &log.runs),
    ))
}

/// One feed item, from either source, ready to sort and emit.
struct FeedItem {
    date: String,
    /// Curated logbook entries outrank runs on the same date.
    curated: bool,
    /// Tie-break among runs sharing a date; 0 for logbook entries.
    start_time: i64,
    title: String,
    link: String,
    description: String,
    guid: String,
}

/// The whole feed as a string. Pure — origin and runs in, XML out. Losses
/// and abandoned runs are filtered here so callers can pass the full log.
pub fn rss_xml(origin: &str, runs: &[Run]) -> String {
    let mut items: Vec<FeedItem> = Vec::new();

    for (index, entry) in LOG.iter().enumerate() {
        let (title, link, description) = match entry {
            Entry::Essay {
                title,
                teaser,
                slug,
                ..
            } => (
                (*title).to_string(),
                format!("{origin}/thoughts/{slug}"),
                (*teaser).to_string(),
            ),
            Entry::Note { body, slug, .. } => (
                truncate(body, 80),
                format!("{origin}/thoughts/{slug}"),
                (*body).to_string(),
            ),
            Entry::Update {
                stamp,
                label,
                body,
                href,
                ..
            } => {
                let link = if href.starts_with('/') {
                    format!("{origin}{href}")
                } else {
                    (*href).to_string()
                };
                (
                    format!("[{stamp}] {label} — {body}"),
                    link,
                    (*body).to_string(),
                )
            }
        };
        items.push(FeedItem {
            date: entry.date().to_string(),
            curated: true,
            start_time: 0,
            title,
            link,
            description,
            guid: format!("{}/log/{:04}", origin, LOG.len() - index),
        });
    }

    for run in runs.iter().filter(|r| r.win && !r.abandoned) {
        items.push(FeedItem {
            date: run.date.clone(),
            curated: false,
            start_time: run.start_time,
            title: format!(
                "[win] spire — {}, Ascension {}",
                run.character, run.ascension
            ),
            link: format!("{origin}/spire"),
            description: format!(
                "{} victory at Ascension {} — {} floors in {}.",
                run.character,
                run.ascension,
                run.floors,
                fmt_duration(run.run_time)
            ),
            guid: format!("{origin}/spire/run/{}", run.id),
        });
    }

    // Newest first; a curated entry leads the runs on its date.
    items.sort_by(|a, b| {
        b.date
            .cmp(&a.date)
            .then_with(|| b.curated.cmp(&a.curated))
            .then_with(|| b.start_time.cmp(&a.start_time))
    });

    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\">\n");
    xml.push_str("<channel>\n");
    xml.push_str("<title>Ben Berman — logbook</title>\n");
    xml.push_str(&format!("<link>{}/</link>\n", escape(origin)));
    xml.push_str("<description>Everything gets an entry, long or short.</description>\n");
    xml.push_str("<language>en-us</language>\n");
    xml.push_str(&format!(
        "<atom:link href=\"{}/feed.xml\" rel=\"self\" type=\"application/rss+xml\"/>\n",
        escape(origin)
    ));
    for item in &items {
        xml.push_str("<item>\n");
        xml.push_str(&format!("<title>{}</title>\n", escape(&item.title)));
        xml.push_str(&format!("<link>{}</link>\n", escape(&item.link)));
        xml.push_str(&format!(
            "<description>{}</description>\n",
            escape(&item.description)
        ));
        xml.push_str(&format!(
            "<guid isPermaLink=\"false\">{}</guid>\n",
            escape(&item.guid)
        ));
        xml.push_str(&format!("<pubDate>{}</pubDate>\n", rfc2822(&item.date)));
        xml.push_str("</item>\n");
    }
    xml.push_str("</channel>\n</rss>\n");
    xml
}

/// XML-escape everything interpolated into the feed.
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

/// Cut to at most `max` chars (a char boundary by construction) with an
/// ellipsis when anything was dropped.
fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.to_string()
    } else {
        let cut: String = text.chars().take(max).collect();
        format!("{}…", cut.trim_end())
    }
}

/// `YYYY-MM-DD` → RFC 2822 at midnight UTC, e.g. `Sun, 12 Jul 2026 00:00:00
/// +0000`. Weekday via Sakamoto's method — no date crate in the tree. Inputs
/// are shape-checked upstream (logbook tests; run dates filtered on parse).
fn rfc2822(iso: &str) -> String {
    let year: i32 = iso[0..4].parse().expect("feed date year");
    let month: usize = iso[5..7].parse().expect("feed date month");
    let day: u32 = iso[8..10].parse().expect("feed date day");

    const OFFSETS: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    let y = if month < 3 { year - 1 } else { year };
    let weekday_index =
        (y + y / 4 - y / 100 + y / 400 + OFFSETS[month - 1] + day as i32).rem_euclid(7);

    format!(
        "{}, {:02} {} {} 00:00:00 +0000",
        WEEKDAYS[weekday_index as usize],
        day,
        MONTHS[month - 1],
        year
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const ORIGIN: &str = "https://example.test";

    fn run(id: &str, date: &str, win: bool) -> Run {
        serde_json::from_str(&format!(
            r#"{{"id": "{id}", "date": "{date}", "start_time": {id},
                "character": "Necrobinder & <Friends>", "win": {win},
                "abandoned": false, "ascension": 3, "acts": 3, "floors": 34,
                "killed_by": null, "kill_kind": null, "run_time": 7534,
                "seed": "SEED", "game_mode": "standard", "build_id": "v0.1"}}"#
        ))
        .unwrap()
    }

    #[test]
    fn one_item_per_log_entry() {
        let xml = rss_xml(ORIGIN, &[]);
        assert_eq!(xml.matches("<item>").count(), LOG.len());
        assert_eq!(xml.matches("</item>").count(), LOG.len());
    }

    #[test]
    fn wins_join_the_feed_and_losses_stay_out() {
        let runs = [
            run("1784587453", "2026-07-20", true),
            run("1784500000", "2026-07-19", false),
        ];
        let xml = rss_xml(ORIGIN, &runs);
        assert_eq!(xml.matches("<item>").count(), LOG.len() + 1);
        assert!(xml.contains(&format!("{ORIGIN}/spire/run/1784587453")));
        assert!(!xml.contains("1784500000"));
        assert!(xml.contains("<pubDate>Mon, 20 Jul 2026 00:00:00 +0000</pubDate>"));
    }

    #[test]
    fn items_are_sorted_newest_first_with_curated_leading_ties() {
        // One win newer than every log entry, one sharing the newest log date.
        let runs = [
            run("1784587453", "2026-07-20", true),
            run("1752300000", "2026-07-12", true),
        ];
        let xml = rss_xml(ORIGIN, &runs);
        let win_new = xml.find("/spire/run/1784587453").unwrap();
        let essay = xml.find("How bad are planes?").unwrap();
        let win_tied = xml.find("/spire/run/1752300000").unwrap();
        assert!(win_new < essay, "newest win leads the feed");
        assert!(essay < win_tied, "curated entry leads a same-date win");
    }

    #[test]
    fn run_fields_are_escaped() {
        let runs = [run("1784587453", "2026-07-20", true)];
        let xml = rss_xml(ORIGIN, &runs);
        assert!(xml.contains("Necrobinder &amp; &lt;Friends&gt;"));
        assert!(!xml.contains("<Friends>"));
    }

    #[test]
    fn no_raw_ampersands_outside_entities() {
        let runs = [run("1784587453", "2026-07-20", true)];
        let xml = rss_xml(ORIGIN, &runs);
        let mut rest = xml.as_str();
        while let Some(pos) = rest.find('&') {
            let tail = &rest[pos..];
            assert!(
                ["&amp;", "&lt;", "&gt;", "&quot;", "&apos;"]
                    .iter()
                    .any(|e| tail.starts_with(e)),
                "raw ampersand near: {}",
                &tail[..tail.len().min(40)]
            );
            rest = &rest[pos + 1..];
        }
    }

    #[test]
    fn pub_dates_are_rfc2822_with_correct_weekdays() {
        // Hand-checked calendar facts.
        assert_eq!(rfc2822("2026-07-12"), "Sun, 12 Jul 2026 00:00:00 +0000");
        assert_eq!(rfc2822("2019-03-30"), "Sat, 30 Mar 2019 00:00:00 +0000");
        assert_eq!(rfc2822("2018-07-10"), "Tue, 10 Jul 2018 00:00:00 +0000");
        assert_eq!(rfc2822("2024-11-08"), "Fri, 08 Nov 2024 00:00:00 +0000");

        // Every emitted pubDate matches the RFC 2822 shape.
        let xml = rss_xml(ORIGIN, &[run("1784587453", "2026-07-20", true)]);
        for line in xml.lines().filter(|l| l.starts_with("<pubDate>")) {
            let date = line
                .trim_start_matches("<pubDate>")
                .trim_end_matches("</pubDate>");
            let ok = date.len() == 31
                && ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"].contains(&&date[0..3])
                && &date[3..5] == ", "
                && date[5..7].chars().all(|c| c.is_ascii_digit())
                && date.ends_with("00:00:00 +0000");
            assert!(ok, "not RFC 2822: {date}");
        }
    }

    #[test]
    fn guids_are_unique() {
        let runs = [
            run("1784587453", "2026-07-20", true),
            run("1784400000", "2026-07-18", true),
        ];
        let xml = rss_xml(ORIGIN, &runs);
        let guids: Vec<&str> = xml.lines().filter(|l| l.starts_with("<guid")).collect();
        assert_eq!(guids.len(), LOG.len() + 2);
        let mut deduped = guids.clone();
        deduped.sort_unstable();
        deduped.dedup();
        assert_eq!(deduped.len(), guids.len(), "duplicate guid");
    }

    #[test]
    fn note_titles_truncate_on_char_boundary() {
        assert_eq!(truncate("short", 80), "short");
        let long = "ré".repeat(60);
        let cut = truncate(&long, 80);
        assert!(cut.ends_with('…'));
        assert_eq!(cut.chars().count(), 81);
    }

    #[test]
    fn escape_covers_the_five() {
        assert_eq!(escape(r#"a&b<c>d"e'f"#), "a&amp;b&lt;c&gt;d&quot;e&apos;f");
    }

    #[test]
    fn structure_is_sound() {
        let xml = rss_xml(ORIGIN, &[]);
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("<title>Ben Berman — logbook</title>"));
        assert!(xml.contains(&format!("<link>{ORIGIN}/</link>")));
        assert!(xml.contains(&format!(
            "<atom:link href=\"{ORIGIN}/feed.xml\" rel=\"self\" type=\"application/rss+xml\"/>"
        )));
        // Internal update hrefs got the origin prefix; externals kept theirs.
        assert!(xml.contains(&format!("<link>{ORIGIN}/keyboards</link>")));
        assert!(xml.contains("<link>https://www.youtube.com/watch?v=8lrjsP1KWrY</link>"));
        // Serial-derived guids span 0001..=count.
        assert!(xml.contains(&format!(
            "{ORIGIN}/log/{:04}",
            crate::content::logbook::LOG.len()
        )));
        assert!(xml.contains(&format!("{ORIGIN}/log/0001")));
        assert!(xml.trim_end().ends_with("</rss>"));
    }
}
