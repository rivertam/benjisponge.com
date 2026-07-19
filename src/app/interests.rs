//! The interests index. Each linked interest is a standalone page and route
//! in its own module below `app/interests/`.

mod drums;
mod felix;
mod keys;
mod lifting;
mod models;
mod puzzles;
mod spire;
mod swing;

use topcoat::{Result, router::page, view::view};

use crate::design::{page_head, shell};

#[page("/interests")]
async fn interests() -> Result {
    let body = view! {
        page_head(
            stamp: "index",
            title: "Interests",
            lede: "Skills of no professional value whatsoever. Everything here is, regrettably, public record.",
        )
        <section class="mt-14 space-y-10">
            <article class="rail-row">
                <p class="rail-stamp">"drums"</p>
                <div class="min-w-0">
                    <h2 class="font-display text-2xl leading-snug font-semibold">
                        <a class="oxlink" href="/interests/drums">"Drums"</a>
                    </h2>
                    <p class="mt-1.5 max-w-prose text-ink2">
                        "Mediocre drummer. Recording turns out to be much harder than playing."
                    </p>
                </div>
            </article>
            <article class="rail-row">
                <p class="rail-stamp">"swing"</p>
                <div class="min-w-0">
                    <h2 class="font-display text-2xl leading-snug font-semibold">
                        <a class="oxlink" href="/interests/swing">"Swing dancing"</a>
                    </h2>
                    <p class="mt-1.5 max-w-prose text-ink2">
                        "Swing dancing (lead and follow but mostly lead)"
                    </p>
                </div>
            </article>
            <article class="rail-row">
                <p class="rail-stamp">"lifting"</p>
                <div class="min-w-0">
                    <h2 class="font-display text-2xl leading-snug font-semibold">
                        <a class="oxlink" href="/interests/lifting">"Lifting"</a>
                    </h2>
                    <p class="mt-1.5 max-w-prose text-ink2">
                        "Deadlift PR 345 lbs, Squat PR 235 lbs, Bench PR like 165 but I never 1RM it"
                    </p>
                </div>
            </article>
            <article class="rail-row">
                <p class="rail-stamp">"keys"</p>
                <div class="min-w-0">
                    <h2 class="font-display text-2xl leading-snug font-semibold">
                        <a class="oxlink" href="/interests/keys">"Keyboards"</a>
                    </h2>
                    <p class="mt-1.5 max-w-prose text-ink2">
                        "Split-columnar keyboard person. Ten thousand strangers have watched my Dactyl Manuform video; TypeRacer has me at 117wpm."
                    </p>
                </div>
            </article>
            <article class="rail-row">
                <p class="rail-stamp">"spire"</p>
                <div class="min-w-0">
                    <h2 class="font-display text-2xl leading-snug font-semibold">
                        <a class="oxlink" href="/interests/spire">"Slay the Spire"</a>
                    </h2>
                    <p class="mt-1.5 max-w-prose text-ink2">
                        "Slay the Spire at Ascension 20, with an annotated run synopsis, because a win nobody can audit barely counts."
                    </p>
                </div>
            </article>
            <article class="rail-row">
                <p class="rail-stamp">"models"</p>
                <div class="min-w-0">
                    <h2 class="font-display text-2xl leading-snug font-semibold">
                        <a class="oxlink" href="/interests/models">"Toy models"</a>
                    </h2>
                    <p class="mt-1.5 max-w-prose text-ink2">
                        "Procedural cities with opinionated residents — a react-three-fiber toy running Schelling-style agents."
                    </p>
                </div>
            </article>
            <article class="rail-row">
                <p class="rail-stamp">"puzzles"</p>
                <div class="min-w-0">
                    <h2 class="font-display text-2xl leading-snug font-semibold">
                        <a class="oxlink" href="/interests/puzzles">"Crosswords"</a>
                    </h2>
                    <p class="mt-1.5 max-w-prose text-ink2">
                        "A Rust crossword engine, so .puz files open in the terminal. Nobody had asked for this."
                    </p>
                </div>
            </article>
            <article class="rail-row">
                <p class="rail-stamp">"felix"</p>
                <div class="min-w-0">
                    <h2 class="font-display text-2xl leading-snug font-semibold">
                        <a class="oxlink" href="/interests/felix">"Felix"</a>
                    </h2>
                    <p class="mt-1.5 max-w-prose text-ink2">
                        "There is a dog. There is, accordingly, a website computing when we are the same age."
                    </p>
                </div>
            </article>
        </section>
    }?;
    view! { shell(title: "Interests — Ben Berman", body: body) }
}
