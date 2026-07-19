//! Off the clock: the interests index and one page per interest, routed by
//! the interest's stamp (`/off-the-clock/drums`). The nav dropdown in
//! `design.rs` iterates the same registry, so a new entry in
//! `content/interests.rs` shows up everywhere at once.

use topcoat::{
    Result,
    context::Cx,
    router::{StatusCode, page, path_param},
    view::view,
};

use crate::{
    content::interests::{Evidence, INTERESTS, Interest},
    design::{page_head, shell},
};

pub fn interest_href(interest: &Interest) -> String {
    format!("/off-the-clock/{}", interest.stamp)
}

/// The video id, if the link is a YouTube watch URL — those render as
/// thumbnail cards instead of plain links.
fn youtube_id(url: &str) -> Option<&str> {
    url.strip_prefix("https://www.youtube.com/watch?v=")
}

#[page("/off-the-clock")]
async fn off_the_clock() -> Result {
    let body = view! {
        page_head(
            stamp: "exhibits",
            title: "Off the clock",
            lede: "Skills of no professional value whatsoever. Everything here is, regrettably, public record.",
        )
        <section class="mt-14 space-y-10">
            for interest in INTERESTS.iter() {
                <article class="rail-row">
                    <p class="rail-stamp">(interest.stamp)</p>
                    <div class="min-w-0">
                        <h2 class="font-display text-2xl leading-snug font-semibold">
                            <a class="oxlink" href=(interest_href(interest))>(interest.title)</a>
                        </h2>
                        <p class="mt-1.5 max-w-prose text-ink2">(interest.line)</p>
                    </div>
                </article>
            }
        </section>
    }?;
    view! { shell(title: "Off the clock — Ben Berman", body: body) }
}

// A `String` segment can't fail to parse, so the error arm is dead; unknown
// slugs are handled below with a proper 404 page instead.
#[path_param(error = not_found())]
struct Slug(String);

#[page("/off-the-clock/{slug}")]
async fn interest_page(cx: &Cx) -> Result {
    let slug = path_param::<Slug>(cx)?;
    let interest = INTERESTS.iter().find(|i| i.stamp == slug.as_str());

    let title = match interest {
        Some(interest) => format!("{} — Ben Berman", interest.title),
        None => "404 — Ben Berman".to_string(),
    };

    let body = match interest {
        Some(interest) => {
            let videos: Vec<&Evidence> = interest
                .links
                .iter()
                .filter(|link| youtube_id(link.url).is_some())
                .collect();
            let plain: Vec<&Evidence> = interest
                .links
                .iter()
                .filter(|link| youtube_id(link.url).is_none())
                .collect();
            view! {
                page_head(stamp: interest.stamp, title: interest.title, lede: interest.line)
                <section class="rail-row mt-10">
                    <div></div>
                    <div class="min-w-0 max-w-prose space-y-4 text-ink2">
                        for para in interest.blurb.iter() {
                            <p>(*para)</p>
                        }
                    </div>
                </section>
                if !videos.is_empty() {
                    <div class="rail-row mt-10">
                        <p class="rail-stamp uppercase tracking-[0.18em]">"footage"</p>
                        <div class="flex min-w-0 flex-wrap gap-5">
                            for video in videos.iter() {
                                <a class="video-card" href=(video.url)>
                                    <img
                                        src=(format!(
                                            "https://img.youtube.com/vi/{}/mqdefault.jpg",
                                            youtube_id(video.url).expect("filtered to youtube links"),
                                        ))
                                        alt=(video.label)
                                        loading="lazy"
                                    >
                                    <span class="video-card-label font-meta text-sm text-ink2">(video.label)</span>
                                </a>
                            }
                        </div>
                    </div>
                }
                if !plain.is_empty() {
                    <div class="rail-row mt-6">
                        <p class="rail-stamp uppercase tracking-[0.18em]">"links"</p>
                        <p class="flex min-w-0 flex-wrap gap-x-4 gap-y-1 font-meta text-sm">
                            for link in plain.iter() {
                                <a class="oxlink" href=(link.url)>(link.label)</a>
                            }
                        </p>
                    </div>
                }
                <div class="rail-row mt-14">
                    <div></div>
                    <p class="min-w-0 font-meta text-sm">
                        <a class="quiet-link" href="/off-the-clock">"← the rest of the exhibits"</a>
                    </p>
                </div>
            }?
        }
        None => view! {
            (StatusCode::NOT_FOUND)
            page_head(stamp: "404", title: "No such interest.", lede: "Plenty of others, though.")
            <div class="rail-row mt-8">
                <p class="rail-stamp uppercase tracking-[0.18em]">"the index"</p>
                <ul class="min-w-0 space-y-1 font-meta text-sm">
                    for interest in INTERESTS.iter() {
                        <li>
                            <a class="quiet-link" href=(interest_href(interest))>(interest_href(interest).as_str())</a>
                        </li>
                    }
                </ul>
            </div>
        }?,
    };
    view! { shell(title: title.as_str(), body: body) }
}
