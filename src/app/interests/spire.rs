use topcoat::{Result, router::page, view::view};

use crate::{
    content::interests::interest,
    design::{page_head, shell},
};

#[page("/interests/spire")]
async fn spire() -> Result {
    let meta = interest("spire");
    let title = format!("{} — Ben Berman", meta.title);
    let body = view! {
        page_head(stamp: meta.slug, title: meta.title, lede: meta.teaser)
        <section class="rail-row mt-10">
            <div></div>
            <div class="min-w-0 max-w-prose space-y-4 text-ink2">
                <p>
                    "Ascension 20 is the highest difficulty the game offers. The writeup exists \
                     because the win deserved documentation more than it deserved celebration."
                </p>
                <p>
                    "I also maintain opinions about the RNG that I can support with screenshots."
                </p>
            </div>
        </section>
        <div class="rail-row mt-6">
            <p class="rail-stamp uppercase tracking-[0.18em]">"links"</p>
            <p class="flex min-w-0 flex-wrap gap-x-4 gap-y-1 font-meta text-sm">
                <a
                    class="oxlink"
                    href="https://reddit.com/r/slaythespire/comments/jkqx35/annotated_run_synopsis_my_second_a20_win_only_a/"
                >"the synopsis →"</a>
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
