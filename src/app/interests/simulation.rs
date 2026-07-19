use topcoat::{
    Result,
    router::page,
    view::{component, view},
};

use crate::{
    content::interests::interest,
    design::{inline_popover, page_head, shell},
};

#[component]
async fn map_generator_citation() -> Result {
    view! {
        <span class="inline-popover-preview">
            "A procedural city map generator, created for artistic purposes."
        </span>
        <a
            class="quiet-link"
            href="https://github.com/ProbableTrain/MapGenerator"
            target="_blank"
            rel="noopener noreferrer"
        >"repo →"</a>
    }
}

#[component]
async fn ecs_citation() -> Result {
    view! {
        <span class="inline-popover-preview">
            "Entity–component system: simulation state as entities with typed component data."
        </span>
        <span class="inline-popover-detail">
            "Properties of entities are typically stored in big, co-located arrays, allowing \
             disparate systems to run independently (parallelized and cache-optimal). \
             Effectively an authoring layer to give struct-of-arrays performance with \
             array-of-structs semantics."
        </span>
        <a
            class="quiet-link"
            href="https://en.wikipedia.org/wiki/Entity_component_system"
            target="_blank"
            rel="noopener noreferrer"
        >"Wikipedia →"</a>
    }
}

#[component]
async fn react_three_fiber_citation() -> Result {
    view! {
        <span class="inline-popover-preview">
            "React renderer for three.js — how the simulation was meant to be visualized."
        </span>
        <a
            class="quiet-link"
            href="https://docs.pmnd.rs/react-three-fiber/getting-started/introduction"
            target="_blank"
            rel="noopener noreferrer"
        >"docs →"</a>
    }
}

#[page("/interests/simulation")]
async fn simulation() -> Result {
    let meta = interest("simulation");
    let title = format!("{} — Ben Berman", meta.title);
    let chaos_citation = view! {
        <p>
            <cite>
                "Gleick, James. "
                <em>"Chaos: Making a New Science"</em>
                ". Viking, 1987."
            </cite>
        </p>
        <a
            class="quiet-link"
            href="https://www.penguinrandomhouse.com/books/321477/chaos-by-james-gleick/"
            target="_blank"
            rel="noopener noreferrer"
        >"publisher →"</a>
    }?;
    let body = view! {
        page_head(stamp: meta.slug, title: meta.title, lede: meta.teaser)
        <section class="rail-row mt-10">
            <p class="rail-stamp uppercase tracking-[0.18em]">"backstory"</p>
            <div class="min-w-0 max-w-prose space-y-4 text-ink2">
                <p>
                    "In 2023, I got into a little debate with my brother. I had said something \
                     like \"The existence and legality of renting property (i.e. landlords) \
                     raises property values\". He said something like \"You're way too sure, \
                     given the potential second- and third-order effects.\""
                </p>
                <p>
                    "I had this thought of \"this would be easy to simulate\", which I have \
                     learned is a dumb, untrue thought on both a literal and philosophical \
                     level."
                </p>
                <p>
                    "I explored many possibilities, forked from "
                    inline_popover(
                        id: "map-generator-cite",
                        label: "ProbablyTrain/MapGenerator",
                        map_generator_citation()
                    )
                    ", and played around with agentic LLMs for development for the first time.
                    I learned a bit about NetLogo and other \"state of the art\" agent-based
                    simulation systems. I did some unfinished philosophy on a good authoring
                    surface. Basically the idea was \"NetLogo but modern and on the web\""
                </p>
                <p>
                    "One of the biggest through-lines here was to create an "
                    inline_popover(
                        id: "ecs-cite",
                        label: "ECS",
                        ecs_citation()
                    )
                    "-style state management system for React (and therefore "
                    inline_popover(
                        id: "react-three-fiber-cite",
                        label: "react-three-fiber",
                        react_three_fiber_citation()
                    )
                    ") with co-located memory supporting a many-entity simulation. One of the \
                     cool ideas behind making it ECS-style would be the potential to use \
                     workers to do (" <em>"gasp"</em> ") multi-threaded simulations/gaming in \
                     JavaScript."
                </p>

            </div>
        </section>
        <div class="rail-row mt-10">
            <p class="rail-stamp uppercase tracking-[0.18em]">"footage"</p>
            <div class="flex min-w-0 flex-wrap gap-5">
                <a class="video-card" href="https://www.youtube.com/watch?v=Bcd_9LvUr-8">
                    <img
                        src="https://img.youtube.com/vi/Bcd_9LvUr-8/mqdefault.jpg"
                        alt="the video →"
                        loading="lazy"
                    >
                    <span class="video-card-label font-meta text-sm text-ink2">
                        "the video →"
                    </span>
                </a>
            </div>
        </div>
        <div class="rail-row mt-6">
            <p class="rail-stamp uppercase tracking-[0.18em]">"links"</p>
            <p class="flex min-w-0 flex-wrap gap-x-4 gap-y-1 font-meta text-sm">
                <a class="oxlink" href="https://github.com/rivertam/City">"the repo →"</a>
            </p>
        </div>
        <div class="rail-row mt-6">
            <p class="rail-stamp uppercase tracking-[0.18em]">"What's the state?"</p>
            <p class="flex min-w-0 flex-wrap gap-x-4 gap-y-1 font-meta text-sm">
                "Abandoned, in part due to finding a job."
            </p>
        </div>
        <div class="rail-row mt-6">
            <p class="rail-stamp uppercase tracking-[0.18em]">"What did I learn?"</p>
            <ul class="flex min-w-0 flex-wrap gap-x-4 gap-y-1 font-meta text-sm">
                <li>"Memory-oriented performance improvements in the JavaScript runtime are quite counter-intuitive."</li>
                <li>
                    "Chaotic systems aren't well-represented by discrete data and decisions such as actors. Shout out to "
                    inline_popover(
                        id: "chaos-book-cite",
                        label: "Chaos by James Gleick",
                        (chaos_citation)
                    )
                    "."
                </li>
            </ul>
        </div>


        <div class="rail-row mt-14">
            <div></div>
            <p class="min-w-0 font-meta text-sm">
                <a class="quiet-link" href="/interests">"← all interests"</a>
            </p>
        </div>
    }?;
    view! { shell(title: title.as_str(), active: "interests", body: body) }
}
