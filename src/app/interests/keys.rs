use topcoat::{Result, router::page, view::view};

use crate::{
    components::{back_link, page_head, rail_prose, rail_section, shell, video_card},
    content::interests::interest,
};

#[page("/interests/keys")]
async fn keys() -> Result {
    let meta = interest("keys");
    let prose = view! {
        <p>
            "The keyboard is a Dactyl Manuform from ohkeycaps — marble case, lubed 67g \
             Zilents, SA keycaps. I made a showcase video in 2021 and ten thousand \
             switch-curious strangers have watched it since, which makes it my most \
             successful publication in any medium."
        </p>
        <p>
            "The typing speed is real and independently auditable: 117wpm average, 165 peak."
        </p>
    }?;
    let footage = view! {
        <div class="flex flex-wrap gap-5">
            video_card(youtube_id: "yZl30vWuERs", label: "the keyboard →")
        </div>
    }?;
    let links = view! {
        <p class="flex flex-wrap gap-x-4 gap-y-1 font-meta text-sm">
            <a
                class="oxlink"
                href="https://data.typeracer.com/pit/profile?user=rivertam"
            >"TypeRacer →"</a>
        </p>
    }?;
    let body = view! {
        page_head(stamp: meta.slug, title: meta.title, lede: meta.teaser)
        rail_prose(mt: "mt-10", stamp: "", body: prose)
        rail_section(mt: "mt-10", stamp: "footage", body: footage)
        rail_section(mt: "mt-6", stamp: "links", body: links)
        back_link(href: "/interests", label: "← all interests")
    }?;
    view! { shell(title: meta.title, active: "interests", body: body) }
}
