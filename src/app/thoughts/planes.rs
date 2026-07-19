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
    design::shell,
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
            title = "How bad are planes? — Ben Berman".to_string();
            None
        }
    };

    let body = view! {
        <article class="mt-16 sm:mt-24">
            <header class="rail-row">
                <p class="rail-stamp">"2026-07-12"</p>
                <div class="min-w-0">
                    <h1 class="font-display text-4xl font-bold tracking-tight">"How bad are planes?"</h1>
                    <p class="mt-4 max-w-prose text-ink2">
                        "This piece started life as a standalone site — "
                        <a class="oxlink" href="https://howbad.org">"howbad.org"</a>
                        ", a revival of Shame Plane — and now lives here as a post. \
                         The form below prices one seat's share of a flight's real \
                         climate bill, computed on this very server."
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
    }?;
    view! { shell(title: title.as_str(), active: "", body: body) }
}
