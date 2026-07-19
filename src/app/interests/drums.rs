use topcoat::{Result, router::page, view::view};

use crate::{
    content::interests::interest,
    design::{page_head, shell},
};

#[page("/interests/drums")]
async fn drums() -> Result {
    let meta = interest("drums");
    let title = format!("{} — Ben Berman", meta.title);
    let body = view! {
        page_head(stamp: meta.slug, title: meta.title, lede: meta.teaser)
        <section class="rail-row mt-10">
            <div></div>
            <div class="min-w-0 max-w-prose space-y-4 text-ink2">
                <p>
                    "I had my first lesson at summer camp, which is to say I've been playing long \
                     enough that I should be better. These days it's a Tama pancake kit — quiet \
                     enough for an apartment, portable enough that I have set it up in a park."
                </p>
                <p>"Two covers survive public scrutiny."</p>
            </div>
        </section>
        <div class="rail-row mt-10">
            <p class="rail-stamp uppercase tracking-[0.18em]">"footage"</p>
            <div class="flex min-w-0 flex-wrap gap-5">
                <a class="video-card" href="https://www.youtube.com/watch?v=VaKI7J2M2Ms">
                    <img
                        src="https://img.youtube.com/vi/VaKI7J2M2Ms/mqdefault.jpg"
                        alt="Taylor Swift cover →"
                        loading="lazy"
                    >
                    <span class="video-card-label font-meta text-sm text-ink2">
                        "Taylor Swift cover →"
                    </span>
                </a>
                <a class="video-card" href="https://www.youtube.com/watch?v=8lrjsP1KWrY">
                    <img
                        src="https://img.youtube.com/vi/8lrjsP1KWrY/mqdefault.jpg"
                        alt="Manchester Orchestra cover →"
                        loading="lazy"
                    >
                    <span class="video-card-label font-meta text-sm text-ink2">
                        "Manchester Orchestra cover →"
                    </span>
                </a>
            </div>
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
