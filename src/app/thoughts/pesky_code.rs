use topcoat::{Result, router::page, view::view};

use crate::design::shell;

#[page("/thoughts/pesky-code")]
async fn pesky_code() -> Result {
    // The body is Ben's own LinkedIn post, quoted verbatim.
    let body = view! {
        <article class="rail-row mt-16 sm:mt-24">
            <p class="rail-stamp">"2025-08-14"</p>
            <div class="min-w-0">
                <h1 class="font-display text-4xl font-bold tracking-tight">"Pesky code"</h1>
                <p class="mt-8 max-w-prose text-xl leading-relaxed">
                    "I'm so glad AI can handle all that pesky code for me so I \
                     can focus on what I truly love: navigating endless chains \
                     of SSO sign-ins followed by dashboards to manage settings \
                     and secrets in different environments ❤️"
                </p>
                <p class="mt-8 font-meta text-xs text-muted">
                    "originally a LinkedIn post, August 2025"
                </p>
            </div>
        </article>
    }?;
    view! { shell(title: "Pesky code — Ben Berman", active: "", body: body) }
}
