use topcoat::{Result, router::page, view::view};

use crate::{
    components::{back_link, page_head, rail_prose, shell},
    content::interests::interest,
};

#[page("/interests/swing")]
async fn swing() -> Result {
    let meta = interest("swing");
    let prose = view! {
        <p>
            "I started with group classes in midtown in 2023 and it immediately ate my \
             evenings — there was a stretch where I was at a social most nights of the week."
        </p>
        <p>
            "The pitch, which I will deliver unprompted: you rotate partners every few \
             minutes, nobody knows you, and it is the single most efficient way to make a \
             week better. If you're in New York, take the intro class. If you're not, your \
             city has a scene too."
        </p>
    }?;
    let body = view! {
        page_head(stamp: meta.slug, title: meta.title, lede: meta.teaser)
        rail_prose(stamp: "", body: prose)
        back_link(href: "/interests", label: "← all interests")
    }?;
    view! { shell(title: meta.title, active: "interests", body: body) }
}
