//! The margin rail: stamped rows, page heads, and the closing back-link row.

use topcoat::{
    Result,
    view::{View, component, view},
};

/// A page's opening rail row: a mono stamp in the margin, a Zilla Slab title,
/// and an optional one-line lede (pass `""` to omit).
#[component]
pub async fn page_head(stamp: &str, title: &str, lede: &str) -> Result {
    view! {
        <header class="rail-row mt-16">
            <p class="rail-stamp rail-stamp-label">(stamp)</p>
            <div class="min-w-0">
                <h1 class="font-display text-4xl font-bold tracking-tight">(title)</h1>
                if !lede.is_empty() {
                    <p class="mt-3 max-w-prose text-ink2">(lede)</p>
                }
            </div>
        </header>
    }
}

/// One rail row: a stamped label in the margin column, the body in the
/// content column. `stamp: ""` renders the empty spacer cell instead (prose
/// continuation rows). `class` is optional extra classes on the row — it
/// defaults to `"mt-10"`, so pass e.g. `class: "mt-6"` to tighten the top
/// margin or `class: ""` inside an already-spaced parent. Child markup follows
/// the named properties, e.g. `rail_section(stamp: "links", <p>"…"</p>)`.
#[component]
pub async fn rail_section(#[default("mt-10")] class: &str, stamp: &str, child: View) -> Result {
    let row_class = if class.is_empty() {
        "rail-row".to_string()
    } else {
        format!("rail-row {class}")
    };
    view! {
        <div class=(row_class.as_str())>
            if stamp.is_empty() {
                <div></div>
            } else {
                <p class="rail-stamp rail-stamp-label">(stamp)</p>
            }
            <div class="min-w-0">(child)</div>
        </div>
    }
}

/// A rail row whose body is running prose: paragraphs at reading measure in
/// the secondary ink. `class` works as on [`rail_section`].
#[component]
pub async fn rail_prose(#[default("mt-10")] class: &str, stamp: &str, child: View) -> Result {
    let prose = view! { <div class="max-w-prose space-y-4 text-ink2">(child)</div> }?;
    view! { rail_section(class: class, stamp: stamp, (prose)) }
}

/// A page's closing rail row: a quiet link back up to the section index.
#[component]
pub async fn back_link(href: &str, label: &str) -> Result {
    let label = label.strip_prefix("← ").unwrap_or(label);
    view! {
        <div class="rail-row mt-14">
            <div></div>
            <p class="min-w-0 font-meta text-sm">
                <a class="quiet-link" href=(href)>
                    <span class="link-arrow link-arrow-before" aria-hidden="true">"<-"</span>
                    (label)
                </a>
            </p>
        </div>
    }
}
