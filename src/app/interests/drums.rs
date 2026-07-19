use topcoat::{
    Result,
    router::page,
    view::{component, view},
};

use crate::{
    components::{
        back_link, ext_link, inline_popover, page_head, rail_prose, rail_section, shell, video_card,
    },
    content::interests::interest,
};

#[component]
async fn island_lake_citation() -> Result {
    view! {
        <span class="inline-popover-preview">
            "Summer arts camp in New Hampshire — where I took my first drum lesson."
        </span>
        ext_link(
            class: "quiet-link",
            href: "https://www.islandlake.com/",
            label: "islandlake.com →"
        )
    }
}

#[page("/interests/drums")]
async fn drums() -> Result {
    let meta = interest("drums");
    let prose = view! {
        <p>
            "Technically I've been playing the drums for like 15 years! Weird how bad I still am lol"
        </p>
    }?;
    let footage = view! {
        <div class="flex flex-wrap gap-5">
            video_card(youtube_id: "VaKI7J2M2Ms", label: "Taylor Swift cover →")
            video_card(youtube_id: "8lrjsP1KWrY", label: "Manchester Orchestra cover →")
        </div>
    }?;
    let body = view! {
        page_head(stamp: meta.slug, title: meta.title, lede: meta.teaser)
        rail_prose(stamp: "", body: prose)
        rail_section(stamp: "footage", body: footage)
        rail_section(stamp: "2006", body: view! {
            <div>
                "Took my first drum lesson at "
                inline_popover(
                    id: "island-lake-cite",
                    label: "Island Lake",
                    island_lake_citation()
                )
            </div>
        }?)
        rail_section(stamp: "2008", body: view! {
            <div>
                "Ignored a drum instructor who patiently tried to teach me rudiments for several months"
            </div>
        }?)
        rail_section(stamp: "2008", body: view! {
            <div>
                "Ignored a drum instructor who patiently tried to teach me rudiments for several months"
            </div>
        }?)

        back_link(href: "/interests", label: "← all interests")
    }?;
    view! { shell(title: meta.title, active: "interests", body: body) }
}
