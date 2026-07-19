use topcoat::{Result, router::page, view::view};

use crate::{
    content::interests::interest,
    design::{page_head, shell},
};

#[page("/interests/simulation")]
async fn simulation() -> Result {
    let meta = interest("simulation");
    let title = format!("{} — Ben Berman", meta.title);
    let body = view! {
        page_head(stamp: meta.slug, title: meta.title, lede: meta.teaser)
        <section class="rail-row mt-10">
            <div></div>
            <div class="min-w-0 max-w-prose space-y-4 text-ink2">
                <p>
                    "A procedural city in react-three-fiber: tensor-field streets, lots, \
                     buildings, parks, and agents with Schelling-style preferences about who \
                     they live near."
                </p>
                <p>
                    "There is a fifteen-minute video in which I explain all of this with the \
                     confidence of someone who has not yet found the bugs."
                </p>
            </div>
        </section>
        <div class="rail-row mt-10">
            <p class="rail-stamp uppercase tracking-[0.18em]">"footage"</p>
            <div class="flex min-w-0 flex-wrap gap-5">
                <a class="video-card" href="https://www.youtube.com/watch?v=Bcd_9LvUr-8">
                    <img
                        src="https://img.youtube.com/vi/Bcd_9LvUr-8/mqdefault.jpg"
                        alt="the video →"
                        loading="lazy"
                    >
                    <span class="video-card-label font-meta text-sm text-ink2">
                        "the video →"
                    </span>
                </a>
            </div>
        </div>
        <div class="rail-row mt-6">
            <p class="rail-stamp uppercase tracking-[0.18em]">"links"</p>
            <p class="flex min-w-0 flex-wrap gap-x-4 gap-y-1 font-meta text-sm">
                <a class="oxlink" href="https://github.com/rivertam/City">"the repo →"</a>
            </p>
        </div>
        <div class="rail-row mt-14">
            <div></div>
            <p class="min-w-0 font-meta text-sm">
                <a class="quiet-link" href="/interests">"← all interests"</a>
            </p>
        </div>
    }?;
    view! { shell(title: title.as_str(), active: "interests", body: body) }
}
