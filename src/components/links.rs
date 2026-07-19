//! Link components.

use topcoat::{
    Result,
    view::{component, view},
};

/// An outbound link: new tab, no opener/referrer leak.
#[component]
pub async fn ext_link(class: &str, href: &str, label: &str) -> Result {
    view! {
        <a class=(class) href=(href) target="_blank" rel="noopener noreferrer">(label)</a>
    }
}
