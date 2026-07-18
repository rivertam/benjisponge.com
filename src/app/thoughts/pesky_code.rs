use topcoat::{Result, router::page, view::view};

use crate::design::shell;

#[page("/thoughts/pesky-code")]
async fn pesky_code() -> Result {
    let body = view! { <h1 class="font-display text-3xl font-bold">"Pesky code"</h1> }?;
    view! { shell(title: "Pesky code — Ben Berman", body: body) }
}
