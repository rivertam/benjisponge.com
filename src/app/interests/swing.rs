use topcoat::{Result, router::page, view::view};

use crate::{
    content::interests::interest,
    design::{page_head, shell},
};

#[page("/interests/swing")]
async fn swing() -> Result {
    let meta = interest("swing");
    let title = format!("{} — Ben Berman", meta.title);
    let body = view! {
        page_head(stamp: meta.slug, title: meta.title, lede: meta.teaser)
        <section class="rail-row mt-10">
            <div></div>
            <div class="min-w-0 max-w-prose space-y-4 text-ink2">
                <p>
                    "I started with group classes in midtown in 2023 and it immediately ate my \
                     evenings — there was a stretch where I was at a social most nights of the week."
                </p>
                <p>
                    "The pitch, which I will deliver unprompted: you rotate partners every few \
                     minutes, nobody knows you, and it is the single most efficient way to make a \
                     week better. If you're in New York, take the intro class. If you're not, your \
                     city has a scene too."
                </p>
            </div>
        </section>
        <div class="rail-row mt-14">
            <div></div>
            <p class="min-w-0 font-meta text-sm">
                <a class="quiet-link" href="/interests">"← all interests"</a>
            </p>
        </div>
    }?;
    view! { shell(title: title.as_str(), active: "interests", body: body) }
}
