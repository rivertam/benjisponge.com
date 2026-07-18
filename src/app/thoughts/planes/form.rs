//! The flight form and airport combobox: ports of `FlightForm.tsx` and
//! `AirportCombobox.tsx` from ~/how-bad.
//!
//! The baseline is a plain GET form submitting to the page's own URL: the
//! route fields are text inputs named `from`/`to` holding IATA codes, so the
//! whole flow works with JavaScript disabled. The combobox is an enhancement
//! on top: a signal mirrors each input and a shard re-renders up to eight
//! matches server-side (`search_airports` — prefix, one-edit fuzzy, metro
//! aliases) as the text changes.

use topcoat::{
    Result,
    runtime::{Event, ReadSignal, shard},
    view::{component, view},
};

use crate::flight::{
    airports::{Airport, search_airports},
    emissions::Cabin,
};

#[component]
pub async fn flight_form(
    from: Option<Airport>,
    to: Option<Airport>,
    cabin: Cabin,
    round_trip: bool,
    revealed: bool,
) -> Result {
    view! {
        <form class=(if revealed { "flight-form form-dock" } else { "flight-form" })>
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
    }
}

/// One route endpoint. The visible input is the GET baseline: it submits its
/// literal text as the `from`/`to` query param (an IATA code when picked from
/// the suggestions). The suggestion list is the enhancement.
#[component]
async fn airport_field(label: &str, name: &str, iata: String) -> Result {
    let input_id = format!("airport-{name}");
    view! {
        signal text = iata;
        // A re-rendered shard has no access to the page's signals, so hand it
        // the input signal's id; its suggestion rows write the pick back
        // through it (see `airport_suggestions`).
        let sid = ReadSignal::new(text).id().to_string();
        <div class="field">
            <label for=(input_id.as_str())>(label)</label>
            <div class="combobox">
                <input
                    id=(input_id.as_str())
                    type="text"
                    name=(name)
                    role="combobox"
                    placeholder=(format!("{label} — city or code"))
                    autocomplete="off"
                    spellcheck="false"
                    :value=$(text.get())
                    @input=$(|e: Event| text.set(e.target.value))
                >
                airport_suggestions(sid: $(sid), input: $(text.get()))
            </div>
        </div>
    }
}

/// Up to eight server-searched matches for the field's text, rendered as the
/// combobox's listbox. Each row's `mousedown` sets the input signal (by the
/// `sid` id, via `raw!` — the one place the port needs hand-written JS) to the
/// row's IATA code; the changed signal re-renders this shard, which then
/// renders nothing because the field holds an exact code. CSS shows the list
/// only while the field has focus (`mousedown` fires before the blur), so it
/// opens while typing and closes on pick or click-away.
#[shard]
async fn airport_suggestions(sid: String, input: String) -> Result {
    let query = input.trim().to_owned();
    let options = if query.is_empty() {
        Vec::new()
    } else {
        search_airports(&query, 8)
    };
    // The field already holds the top match's exact code: nothing to suggest.
    let settled = options
        .first()
        .is_some_and(|a| a.iata.eq_ignore_ascii_case(&query));
    view! {
        if !settled && !options.is_empty() {
            <ul class="combobox-list" role="listbox">
                for airport in options {
                    let code = airport.iata.clone();
                    let sid = sid.clone();
                    <li
                        role="option"
                        @mousedown=$(|_e: Event| raw!("cx.signal(${sid}.toString()).set(${code})"))
                    >
                        <span class="opt-main">(format!("{}, {}", airport.city, airport.country))</span>
                        <span class="opt-code">(airport.iata.as_str())</span>
                    </li>
                }
            </ul>
        }
    }
}
