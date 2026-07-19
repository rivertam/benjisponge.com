//! The flight form and airport combobox: ports of `FlightForm.tsx` and
//! `AirportCombobox.tsx` from ~/how-bad.
//!
//! The baseline is a plain GET form submitting to the page's own URL: the
//! route fields are text inputs named `from`/`to` holding IATA codes, so the
//! whole flow works with JavaScript disabled. The combobox is a client-side
//! enhancement (`airport-combobox.js`): it fetches the bundled airports
//! dataset once and runs the same search (prefix, fuzzy, metros, country)
//! in the browser — no shard round-trips.

use topcoat::{
    Result,
    asset::{Asset, asset},
    view::{component, view},
};

use crate::flight::{airports::Airport, emissions::Cabin};

const AIRPORT_COMBOBOX_JS: Asset = asset!("./airport-combobox.js");
const AIRPORTS_JSON: Asset = asset!("../../../../data/airports.json");

#[component]
pub async fn flight_form(
    from: Option<Airport>,
    to: Option<Airport>,
    cabin: Cabin,
    round_trip: bool,
    revealed: bool,
) -> Result {
    view! {
        <form
            class=(if revealed { "flight-form form-dock" } else { "flight-form" })
            data-airports-url=(AIRPORTS_JSON)
        >
            <header class=(if revealed { "form-head form-head--dock" } else { "form-head" })>
                <p class="eyebrow">
                    <a href="/thoughts">"thoughts"</a>
                    (if revealed { " · how bad are planes" } else { " / how bad are planes" })
                </p>
            </header>
            <div class="route-fields">
                airport_field(label: "From", name: "from", iata: from.map(|a| a.iata).unwrap_or_default())
                airport_field(label: "To", name: "to", iata: to.map(|a| a.iata).unwrap_or_default())
            </div>
            <div class="trip-options">
                <label>
                    <input type="radio" name="cabin" value="economy" checked=(cabin == Cabin::Economy)>
                    "Economy"
                </label>
                <label>
                    <input type="radio" name="cabin" value="business" checked=(cabin == Cabin::Business)>
                    "Business"
                </label>
                <label>
                    <input type="radio" name="cabin" value="first" checked=(cabin == Cabin::First)>
                    "First"
                </label>
                <label>
                    <input type="radio" name="trip" value="round" checked=(round_trip)>
                    "Round trip"
                </label>
                <label>
                    <input type="radio" name="trip" value="oneway" checked=(!round_trip)>
                    "One way"
                </label>
            </div>
            // Unlike the original (where React recomputes live and the button
            // disappears after reveal), the server round-trip always needs a
            // submit control, so the button stays — condensed by .form-dock.
            <button type="submit" class="print-btn">"See how it compares"</button>
        </form>
        <script type="module" src=(AIRPORT_COMBOBOX_JS)></script>
    }
}

/// One route endpoint. Plain text input for the GET baseline; the client
/// script upgrades `.combobox[data-airport-combobox]` into a typeahead.
#[component]
async fn airport_field(label: &str, name: &str, iata: String) -> Result {
    let input_id = format!("airport-{name}");
    view! {
        <div class="field">
            <label for=(input_id.as_str())>(label)</label>
            <div class="combobox" data-airport-combobox="">
                <input
                    id=(input_id.as_str())
                    type="text"
                    name=(name)
                    role="combobox"
                    placeholder=(format!("{label} — city or code"))
                    autocomplete="off"
                    spellcheck="false"
                    value=(iata.as_str())
                >
            </div>
        </div>
    }
}
