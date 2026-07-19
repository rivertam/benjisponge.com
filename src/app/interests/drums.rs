use topcoat::{Result, router::page, view::view};

use crate::{
    components::{back_link, ext_link, inline_popover, page_head, rail_section, shell, video_card},
    content::interests::interest,
};

#[page("/interests/drums")]
async fn drums() -> Result {
    let meta = interest("drums");
    let footage = view! {
        <div class="flex flex-wrap gap-5">
            video_card(youtube_id: "VaKI7J2M2Ms", label: "Taylor Swift cover →")
            video_card(youtube_id: "8lrjsP1KWrY", label: "Manchester Orchestra cover →")
        </div>
    }?;
    let body = view! {
        page_head(stamp: meta.slug, title: meta.title, lede: meta.teaser)
        rail_section(class: "mt-4", stamp: "footage", body: footage)
        rail_section(class: "mt-4", stamp: "2006", body: view! {
            <div>
                "Took my first drum lesson at "
                inline_popover(
                    id: "island-lake-cite",
                    label: "Island Lake",
                    <span class="inline-popover-preview">
                        "Summer camp in Pennsylvania. Other skills I learned include light mountain \
                         biking, the music from Newsies and Bat Boy, and how to play \
                         Warcraft III: Frozen Throne"
                    </span>
                    ext_link(
                        class: "quiet-link",
                        href: "https://www.islandlake.com/",
                        label: "islandlake.com →"
                    )
                )
            </div>
        }?)
        rail_section(class: "mt-4", stamp: "2007", body: view! {
            <div>
                "Ignored a drum instructor who patiently tried to teach me "
                inline_popover(
                    id: "rudiments-cite",
                    label: "rudiments",
                    <span class="inline-popover-preview">
                        "Standard sticking patterns — the vocabulary drum teachers start with."
                    </span>
                    ext_link(
                        class: "quiet-link",
                        href: "https://en.wikipedia.org/wiki/Drum_rudiment",
                        label: "Wikipedia →"
                    )
                )
                " for several months"
            </div>
        }?)
        rail_section(class: "mt-4", stamp: "2007-2010", body: view! {
            <div>
                "Played "
                inline_popover(
                    id: "rock-band-cite",
                    label: "Rock Band",
                    <span class="inline-popover-preview">
                        "Rhythm game series with plastic instrument controllers."
                    </span>
                    ext_link(
                        class: "quiet-link",
                        href: "https://www.youtube.com/watch?v=PFoizasD3Hc",
                        label: "What it looked like →"
                    )
                )
                " almost every day"
            </div>
        }?)
        rail_section(class: "mt-4", stamp: "2010", body: view! {
            <div>
                "Parents got me a "
                inline_popover(
                    id: "dtxplorer-cite",
                    label: "Yamaha DTXplorer",
                    <span class="inline-popover-preview">
                        "Yamaha's entry-level electronic kit and one of the reasons I actually don't \
                         recommend getting a cheap electric kit for kids if they haven't decided whether \
                         they like playing the drums yet (they will learn to dislike playing the drums \
                         because many practice sessions start with a debugging session)."
                    </span>
                    ext_link(
                        class: "quiet-link",
                        href: "https://usa.yamaha.com/products/musical_instruments/drums/el_drums/dtx/index.html",
                        label: "Yamaha DTX →"
                    )
                )
                " which I practiced with sometimes"
            </div>
        }?)
        rail_section(class: "mt-4", stamp: "2011", body: view! {
            <div>
                "Parents got me a used acoustic kit which I still have!"
            </div>
        }?)
        rail_section(class: "mt-4", stamp: "2016", body: view! {
            <div>
                "Started playing with some folks in Sunnyside"
            </div>
        }?)
        rail_section(class: "mt-4", stamp: "2016", body: view! {
            <div>
                "Bought myself an Alesis DM10X"
            </div>
        }?)
        rail_section(class: "mt-4", stamp: "2017", body: view! {
            <div>
                "Sold my Alesis DM10X to Guitar Center"
            </div>
        }?)

        rail_section(class: "mt-4", stamp: "2018-2019", body: view! {
            <div>
                "Got weirdly into custom electronic drums and converted my acoustic
                 kit to electronic using UFO Drums heads, Jobeky cymbals and sensors,
                 and a hodgepodge of modules including a MegaDrum module. Start playing
                 with some embedded software to analyze piezo sensors. Also became
                 fascinated by Sunhouse, the company behind Sensory Percussion.
                 Nothing really has come out of any of this (yet)."
            </div>
        }?)

        rail_section(class: "mt-4", stamp: "2020", body: view! {
            <div>
                "Bought myself an EFNote 5 (excellent decision)."
            </div>
        }?)

        rail_section(class: "mt-4", stamp: "2021", body: view! {
            <div>
                "Met a few folks I still play with (Justin and Sanket, mostly)."
            </div>
        }?)

        rail_section(class: "mt-4", stamp: "What I play", body: view! {
            <ul>
                <li>
                    inline_popover(
                        id: "efnote-cite",
                        label: "EFNote 5",
                        <span class="inline-popover-preview">
                            "Electronic drum kit that people look at and will be like \"what are you \
                             talking about, that's an acoustic kit\""
                        </span>
                        ext_link(
                            class: "quiet-link",
                            href: "https://www.efnotepro.com/products/efnote5",
                            label: "efnotepro.com →"
                        )
                    )
                </li>
                <li>
                    inline_popover(
                        id: "strike-multipad-cite",
                        label: "Alesis Strike MultiPad",
                        <span class="inline-popover-preview">
                            "Sample pad with built-in sounds"
                        </span>
                        ext_link(
                            class: "quiet-link",
                            href: "https://www.alesis.com/strike-multipad.html",
                            label: "alesis.com →"
                        )
                    )
                </li>
                <li>
                    inline_popover(
                        id: "club-jam-cite",
                        label: "Tama Club-Jam Pancake kit",
                        <span class="inline-popover-preview">
                            "Compact acoustic kit — 18\" bass, small toms, still my main acoustic set."
                        </span>
                        ext_link(
                            class: "quiet-link",
                            href: "https://www.tama.com/usa/products/drums/acoustic/club-jam/",
                            label: "tama.com →"
                        )
                    )
                </li>
            </ul>
        }?)
        back_link(href: "/interests", label: "← all interests")
    }?;
    view! { shell(title: meta.title, active: "interests", body: body) }
}
