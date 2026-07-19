//! The migrated flight page from ~/how-bad, living on as a blog post. The
//! page-level composition mirrors `App.tsx`: resolve the route from the query
//! string, compute the impact server-side, and render form → charts → budget
//! → receipt → sources. URLs stay shareable exactly like the original.

mod charts;
mod form;
mod ice;
mod receipt;

use topcoat::{
    Result,
    context::Cx,
    router::{page, query_params},
    view::view,
};

use crate::{
    components::{ext_link, inline_popover, shell},
    flight::{
        airports::{Airport, find_airport},
        emissions::{Cabin, FlightInput, flight_impact},
        format::format_tonnes,
        sources::sources_section,
    },
};

use form::flight_form;

#[query_params(error = redirect("?"))]
struct PlanesQuery {
    from: Option<String>,
    to: Option<String>,
    cabin: Option<String>,
    oneway: Option<String>,
    trip: Option<String>,
    view: Option<String>,
}

fn resolve(param: Option<&str>) -> Option<&'static Airport> {
    find_airport(param?.trim())
}

#[page("/thoughts/how-bad-are-planes")]
async fn planes(cx: &Cx) -> Result {
    let q = query_params::<PlanesQuery>(cx)?;

    let from = resolve(q.from.as_deref());
    let to = resolve(q.to.as_deref());
    let cabin = q
        .cabin
        .as_deref()
        .and_then(Cabin::parse)
        .unwrap_or(Cabin::Economy);
    // The original marked one-way with the mere presence of `oneway` (any
    // value); the form's trip radios say `trip=oneway`. Accept both so old
    // share URLs keep working.
    let round_trip = !(q.oneway.is_some() || q.trip.as_deref() == Some("oneway"));
    let chart_view = if q.view.as_deref() == Some("compare") {
        "compare"
    } else {
        "cuts"
    };
    let revealed = from.is_some() && to.is_some();

    let title;
    let revealed_part = match (from, to) {
        (Some(from), Some(to)) => {
            let input = FlightInput {
                from: from.coordinates(),
                to: to.coordinates(),
                cabin,
                round_trip,
            };
            let impact = flight_impact(&input);
            // Every cabin priced over the same itinerary — App.tsx's `cabinYears`.
            let years =
                |cabin: Cabin| flight_impact(&FlightInput { cabin, ..input }).travel_budget_years;

            // The share query mirrors the original's `routeSearchParams`:
            // defaults (economy, round trip) are omitted, chart view never rides along.
            let mut share_path = format!(
                "/thoughts/how-bad-are-planes?from={}&to={}",
                from.iata, to.iata
            );
            if cabin != Cabin::Economy {
                share_path.push_str(&format!("&cabin={}", cabin.as_str()));
            }
            if !round_trip {
                share_path.push_str("&trip=oneway");
            }

            title = format!(
                "{} {} {} · {} CO₂e — How bad are planes?",
                from.iata,
                if round_trip { "⇄" } else { "→" },
                to.iata,
                format_tonnes(impact.tonnes_co2e),
            );

            Some(view! {
                charts::charts_section(
                    impact: impact,
                    round_trip: round_trip,
                    to_city: to.city.clone(),
                    initial_view: chart_view.to_string(),
                )
                charts::budget_chart(
                    flight_tonnes: impact.tonnes_co2e,
                    economy_years: years(Cabin::Economy),
                    business_years: years(Cabin::Business),
                    first_years: years(Cabin::First),
                )
                receipt::receipt(
                    from: from.clone(),
                    to: to.clone(),
                    cabin: cabin,
                    round_trip: round_trip,
                    impact: impact,
                    share_path: share_path,
                )
                sources_section()
            }?)
        }
        _ => {
            title = "How bad are planes?".to_string();
            None
        }
    };

    view! { shell(title: title.as_str(), active: "",
        <article class="mt-16 sm:mt-24">
            <header class="rail-row">
                <p class="rail-stamp">"2026-07-12"</p>
                <div class="min-w-0">
                    <h1 class="font-display text-4xl font-bold tracking-tight">"How bad are planes?"</h1>
                    <p class="mt-4 max-w-prose text-ink2">
                        "In 2019, I read "
                        inline_popover(
                            id: "planet-b-cite",
                            label: "There Is No Planet B",
                            <span class="inline-popover-preview">
                                "Mike Berners-Lee’s 2019 handbook on climate priorities — where flying \
                                 lands among the high-impact personal choices."
                            </span>
                            ext_link(
                                class: "quiet-link",
                                href: "https://theresnoplanetb.net/",
                                label: "theresnoplanetb.net →"
                            )
                        )
                        " by Mike Berners-Lee a couple days before a trip I took to Asheville, \
                         North Carolina to visit my mom. I learned not just that planes were bad \
                         for the environment, but the magnitude."
                    </p>
                    <p class="mt-4 max-w-prose text-ink2">
                        "One of my favorite philosophies in life is the "
                        inline_popover(
                            id: "pareto-cite",
                            label: "Pareto Principle",
                            <span class="inline-popover-preview">
                                "Also called the 80/20 rule: a small share of causes often drives \
                                 most of the effect. Named for Vilfredo Pareto’s observation about \
                                 wealth concentration."
                            </span>
                            ext_link(
                                class: "quiet-link",
                                href: "https://en.wikipedia.org/wiki/Pareto_principle",
                                label: "Wikipedia →"
                            )
                        )
                        ": don't waste all your time and effort on the minutiae. Find the points \
                         of highest impact. What I discovered is that, among the people I know and \
                         myself historically, flying planes eclipses almost all of our other habits. \
                         I would say for most people I know, four domestic flights (round trip) \
                         each year is quite typical, with international trips at least once every \
                         2-3 years. I strongly encourage you to play with the calculator below to \
                         see why I think most flights are simply not worth it."
                    </p>

                    <p class="mt-4 max-w-prose text-ink2">
                        "I also know people who have essentially never been on planes. Most of them \
                         simply can't afford it, often to the extent that they haven't even considered \
                         traveling for leisure."
                    </p>

                    <p class="mt-4 max-w-prose text-ink2">
                        "Commercial consumer flying accounts for "
                        inline_popover(
                            id: "aviation-share-cite",
                            label: "about 2% of the world's CO₂",
                            <span class="inline-popover-preview">
                                "All aviation is ~2.5% of global CO₂ (fossil + land use). With \
                                 non-CO₂ effects — mainly contrail cirrus — it’s ~3.5% of warming \
                                 to date. Of that CO₂, ~88% is commercial, ~8% military, and ~4% \
                                 private; within commercial, ~81% is passengers and ~19% freight. \
                                 Passenger flying alone is therefore ~2% of global CO₂."
                            </span>
                            ext_link(
                                class: "quiet-link",
                                href: "https://ourworldindata.org/global-aviation-emissions",
                                label: "Our World in Data →"
                            )
                            ext_link(
                                class: "quiet-link",
                                href: "https://doi.org/10.1016/j.gloenvcha.2020.102194",
                                label: "Gössling & Humpe 2020 →"
                            )
                        )
                        " despite benefiting a "
                        inline_popover(
                            id: "who-flies-cite",
                            label: "sliver of the population",
                            <span class="inline-popover-preview">
                                "Gössling & Humpe (2020): about 11% of the world flew in 2018, at \
                                 most 4% internationally. The most frequent 1% of people account \
                                 for more than half of passenger-aviation CO₂."
                            </span>
                            ext_link(
                                class: "quiet-link",
                                href: "https://doi.org/10.1016/j.gloenvcha.2020.102194",
                                label: "Gössling & Humpe 2020 →"
                            )
                        )
                        ". A huge portion of the global south have either never flown on a plane \
                         or it's a very rare, very expensive privilege used for special circumstances \
                         such as migrating between countries. Flying and travel to this extent has \
                         only been in human lives for the last "
                        inline_popover(
                            id: "jet-age-cite",
                            label: "~80 years",
                            <span class="inline-popover-preview">
                                "Mass commercial jet travel starts in the 1950s (Comet, then the \
                                 Boeing 707). Aviation CO₂ has roughly quadrupled since the \
                                 mid-1960s, and its share of global emissions is still rising."
                            </span>
                            ext_link(
                                class: "quiet-link",
                                href: "https://ourworldindata.org/global-aviation-emissions",
                                label: "Our World in Data →"
                            )
                        )
                        ", and it's ramped up significantly and isn't stopping. Our \
                         great-great-grandparents had to take ships to get across the Atlantic Ocean."
                    </p>

                    <p class="mt-4 max-w-prose text-ink2">
                        "Despite the negative impacts, I've been told that the positives are enormous. \
                         Cultural diffusion, global collaboration, world peace. These are each true to \
                         quite a good extent. My purpose in this page isn't to argue against this, it's \
                         just to point out that if you want to show "
                        <pre>"Pros(commercial travel) > Cons(commercial travel)"</pre>
                        ", let's at least start by having a good perception of "
                        <pre>"Cons(commercial travel)"</pre>
                        "."
                    </p>
                </div>
            </header>
            <div class="paper-warm mt-10">
                flight_form(
                    from: from.cloned(),
                    to: to.cloned(),
                    cabin: cabin,
                    round_trip: round_trip,
                    revealed: revealed,
                )
                if let Some(part) = revealed_part {
                    (part)
                }
            </div>
        </article>
    ) }
}
