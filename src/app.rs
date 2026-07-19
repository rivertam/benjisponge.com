mod feed;
mod interests;
mod not_found;
mod resume;
mod thoughts;

use topcoat::{
    Result,
    asset::{AssetBundle, RouterBuilderAssetExt},
    context::Cx,
    router::{Router, RouterBuilderDiscoverExt, page, query_params},
    view::view,
};

use crate::{
    content::logbook::{Entry, FILTER_TAGS, Kind, LOG, serial},
    design::shell,
};

pub fn router() -> Router {
    Router::builder()
        .assets(AssetBundle::load().unwrap())
        .discover()
        .build()
}

/// The hero's rotating objects of affection. CSS cycles through them
/// (`.log-hero-word` in `styles/logbook.css`); reduced motion pins the first.
static HERO_WORDS: [&str; 6] = [
    "Rust.",
    "drums.",
    "split keyboards.",
    "my dog.",
    "crosswords.",
    "swing.",
];

#[query_params(error = redirect("?"))]
struct HomeQuery {
    kind: Option<String>,
    tag: Option<String>,
}

/// A filter-row chip: a link that sets, swaps, or clears one query param.
struct Chip {
    label: String,
    href: String,
    active: bool,
}

/// One visible timeline row, precomputed so the markup stays declarative.
struct Row {
    /// Set when the year changes between consecutive visible entries.
    year_mark: Option<&'static str>,
    serial: String,
    entry: &'static Entry,
}

/// The homepage URL for a filter state, dropping absent params.
fn home_url(kind: Option<&str>, tag: Option<&str>) -> String {
    let mut params = Vec::new();
    if let Some(kind) = kind {
        params.push(format!("kind={kind}"));
    }
    if let Some(tag) = tag {
        params.push(format!("tag={}", urlencode(tag)));
    }
    if params.is_empty() {
        "/".to_string()
    } else {
        format!("/?{}", params.join("&"))
    }
}

/// Percent-encode a query value (tags come back from the URL, so round-trip
/// anything beyond the unreserved set).
fn urlencode(raw: &str) -> String {
    let mut encoded = String::new();
    for byte in raw.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char)
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

