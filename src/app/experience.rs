use topcoat::{Result, router::page, view::view};

use crate::design::shell;

#[page("/experience")]
async fn experience() -> Result {
    let body = view! { <h1 class="font-display text-3xl font-bold">"Experience"</h1> }?;
    view! { shell(title: "Experience — Ben Berman", body: body) }
}
