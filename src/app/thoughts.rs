pub mod pesky_code;
pub mod planes;

use topcoat::{Result, router::page, view::view};

use crate::design::shell;

#[page("/thoughts")]
async fn thoughts() -> Result {
    let body = view! { <h1 class="font-display text-3xl font-bold">"Thoughts"</h1> }?;
    view! { shell(title: "Thoughts — Ben Berman", body: body) }
}
