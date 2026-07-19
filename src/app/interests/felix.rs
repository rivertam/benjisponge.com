use topcoat::{
    Result,
    asset::{Asset, asset},
    router::{page, redirect_permanent, route},
    view::view,
};

use crate::{
    components::{back_link, link_label, rail_prose, rail_section, shell},
    content::interests::interest,
};

struct Photo {
    src: Asset,
    alt: &'static str,
    stamp: &'static str,
    caption: &'static str,
    width: u16,
    height: u16,
}

const STUDIO_PORTRAIT: Asset = asset!("./felix/photos/2023-portrait.webp");
const FELIX_HOME_JS: Asset = asset!("./felix/felix-home.js");

const PHOTOS: &[Photo] = &[
    Photo {
        src: asset!("./felix/photos/2022-attention.webp"),
        alt: "Felix sitting attentively on a hardwood floor and looking up.",
        stamp: "nov 2022",
        caption: "Patiently awaiting the next instruction.",
        width: 1200,
        height: 1600,
    },
    Photo {
        src: asset!("./felix/photos/2022-walk-ready.webp"),
        alt: "Felix indoors wearing a gray coat, scarf, and four small boots.",
        stamp: "dec 2022",
        caption: "Coat, scarf, boots: ready.",
        width: 1200,
        height: 1600,
    },
    Photo {
        src: asset!("./felix/photos/2023-overlook.webp"),
        alt: "Felix beside a black dog at a mountain overlook.",
        stamp: "aug 2023",
        caption: "An overlook with a friend.",
        width: 1200,
        height: 1600,
    },
    Photo {
        src: asset!("./felix/photos/2023-play-bow.webp"),
        alt: "Felix bowing playfully on a grassy patio.",
        stamp: "2023",
        caption: "A play bow, the formal invitation.",
        width: 1600,
        height: 1200,
    },
    Photo {
        src: asset!("./felix/photos/2023-sprint.webp"),
        alt: "Felix running toward the camera through an autumn park.",
        stamp: "oct 2023",
        caption: "All speed, no plan.",
        width: 800,
        height: 1200,
    },
    Photo {
        src: asset!("./felix/photos/2024-big-smile.webp"),
        alt: "Felix sitting on a dark tiled floor with a wide open-mouthed smile.",
        stamp: "may 2024",
        caption: "An extremely cheerful hello.",
        width: 1200,
        height: 1600,
    },
    Photo {
        src: asset!("./felix/photos/2024-poolside.webp"),
        alt: "Felix standing beside a small pool and a deck.",
        stamp: "nov 2024",
        caption: "Poolside inspection.",
        width: 1312,
        height: 1600,
    },
    Photo {
        src: asset!("./felix/photos/2025-rug.webp"),
        alt: "Felix sitting on a rug, looking up at the camera with his tongue out.",
        stamp: "mar 2025",
        caption: "Photo day on the rug.",
        width: 1083,
        height: 1600,
    },
    Photo {
        src: asset!("./felix/photos/2025-chair.webp"),
        alt: "Felix sitting happily in a large dark armchair.",
        stamp: "apr 2025",
        caption: "The chair's current occupant.",
        width: 1200,
        height: 1600,
    },
    Photo {
        src: asset!("./felix/photos/2025-waterfront.webp"),
        alt: "Felix sitting on a picnic table beside a waterfront city skyline.",
        stamp: "apr 2025",
        caption: "Waterfront duty.",
        width: 1200,
        height: 1600,
    },
    Photo {
        src: asset!("./felix/photos/felix-close-up.webp"),
        alt: "Close-up side view of Felix's nose as a hand reaches toward it.",
        stamp: "undated",
        caption: "An unreasonable amount of snoot.",
        width: 810,
        height: 540,
    },
];

#[page("/felix")]
async fn felix() -> Result {
    let meta = interest("felix");
    view! { shell(title: meta.title, active: "interests", hide_nav: true,
        <header class="felix-hero" aria-labelledby="felix-title">
            <img
                class="felix-hero-media"
                src=(STUDIO_PORTRAIT)
                alt="Black-and-white studio portrait of Felix against a dark background."
                width="1000"
                height="667"
                decoding="async"
                fetchpriority="high"
            >
            <div class="felix-hero-copy">
                <p class="felix-hero-stamp">"felix / oct 2023"</p>
                <h1 id="felix-title" class="felix-hero-title">(meta.title)</h1>
                <p class="felix-hero-lede">"Carolina Dog"</p>
                <a class="felix-hero-scroll" href="#felix-notes">
                    "meet Felix "
                    <span aria-hidden="true">"↓"</span>
                </a>
            </div>
        </header>
        <a class="felix-home" href="/" data-felix-home="true">
            <span class="link-arrow link-arrow-before" aria-hidden="true">"<-"</span>
            " Ben"
        </a>
        <script type="module" src=(FELIX_HOME_JS)></script>
        <div id="felix-notes">
            rail_prose(class: "mt-12", stamp: "about",
                <p>
                    "The dog is Felix. The website is saamd.com — Same Age As My Dog — which \
                     computes the one day on which a dog and its person are, in dog years, the \
                     same age."
                </p>
                <p>
                    "This date matters to no one, which is why it needed a calculator."
                </p>
            )
        </div>
        rail_section(class: "mt-8", stamp: "photos",
            <div class="felix-gallery" aria-label="Photos of Felix">
                for photo in PHOTOS {
                    <figure class="felix-photo">
                        <img
                            src=(photo.src)
                            alt=(photo.alt)
                            width=(photo.width)
                            height=(photo.height)
                            loading="lazy"
                            decoding="async"
                        >
                        <figcaption>
                            <span class="felix-photo-stamp">(photo.stamp)</span>
                            <span>(photo.caption)</span>
                        </figcaption>
                    </figure>
                }
            </div>
        )
        rail_section(class: "mt-6", stamp: "links",
            <p class="flex flex-wrap gap-x-4 gap-y-1 font-meta text-sm">
                <a class="oxlink" href="https://www.saamd.com">
                    link_label(label: "saamd.com →")
                </a>
            </p>
        )
        back_link(href: "/interests", label: "all interests")
    ) }
}

#[route(GET "/interests/felix")]
async fn legacy_felix() -> Result {
    Err(redirect_permanent("/felix").into())
}
