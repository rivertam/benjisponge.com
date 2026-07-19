//! RSS 2.0 feed at `/feed.xml`, generated from the logbook registry — every
//! entry, long or short, becomes an `<item>`, so publishing to the log
//! publishes to the feed. Not a page: it renders no shell and stays out of
//! `site_routes()` (the 404 index is for pages).

use topcoat::{Result, router::route};

use crate::content::logbook::{Entry, LOG};

/// Where absolute links point. `SITE_ORIGIN` overrides the default at
/// runtime; the default is a placeholder until the real domain is wired up.
fn origin() -> String {
    std::env::var("SITE_ORIGIN").unwrap_or_else(|_| "https://benjisponge.com".to_string())
}

#[route(GET "/feed.xml")]
async fn feed() -> Result<([(&'static str, &'static str); 1], String)> {
    Ok((
        [("Content-Type", "application/rss+xml; charset=utf-8")],
        rss_xml(&origin()),
    ))
}

/// The whole feed as a string. Pure — origin in, XML out.
pub fn rss_xml(origin: &str) -> String {
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
                *teaser,
            ),
            Entry::Note { body, slug, .. } => (
                truncate(body, 80),
                format!("{origin}/thoughts/{slug}"),
                *body,
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
                (format!("[{stamp}] {label} — {body}"), link, *body)
            }
        };
        xml.push_str("<item>\n");
        xml.push_str(&format!("<title>{}</title>\n", escape(&title)));
        xml.push_str(&format!("<link>{}</link>\n", escape(&link)));
        xml.push_str(&format!(
            "<description>{}</description>\n",
            escape(description)
        ));
        xml.push_str(&format!(
            "<guid isPermaLink=\"false\">{}/log/{:04}</guid>\n",
            escape(origin),
            LOG.len() - index
        ));
        xml.push_str(&format!("<pubDate>{}</pubDate>\n", rfc2822(entry.date())));
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
/// +0000`. Weekday via Sakamoto's method — no date crate in the tree.
fn rfc2822(iso: &str) -> String {
    let year: i32 = iso[0..4].parse().expect("logbook date year");
    let month: usize = iso[5..7].parse().expect("logbook date month");
    let day: u32 = iso[8..10].parse().expect("logbook date day");

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

    #[test]
    fn one_item_per_log_entry() {
        let xml = rss_xml(ORIGIN);
        assert_eq!(xml.matches("<item>").count(), LOG.len());
        assert_eq!(xml.matches("</item>").count(), LOG.len());
    }

    #[test]
    fn no_raw_ampersands_outside_entities() {
        let xml = rss_xml(ORIGIN);
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
        let xml = rss_xml(ORIGIN);
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
        let xml = rss_xml(ORIGIN);
        let guids: Vec<&str> = xml.lines().filter(|l| l.starts_with("<guid")).collect();
        assert_eq!(guids.len(), LOG.len());
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
        let xml = rss_xml(ORIGIN);
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("<title>Ben Berman — logbook</title>"));
        assert!(xml.contains(&format!("<link>{ORIGIN}/</link>")));
        assert!(xml.contains(&format!(
            "<atom:link href=\"{ORIGIN}/feed.xml\" rel=\"self\" type=\"application/rss+xml\"/>"
        )));
        // Internal update hrefs got the origin prefix; externals kept theirs.
        assert!(xml.contains(&format!("<link>{ORIGIN}/interests/keys</link>")));
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
