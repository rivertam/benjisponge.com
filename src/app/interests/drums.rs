use topcoat::{Result, router::page, view::view};

use crate::{
    components::{back_link, page_head, rail_prose, rail_section, shell, video_card},
    content::interests::interest,
};

#[page("/interests/drums")]
async fn drums() -> Result {
    let meta = interest("drums");
    let prose = view! {
        <p>
            "I had my first lesson at summer camp, which is to say I've been playing long \
             enough that I should be better. These days it's a Tama pancake kit — quiet \
             enough for an apartment, portable enough that I have set it up in a park."
        </p>
        <p>"Two covers survive public scrutiny."</p>
    }?;
    let footage = view! {
        <div class="flex flex-wrap gap-5">
            video_card(youtube_id: "VaKI7J2M2Ms", label: "Taylor Swift cover →")
            video_card(youtube_id: "8lrjsP1KWrY", label: "Manchester Orchestra cover →")
        </div>
    }?;
    let body = view! {
        page_head(stamp: meta.slug, title: meta.title, lede: meta.teaser)
        rail_prose(mt: "mt-10", stamp: "", body: prose)
        rail_section(mt: "mt-10", stamp: "footage", body: footage)
        back_link(href: "/interests", label: "← all interests")
    }?;
    view! { shell(title: meta.title, active: "interests", body: body) }
}
