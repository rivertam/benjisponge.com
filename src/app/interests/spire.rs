use topcoat::{Result, router::page, view::view};

use crate::{
    components::{back_link, page_head, rail_prose, rail_section, shell},
    content::interests::interest,
};

#[page("/interests/spire")]
async fn spire() -> Result {
    let meta = interest("spire");
    view! { shell(title: meta.title, active: "interests",
        page_head(stamp: meta.slug, title: meta.title, lede: meta.teaser)
        rail_prose(stamp: "",
            <p>
                "Ascension 20 is the highest difficulty the game offers. The writeup exists \
                 because the win deserved documentation more than it deserved celebration."
            </p>
            <p>
                "I also maintain opinions about the RNG that I can support with screenshots."
            </p>
        )
        rail_section(class: "mt-6", stamp: "links",
            <p class="flex flex-wrap gap-x-4 gap-y-1 font-meta text-sm">
                <a
                    class="oxlink"
                    href="https://reddit.com/r/slaythespire/comments/jkqx35/annotated_run_synopsis_my_second_a20_win_only_a/"
                >"the synopsis →"</a>
            </p>
        )
        back_link(href: "/interests", label: "← all interests")
    ) }
}
