use topcoat::{Result, router::page, view::view};

use crate::{
    components::{back_link, page_head, rail_prose, rail_section, shell},
    content::interests::interest,
};

#[page("/interests/puzzles")]
async fn puzzles() -> Result {
    let meta = interest("puzzles");
    let prose = view! {
        <p>
            "puzuzu parses AcrossLite .puz files and gives you a solving TUI — in Rust, \
             published to npm, demo recording in the README."
        </p>
        <p>"It has three GitHub stars and I earned every one of them."</p>
    }?;
    let links = view! {
        <p class="flex flex-wrap gap-x-4 gap-y-1 font-meta text-sm">
            <a class="oxlink" href="https://github.com/rivertam/puzuzu">"puzuzu →"</a>
        </p>
    }?;
    let body = view! {
        page_head(stamp: meta.slug, title: meta.title, lede: meta.teaser)
        rail_prose(mt: "mt-10", stamp: "", body: prose)
        rail_section(mt: "mt-6", stamp: "links", body: links)
        back_link(href: "/interests", label: "← all interests")
    }?;
    view! { shell(title: meta.title, active: "interests", body: body) }
}
