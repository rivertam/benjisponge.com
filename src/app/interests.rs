//! The interests index. Each linked interest is a standalone page and route
//! in its own module below `app/interests/`.

mod drums;
mod felix;
mod keys;
mod lifting;
mod models;
mod puzzles;
mod spire;
mod swing;

use topcoat::{Result, router::page, view::view};

use crate::{
    content::interests::INTERESTS,
    design::{page_head, shell},
};

#[page("/interests")]
async fn interests() -> Result {
    let body = view! {
        page_head(
            stamp: "index",
            title: "Interests",
            lede: "Skills of no professional value whatsoever. Everything here is, regrettably, public record.",
        )
        <section class="mt-14 space-y-10">
            for interest in INTERESTS.iter() {
                <article class="rail-row">
                    <p class="rail-stamp">(interest.slug)</p>
                    <div class="min-w-0">
                        <h2 class="font-display text-2xl leading-snug font-semibold">
                            <a
                                class="oxlink"
                                href=(format!("/interests/{}", interest.slug))
                            >(interest.title)</a>
                        </h2>
                        <p class="mt-1.5 max-w-prose text-ink2">(interest.teaser)</p>
                    </div>
                </article>
            }
        </section>
    }?;
    view! { shell(title: "Interests — Ben Berman", active: "interests", body: body) }
}