#[page("/")]
async fn home(cx: &Cx) -> Result {
    let q = query_params::<HomeQuery>(cx)?;
    // An unknown kind silently falls back to the full log; a tag filters
    // whatever it names (arbitrary tags just match fewer entries).
    let kind = match q.kind.as_deref() {
        Some("essay") => Some(Kind::Essay),
        Some("note") => Some(Kind::Note),
        Some("update") => Some(Kind::Update),
        _ => None,
    };
    let kind_param = kind.map(|k| match k {
        Kind::Essay => "essay",
        Kind::Note => "note",
        Kind::Update => "update",
    });
    let tag = q.tag.as_deref().filter(|t| !t.is_empty());

    let kind_chips: Vec<Chip> = [
        ("all", None),
        ("essays", Some("essay")),
        ("notes", Some("note")),
        ("updates", Some("update")),
    ]
    .into_iter()
    .map(|(label, value)| Chip {
        label: label.to_string(),
        href: home_url(value, tag),
        active: kind_param == value,
    })
    .collect();

    // The six fixed tags; an active tag from outside the list joins the row
    // so it can be toggled off.
    let mut tag_chips: Vec<Chip> = FILTER_TAGS
        .iter()
        .map(|t| {
            let active = tag == Some(t);
            Chip {
                label: if active {
                    format!("#{t} ×")
                } else {
                    format!("#{t}")
                },
                href: if active {
                    home_url(kind_param, None)
                } else {
                    home_url(kind_param, Some(t))
                },
                active,
            }
        })
        .collect();
    if let Some(t) = tag
        && !FILTER_TAGS.contains(&t)
    {
        tag_chips.push(Chip {
            label: format!("#{t} ×"),
            href: home_url(kind_param, None),
            active: true,
        });
    }

    let mut rows: Vec<Row> = Vec::new();
    let mut last_year: Option<&str> = None;
    for (index, entry) in LOG.iter().enumerate() {
        if kind.is_some_and(|k| entry.kind() != k)
            || tag.is_some_and(|t| !entry.tags().contains(&t))
        {
            continue;
        }
        let year = &entry.date()[0..4];
        let year_mark = match last_year {
            Some(prev) if prev != year => Some(year),
            _ => None,
        };
        last_year = Some(year);
        rows.push(Row {
            year_mark,
            serial: serial(index),
            entry,
        });
    }

    let body = view! {
        // Hero: "I like {word}", the word cycling through the log's whole
        // range of enthusiasms. Pure CSS — see .log-hero-* in logbook.css.
        <section class="mt-16">
            <h1 class="log-hero font-display text-[2.75rem] leading-none font-bold tracking-tight sm:text-[4rem]">
                "I like "
                <span class="log-hero-words">
                    for word in HERO_WORDS.iter() {
                        <span class="log-hero-word text-oxide">(word)</span>
                    }
                </span>
            </h1>
            <p class="mt-4 max-w-prose text-[17px] leading-relaxed text-ink2">
                "Software engineer in New York; co-founder of DigiChem. This is the \
                 logbook — everything gets an entry, long or short."
            </p>
            <p class="mt-5 font-meta text-[13px] text-muted">
                "now — building DigiChem · recording drums · Ascension 20 "
                <span class="log-caret text-oxide">"▍"</span>
            </p>
        </section>

        // Filter row: kind chips, tag chips, and the feed. Server-side —
        // every chip is a link that rewrites the query string.
        <div class="mt-11 flex flex-wrap items-baseline gap-4 border-t border-hairline pt-4 font-meta text-[13px]">
            for chip in kind_chips.iter() {
                <a
                    class=(if chip.active { "log-chip log-chip-active" } else { "log-chip" })
                    href=(chip.href.as_str())
                >(chip.label.as_str())</a>
            }
            <span class="text-hairline">"|"</span>
            for chip in tag_chips.iter() {
                <a
                    class=(if chip.active { "log-tag log-tag-active" } else { "log-tag" })
                    href=(chip.href.as_str())
                >(chip.label.as_str())</a>
            }
            <a class="ml-auto text-muted hover:text-oxide" href="/feed.xml">"rss ↗"</a>
        </div>

        // The timeline: one vertical hairline, a marker per entry, a year
        // badge wherever the visible entries change year.
        <section class="log-timeline">
            for row in rows.iter() {
                if let Some(year) = row.year_mark {
                    <div class="log-row">
                        <span class="log-year">(year)</span>
                    </div>
                }
                if let Entry::Essay { title, teaser, slug, tags, .. } = row.entry {
                    <article class="log-row">
                        <span class="log-mark log-mark-essay"></span>
                        <div class="log-rail">
                            <p class="log-date">(row.entry.date())</p>
                            <p class="log-serial">(row.serial.as_str())</p>
                        </div>
                        <div class="log-card">
                            <span class="log-stamp">"essay"</span>
                            <h2 class="log-card-title font-display font-bold">
                                <a class="oxlink" href=(format!("/thoughts/{slug}"))>(title)</a>
                            </h2>
                            <p class="mt-2.5 max-w-prose leading-relaxed text-ink2">(teaser)</p>
                            <div class="mt-4 flex flex-wrap items-baseline gap-3 font-meta text-xs">
                                for t in tags.iter() {
                                    <a class="log-tag" href=(home_url(kind_param, Some(t)))>(format!("#{t}"))</a>
                                }
                                <a
                                    class="ml-auto text-ink2 no-underline hover:text-oxide"
                                    href=(format!("/thoughts/{slug}"))
                                >"read →"</a>
                            </div>
                        </div>
                    </article>
                }
                if let Entry::Note { body, source, slug, .. } = row.entry {
                    <article class="log-row">
                        <span class="log-mark log-mark-note"></span>
                        <div class="log-rail">
                            <p class="log-date">(row.entry.date())</p>
                            <p class="log-serial">(row.serial.as_str())</p>
                        </div>
                        <div class="log-note min-w-0">
                            <p class="log-note-body font-display">(body)</p>
                            <p class="mt-2.5 font-meta text-xs text-muted">
                                "note · "
                                (source)
                                " · "
                                <a class="log-permalink" href=(format!("/thoughts/{slug}"))>"permalink"</a>
                            </p>
                        </div>
                    </article>
                }
                if let Entry::Update { stamp, label, body, href, link_label, .. } = row.entry {
                    <article class="log-row items-baseline">
                        <span class="log-mark log-mark-update"></span>
                        <p class="log-date">(row.entry.date())</p>
                        <p class="log-update min-w-0">
                            <span class="log-update-stamp">(format!("[{stamp}]"))</span>
                            " "
                            <span class="text-patina">(format!("{label} ·"))</span>
                            " "
                            (body)
                            " "
                            <a class="log-update-link" href=(href)>(link_label)</a>
                        </p>
                    </article>
                }
            }
        </section>
    }?;
    view! { shell(title: "Ben Berman", active: "log", body: body) }
}
