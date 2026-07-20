//! The flight page as a dispatch desk: a full-bleed working surface where
//! the route form is a flight-progress strip, the charts/receipt are the
//! climate paperwork the desk returns, and the essay rides alongside as the
//! dispatcher's margin notes. The page-level composition still mirrors the
//! original `App.tsx` data flow: resolve the route from the query string,
//! compute the impact server-side, and render form → charts (cuts / compare /
//! receipt / allowance tabs) → sources. URLs stay shareable exactly like the
//! original.

mod airports;
mod charts;
mod comparison_scale;
mod emissions;
mod form;
mod format;
mod ice;
mod instruments;
mod receipt;
mod reference_data;
mod sources;

use topcoat::{
    Result,
    context::Cx,
    router::{page, query_params},
    view::view,
};

use crate::components::{
    back_link, doc_head, ext_link, full_bleed, inline_popover, margin_notes, shell,
};

use self::{
    airports::{Airport, find_airport},
    emissions::{Cabin, FlightInput, flight_impact},
    format::{format_km, format_tonnes},
    sources::sources_section,
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
    let chart_view = match q.view.as_deref() {
        Some("compare") => "compare",
        Some("receipt") => "receipt",
        Some("allowance") => "allowance",
        _ => "cuts",
    };
    let revealed = from.is_some() && to.is_some();

    let title;
    let mut seal_total = String::new();
    // The route card rides the top of the margin column, above the notes —
    // outside the dispatch pane, so it's built alongside revealed_part.
    let mut route_card = None;
    let revealed_part = match (from, to) {
        (Some(from), Some(to)) => {
            let input = FlightInput {
                from: from.coordinates(),
                to: to.coordinates(),
                cabin,
                round_trip,
            };
            let impact = flight_impact(&input);
            let legs_km = impact.distance_km * if round_trip { 2.0 } else { 1.0 };
            route_card = Some(view! {
                <div class="dispatch desk-route">
                    instruments::route_figure(
                        from_iata: from.iata.clone(),
                        from_lat: from.lat,
                        from_lon: from.lon,
                        to_iata: to.iata.clone(),
                        to_lat: to.lat,
                        to_lon: to.lon,
                        round_trip: round_trip,
                        km_flown: format_km(legs_km),
                    )
                </div>
            }?);

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

            seal_total = format_tonnes(impact.tonnes_co2e);
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
                    from: from.clone(),
                    to: to.clone(),
                    cabin: cabin,
                    share_path: share_path,
                    initial_view: chart_view.to_string(),
                )
                sources_section()
            }?)
        }
        _ => {
            title = "How bad are planes?".to_string();
            None
        }
    };

    let spread_class = if revealed {
        "desk-spread desk-spread--filed"
    } else {
        "desk-spread"
    };

    view! { shell(title: title.as_str(), active: "", hide_nav: true,
        <article>
            full_bleed(class: "desk-band",
                <div class=(spread_class)>
                    doc_head(stamp: "2026-07-12", title: "How bad are planes?")
                    <div class="dispatch">
                        flight_form(
                            from: from.cloned(),
                            to: to.cloned(),
                            cabin: cabin,
                            round_trip: round_trip,
                            revealed: revealed,
                            total: seal_total,
                        )
                        if !revealed {
                            <div class="strip-ghosts" aria-hidden="true">
                                <div></div>
                                <div></div>
                                <div></div>
                            </div>
                        }
                        if let Some(part) = revealed_part {
                            (part)
                        }
                    </div>
                    if let Some(card) = route_card {
                        (card)
                    }
                    margin_notes(stamp: "",
                        <p>
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
                        <p>
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

                        <p>
                            "I also know people who have essentially never been on planes. Most of them \
                             simply can't afford it, often to the extent that they haven't even considered \
                             traveling for leisure."
                        </p>

                        <p>
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

                        <p>
                            "The upsides are real: cultural diffusion, collaboration across borders, a \
                             smaller world. I'm not here to deny them. If you want to argue that the pros \
                             of commercial travel outweigh the cons, fine — but let's at least get the \
                             cons right first."
                        </p>
                    )
                </div>
            )
            back_link(href: "/thoughts", label: "all thoughts")
        </article>
    ) }
}
