use topcoat::{Result, router::page, view::view};

use crate::{
    content::interests::interest,
    design::{page_head, shell},
};

#[page("/interests/puzzles")]
async fn puzzles() -> Result {
    let meta = interest("puzzles");
    let title = format!("{} — Ben Berman", meta.title);
    let body = view! {
        page_head(stamp: meta.slug, title: meta.title, lede: meta.teaser)
        <section class="rail-row mt-10">
            <div></div>
            <div class="min-w-0 max-w-prose space-y-4 text-ink2">
                <p>
                    "puzuzu parses AcrossLite .puz files and gives you a solving TUI — in Rust, \
                     published to npm, demo recording in the README."
                </p>
                <p>"It has three GitHub stars and I earned every one of them."</p>
            </div>
        </section>
        <div class="rail-row mt-6">
            <p class="rail-stamp uppercase tracking-[0.18em]">"links"</p>
            <p class="flex min-w-0 flex-wrap gap-x-4 gap-y-1 font-meta text-sm">
                <a class="oxlink" href="https://github.com/rivertam/puzuzu">"puzuzu →"</a>
            </p>
        </div>
        <div class="rail-row mt-14">
            <div></div>
            <p class="min-w-0 font-meta text-sm">
                <a class="quiet-link" href="/interests">"← all interests"</a>
            </p>
        </div>
    }?;
    view! { shell(title: title.as_str(), body: body) }
}
