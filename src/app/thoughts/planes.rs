mod charts;
mod form;
mod receipt;

use topcoat::{Result, router::page, view::view};

use crate::design::shell;

#[page("/thoughts/how-bad-are-planes")]
async fn planes() -> Result {
    let body =
        view! { <h1 class="font-display text-3xl font-bold">"How bad are planes?"</h1> }?;
    view! { shell(title: "How bad are planes? — Ben Berman", body: body) }
}
