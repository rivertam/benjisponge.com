use topcoat::{Result, router::page, view::view};

use crate::design::{page_head, shell};

#[page("/interests/keys")]
async fn keys() -> Result {
    let body = view! {
        page_head(
            stamp: "keys",
            title: "Keyboards",
            lede: "Split-columnar keyboard person. Ten thousand strangers have watched my Dactyl Manuform video; TypeRacer has me at 117wpm.",
        )
        <section class="rail-row mt-10">
            <div></div>
            <div class="min-w-0 max-w-prose space-y-4 text-ink2">
                <p>
                    "The keyboard is a Dactyl Manuform from ohkeycaps — marble case, lubed 67g \
                     Zilents, SA keycaps. I made a showcase video in 2021 and ten thousand \
                     switch-curious strangers have watched it since, which makes it my most \
                     successful publication in any medium."
                </p>
                <p>
                    "The typing speed is real and independently auditable: 117wpm average, 165 peak."
                </p>
            </div>
        </section>
        <div class="rail-row mt-10">
            <p class="rail-stamp uppercase tracking-[0.18em]">"footage"</p>
            <div class="flex min-w-0 flex-wrap gap-5">
                <a class="video-card" href="https://www.youtube.com/watch?v=yZl30vWuERs">
                    <img
                        src="https://img.youtube.com/vi/yZl30vWuERs/mqdefault.jpg"
                        alt="the keyboard →"
                        loading="lazy"
                    >
                    <span class="video-card-label font-meta text-sm text-ink2">
                        "the keyboard →"
                    </span>
                </a>
            </div>
        </div>
        <div class="rail-row mt-6">
            <p class="rail-stamp uppercase tracking-[0.18em]">"links"</p>
            <p class="flex min-w-0 flex-wrap gap-x-4 gap-y-1 font-meta text-sm">
                <a
                    class="oxlink"
                    href="https://data.typeracer.com/pit/profile?user=rivertam"
                >"TypeRacer →"</a>
            </p>
        </div>
        <div class="rail-row mt-14">
            <div></div>
            <p class="min-w-0 font-meta text-sm">
                <a class="quiet-link" href="/interests">"← all interests"</a>
            </p>
        </div>
    }?;
    view! { shell(title: "Keyboards — Ben Berman", body: body) }
}
