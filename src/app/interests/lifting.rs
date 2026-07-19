use topcoat::{
    Result,
    router::{page, redirect_permanent, route},
    view::view,
};

use crate::{
    components::{back_link, page_head, rail_prose, shell},
    content::interests::interest,
};

#[page("/lifting")]
async fn lifting() -> Result {
    let meta = interest("lifting");
    view! { shell(title: meta.title, active: "interests",
        page_head(stamp: meta.slug, title: meta.title, lede: meta.teaser)
        rail_prose(stamp: "",
            <p>
                "Five days a week, mostly the big compounds, entirely plant-powered. The \
                 numbers above are not impressive and I am at peace with that; the streak is \
                 the point."
            </p>
        )
        back_link(href: "/interests", label: "all interests")
    ) }
}

#[route(GET "/interests/lifting")]
async fn legacy_lifting() -> Result {
    Err(redirect_permanent("/lifting").into())
}
