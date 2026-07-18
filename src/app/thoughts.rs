pub mod pesky_code;
pub mod planes;

use topcoat::{Result, router::page, view::view};

use crate::{
    content::posts::POSTS,
    design::{page_head, shell},
};

#[page("/thoughts")]
async fn thoughts() -> Result {
    let body = view! {
        page_head(
            stamp: "log",
            title: "Thoughts",
            lede: "Thoughts of varying seriousness and length.",
        )
        <section class="mt-14 space-y-10">
            for post in POSTS.iter() {
                <article class="rail-row">
                    <p class="rail-stamp">(post.date)</p>
                    <div class="min-w-0">
                        <h2 class="font-display text-2xl leading-snug font-semibold">
                            <a class="oxlink" href=(format!("/thoughts/{}", post.slug))>(post.title)</a>
                        </h2>
                        <p class="mt-1.5 max-w-prose text-ink2">(post.teaser)</p>
                    </div>
                </article>
            }
        </section>
    }?;
    view! { shell(title: "Thoughts — Ben Berman", body: body) }
}
