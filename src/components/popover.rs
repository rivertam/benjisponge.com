//! Inline citation popovers.

use topcoat::{
    Result,
    view::{View, component, view},
};

/// An inline, dismissible popover for citations and other short asides.
/// `id` must be unique on the page and a valid CSS custom-ident fragment.
#[component]
pub async fn inline_popover(id: &str, label: &str, child: View) -> Result {
    let anchor_name = format!("anchor-name: --inline-popover-{};", id);
    let position_anchor = format!("position-anchor: --inline-popover-{};", id);
    view! {
        <button
            type="button"
            class="inline-popover-trigger oxlink"
            popovertarget=(id)
            style=(anchor_name.as_str())
        >(label)</button>
        <span
            id=(id)
            class="inline-popover-panel"
            popover="auto"
            style=(position_anchor.as_str())
        >
            <button
                type="button"
                class="inline-popover-close"
                popovertarget=(id)
                popovertargetaction="hide"
                aria-label="Close popover"
            >"×"</button>
            <span class="inline-popover-kicker">(label)</span>
            (child)
        </span>
    }
}
