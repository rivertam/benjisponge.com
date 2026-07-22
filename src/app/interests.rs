//! The interests index. Each linked interest is a standalone top-level page
//! in its own module below `app/interests/`.

mod drums;
mod felix;
mod keyboards;
mod lifting;
mod puzzles;
mod simulation;
mod spire;
mod swing;

use topcoat::{Result, router::page, view::view};

use crate::{
    components::{index_card, page_head, shell},
    content::interests::INTERESTS,
};

#[page("/interests")]
async fn interests() -> Result {
    view! {
        shell(
            title: "Interests",
            active: "interests",
            page_head(stamp: "index", title: "Interests", lede: "I contain multitudes.")
            <section class="mt-14 space-y-10">
                for interest in INTERESTS.iter() {
                    index_card(
                        stamp: interest.slug,
                        href: format!("/{}", interest.slug),
                        title: interest.title,
                        teaser: interest.teaser
                    )
                }
            </section>
        )
    }
}
