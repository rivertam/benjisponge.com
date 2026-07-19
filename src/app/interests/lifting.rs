use topcoat::{Result, router::page, view::view};

use crate::{
    components::{back_link, page_head, rail_prose, shell},
    content::interests::interest,
};

#[page("/interests/lifting")]
async fn lifting() -> Result {
    let meta = interest("lifting");
    let prose = view! {
        <p>
            "Five days a week, mostly the big compounds, entirely plant-powered. The \
             numbers above are not impressive and I am at peace with that; the streak is \
             the point."
        </p>
    }?;
    let body = view! {
        page_head(stamp: meta.slug, title: meta.title, lede: meta.teaser)
        rail_prose(stamp: "", body: prose)
        back_link(href: "/interests", label: "← all interests")
    }?;
    view! { shell(title: meta.title, active: "interests", body: body) }
}
