//! Link components.

use topcoat::{
    Result,
    view::{component, view},
};

/// Splits a visual trailing arrow from a link label so the glyph can be
/// optically aligned without affecting the link's readable text.
pub(crate) fn split_link_label(label: &str) -> (&str, Option<&'static str>) {
    for (suffix, arrow) in [(" ←", "←"), (" →", "→"), (" ↗", "↗")] {
        if let Some(text) = label.strip_suffix(suffix) {
            return (text, Some(arrow));
        }
    }
    (label, None)
}

/// A link label whose optional trailing arrow is optically centered.
#[component]
pub async fn link_label(label: &str) -> Result {
    let (text, arrow) = split_link_label(label);
    let arrow = match arrow {
        Some("←") => Some("<-"),
        Some(_) => Some("->"),
        None => None,
    };
    view! {
        (text)
        if let Some(arrow) = arrow {
            <span class="link-arrow link-arrow-after" aria-hidden="true">(arrow)</span>
        }
    }
}

/// An outbound link: new tab, no opener/referrer leak.
#[component]
pub async fn ext_link(class: &str, href: &str, label: &str) -> Result {
    view! {
        <a class=(class) href=(href) target="_blank" rel="noopener noreferrer">
            link_label(label: label)
        </a>
    }
}
