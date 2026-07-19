use topcoat::{Result, router::page, view::view};

use crate::{
    components::{back_link, page_head, rail_prose, rail_section, shell},
    content::interests::interest,
};

#[page("/interests/felix")]
async fn felix() -> Result {
    let meta = interest("felix");
    view! { shell(title: meta.title, active: "interests",
        page_head(stamp: meta.slug, title: meta.title, lede: meta.teaser)
        rail_prose(stamp: "",
            <p>
                "The dog is Felix. The website is saamd.com — Same Age As My Dog — which \
                 computes the one day on which a dog and its person are, in dog years, the \
                 same age."
            </p>
            <p>
                "This date matters to no one, which is why it needed a calculator."
            </p>
        )
        rail_section(class: "mt-6", stamp: "links",
            <p class="flex flex-wrap gap-x-4 gap-y-1 font-meta text-sm">
                <a class="oxlink" href="https://www.saamd.com">"saamd.com →"</a>
            </p>
        )
        back_link(href: "/interests", label: "← all interests")
    ) }
}
