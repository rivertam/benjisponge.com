use topcoat::{Result, router::page, view::view};

use crate::design::{page_head, shell};

#[page("/interests/lifting")]
async fn lifting() -> Result {
    let body = view! {
        page_head(
            stamp: "lifting",
            title: "Lifting",
            lede: "Deadlift PR 345 lbs, Squat PR 235 lbs, Bench PR like 165 but I never 1RM it",
        )
        <section class="rail-row mt-10">
            <div></div>
            <div class="min-w-0 max-w-prose space-y-4 text-ink2">
                <p>
                    "Five days a week, mostly the big compounds, entirely plant-powered. The \
                     numbers above are not impressive and I am at peace with that; the streak is \
                     the point."
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
    view! { shell(title: "Lifting — Ben Berman", body: body) }
}
