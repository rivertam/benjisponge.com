use topcoat::{Result, router::page, view::view};

use crate::{
    content::interests::interest,
    design::{page_head, shell},
};

#[page("/interests/felix")]
async fn felix() -> Result {
    let meta = interest("felix");
    let title = format!("{} — Ben Berman", meta.title);
    let body = view! {
        page_head(stamp: meta.slug, title: meta.title, lede: meta.teaser)
        <section class="rail-row mt-10">
            <div></div>
            <div class="min-w-0 max-w-prose space-y-4 text-ink2">
                <p>
                    "The dog is Felix. The website is saamd.com — Same Age As My Dog — which \
                     computes the one day on which a dog and its person are, in dog years, the \
                     same age."
                </p>
                <p>
                    "This date matters to no one, which is why it needed a calculator."
                </p>
            </div>
        </section>
        <div class="rail-row mt-6">
            <p class="rail-stamp uppercase tracking-[0.18em]">"links"</p>
            <p class="flex min-w-0 flex-wrap gap-x-4 gap-y-1 font-meta text-sm">
                <a class="oxlink" href="https://www.saamd.com">"saamd.com →"</a>
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
