//! Fitness feature map and cross-layer invariants: `docs/fitness.md`.

use topcoat::{
    Result,
    asset::{Asset, asset},
    router::{page, redirect_permanent, route},
    view::view,
};

use crate::{
    components::{back_link, page_head, rail_section, shell},
    content::interests::interest,
};

const FITNESS_JS: Asset = asset!("./lifting/fitness.js");

const MOVEMENTS: &[(&str, &str)] = &[
    ("squat-type", "squat-type"),
    ("hinge", "hinge"),
    ("horizontal-push", "horizontal push"),
    ("vertical-push", "vertical push"),
    ("horizontal-pull", "horizontal pull"),
    ("vertical-pull", "vertical pull"),
    ("core", "core"),
    ("carry", "carry"),
    ("cardio", "cardio"),
    ("olympic-lift", "Olympic lift"),
];

const MOVEMENT_DETAILS: &[(&str, &str)] = &[
    ("dip", "dip"),
    ("fly", "fly"),
    ("elbow-flexion", "elbow flexion"),
    ("elbow-extension", "elbow extension"),
    ("shoulder-abduction", "shoulder abduction"),
    ("shoulder-flexion", "shoulder flexion"),
    ("shoulder-extension", "shoulder extension"),
    ("rear-delt", "rear delt"),
    ("shrug", "shrug"),
    ("knee-flexion", "knee flexion"),
    ("knee-extension", "knee extension"),
    ("hip-abduction", "hip abduction"),
    ("hip-adduction", "hip adduction"),
    ("calf-raise", "calf raise"),
    ("grip-wrist", "grip / wrist"),
    ("throw", "throw"),
];

const MUSCLES: &[(&str, &str)] = &[
    ("quads", "quads"),
    ("glutes", "glutes"),
    ("hamstrings", "hamstrings"),
    ("calves", "calves"),
    ("chest", "chest"),
    ("back", "back"),
    ("shoulders", "shoulders"),
    ("biceps", "biceps"),
    ("triceps", "triceps"),
    ("forearms", "forearms"),
    ("traps", "traps"),
    ("adductors", "adductors"),
    ("core", "core"),
];

const EQUIPMENT: &[(&str, &str)] = &[
    ("barbell", "barbell"),
    ("dumbbell", "dumbbell"),
    ("cable", "cable"),
    ("machine", "machine"),
    ("smith-machine", "smith machine"),
    ("bodyweight", "bodyweight"),
    ("landmine", "landmine"),
    ("rings", "rings"),
    ("sandbag", "sandbag"),
    ("medicine-ball", "medicine ball"),
];

const SET_TYPES: &[(&str, &str)] = &[
    ("NORMAL_SET", "working"),
    ("WARMUP_SET", "warm-up"),
    ("FAILURE_SET", "failure"),
    ("PARTIAL_REPS_SET", "partial reps"),
    ("NEGATIVE_REPS_SET", "negative reps"),
    ("DROP_SET", "drop set"),
];

#[page("/lifting")]
async fn lifting() -> Result {
    let meta = interest("lifting");
    view! { shell(title: meta.title, active: "interests",
        page_head(
            stamp: meta.slug,
            title: meta.title,
            lede: meta.teaser,
        )
        <div
            class="fitness"
            data-fitness=""
            data-fitness-api="/api/fitness"
            data-fitness-fallback-api="http://127.0.0.1:8791/api/fitness"
        >
            rail_section(class: "mt-10", stamp: "filters",
                <section class="fitness-filter-card" aria-labelledby="fitness-filter-title">
                    <div class="fitness-filter-head">
                        <div>
                            <p id="fitness-filter-title" class="fitness-kicker">"the whole archive"</p>
                            <p class="fitness-summary" data-fitness-summary="" aria-live="polite">
                                "Loading the workout database…"
                            </p>
                        </div>
                        <button
                            class="fitness-clear"
                            type="button"
                            data-fitness-clear=""
                            hidden=""
                        >"clear filters"</button>
                    </div>

                    <form class="fitness-form" data-fitness-form="" action="/lifting" method="get">
                        <div class="fitness-primary-grid">
                            <label class="fitness-field fitness-search-field" for="fitness-search">
                                <span>"search"</span>
                                <input
                                    id="fitness-search"
                                    name="q"
                                    type="search"
                                    placeholder="exercise, workout, or note"
                                    autocomplete="off"
                                >
                            </label>
                            <label class="fitness-field" for="fitness-exercise">
                                <span>"exact exercise"</span>
                                <select id="fitness-exercise" name="exercise">
                                    <option value="">"all exercises"</option>
                                </select>
                            </label>
                        </div>

                        <fieldset class="fitness-facet fitness-movement">
                            <legend>"movement pattern"</legend>
                            <div class="fitness-chip-grid">
                                for (value, label) in MOVEMENTS {
                                    <label class="fitness-check-chip">
                                        <input type="checkbox" name="movement" value=(*value)>
                                        <span>(*label)</span>
                                    </label>
                                }
                            </div>
                            <p class="fitness-field-note">
                                "Squat-type includes squats, lunges, split squats, step-ups, and leg presses."
                            </p>
                        </fieldset>

                        <details class="fitness-more">
                            <summary>"more filters"</summary>
                            <div class="fitness-more-body">
                                <div class="fitness-range-grid">
                                    <label class="fitness-field" for="fitness-from">
                                        <span>"from"</span>
                                        <input id="fitness-from" name="from" type="date">
                                    </label>
                                    <label class="fitness-field" for="fitness-to">
                                        <span>"through"</span>
                                        <input id="fitness-to" name="to" type="date">
                                    </label>
                                    <label class="fitness-field" for="fitness-daypart">
                                        <span>"time of day"</span>
                                        <select id="fitness-daypart" name="time_of_day">
                                            <option value="">"any time"</option>
                                            <option value="morning">"morning · 5–11"</option>
                                            <option value="afternoon">"afternoon · 12–4"</option>
                                            <option value="evening">"evening · 5–8"</option>
                                            <option value="night">"night · 9–4"</option>
                                        </select>
                                    </label>
                                    <label class="fitness-field" for="fitness-weekday">
                                        <span>"weekday"</span>
                                        <select id="fitness-weekday" name="weekday">
                                            <option value="">"any day"</option>
                                            <option value="mon">"Monday"</option>
                                            <option value="tue">"Tuesday"</option>
                                            <option value="wed">"Wednesday"</option>
                                            <option value="thu">"Thursday"</option>
                                            <option value="fri">"Friday"</option>
                                            <option value="sat">"Saturday"</option>
                                            <option value="sun">"Sunday"</option>
                                        </select>
                                    </label>
                                </div>

                                <fieldset class="fitness-facet">
                                    <legend>"movement detail"</legend>
                                    <div class="fitness-chip-grid fitness-chip-grid-compact">
                                        for (value, label) in MOVEMENT_DETAILS {
                                            <label class="fitness-check-chip">
                                                <input type="checkbox" name="movement" value=(*value)>
                                                <span>(*label)</span>
                                            </label>
                                        }
                                    </div>
                                </fieldset>

                                <fieldset class="fitness-facet">
                                    <legend>"muscle group"</legend>
                                    <div class="fitness-chip-grid fitness-chip-grid-compact">
                                        for (value, label) in MUSCLES {
                                            <label class="fitness-check-chip">
                                                <input type="checkbox" name="muscle" value=(*value)>
                                                <span>(*label)</span>
                                            </label>
                                        }
                                    </div>
                                </fieldset>

                                <fieldset class="fitness-facet">
                                    <legend>"equipment"</legend>
                                    <div class="fitness-chip-grid fitness-chip-grid-compact">
                                        for (value, label) in EQUIPMENT {
                                            <label class="fitness-check-chip">
                                                <input type="checkbox" name="equipment" value=(*value)>
                                                <span>(*label)</span>
                                            </label>
                                        }
                                    </div>
                                </fieldset>

                                <fieldset class="fitness-facet">
                                    <legend>"set kind"</legend>
                                    <div class="fitness-chip-grid fitness-chip-grid-compact">
                                        for (value, label) in SET_TYPES {
                                            <label class="fitness-check-chip">
                                                <input type="checkbox" name="set_type" value=(*value)>
                                                <span>(*label)</span>
                                            </label>
                                        }
                                    </div>
                                </fieldset>

                                <div class="fitness-number-groups">
                                    <fieldset class="fitness-mini-range">
                                        <legend>"load"</legend>
                                        <label for="fitness-min-load">
                                            <span class="sr-only">"minimum load"</span>
                                            <input
                                                id="fitness-min-load"
                                                name="min_load"
                                                type="number"
                                                inputmode="decimal"
                                                step="any"
                                                min="0"
                                                placeholder="min"
                                            >
                                        </label>
                                        <span aria-hidden="true">"–"</span>
                                        <label for="fitness-max-load">
                                            <span class="sr-only">"maximum load"</span>
                                            <input
                                                id="fitness-max-load"
                                                name="max_load"
                                                type="number"
                                                inputmode="decimal"
                                                step="any"
                                                min="0"
                                                placeholder="max"
                                            >
                                        </label>
                                    </fieldset>
                                    <fieldset class="fitness-mini-range">
                                        <legend>"reps"</legend>
                                        <label for="fitness-min-reps">
                                            <span class="sr-only">"minimum reps"</span>
                                            <input
                                                id="fitness-min-reps"
                                                name="min_reps"
                                                type="number"
                                                inputmode="numeric"
                                                step="1"
                                                min="0"
                                                placeholder="min"
                                            >
                                        </label>
                                        <span aria-hidden="true">"–"</span>
                                        <label for="fitness-max-reps">
                                            <span class="sr-only">"maximum reps"</span>
                                            <input
                                                id="fitness-max-reps"
                                                name="max_reps"
                                                type="number"
                                                inputmode="numeric"
                                                step="1"
                                                min="0"
                                                placeholder="max"
                                            >
                                        </label>
                                    </fieldset>
                                    <label class="fitness-field fitness-effort" for="fitness-effort">
                                        <span>"max RIR/RPE"</span>
                                        <input
                                            id="fitness-effort"
                                            name="max_effort"
                                            type="number"
                                            inputmode="decimal"
                                            min="0"
                                            step="0.5"
                                            placeholder="any"
                                        >
                                    </label>
                                </div>
                                <p class="fitness-field-note">
                                    "Load is shown exactly as exported; the source file does not include its unit."
                                </p>

                                <div class="fitness-flags">
                                    <label><input type="checkbox" name="has_record" value="true"> <span>"personal records"</span></label>
                                    <label><input type="checkbox" name="has_superset" value="true"> <span>"supersets"</span></label>
                                    <label><input type="checkbox" name="has_notes" value="true"> <span>"with notes"</span></label>
                                    <label><input type="checkbox" name="incomplete" value="true"> <span>"incomplete rows"</span></label>
                                    <label><input type="checkbox" name="duration" value="suspicious"> <span>"suspect timers only"</span></label>
                                </div>

                                <label class="fitness-field fitness-page-size" for="fitness-page-size">
                                    <span>"workouts per page"</span>
                                    <select id="fitness-page-size" name="per_page">
                                        <option value="10">"10"</option>
                                        <option value="20" selected="">"20"</option>
                                        <option value="40">"40"</option>
                                    </select>
                                </label>
                            </div>
                        </details>

                        <div class="fitness-form-foot">
                            <div
                                class="fitness-active-filters"
                                data-fitness-active=""
                                aria-label="Active filters"
                            ></div>
                            <button class="fitness-apply" type="submit">"apply filters"</button>
                        </div>
                    </form>
                </section>
            )

            rail_section(class: "mt-12", stamp: "sets",
                <header class="fitness-results-head">
                    <div>
                        <h2 class="font-display text-2xl font-semibold">"Set log"</h2>
                        <p
                            class="fitness-result-count"
                            data-fitness-result-count=""
                            role="status"
                            aria-live="polite"
                        >"Fetching sets…"</p>
                    </div>
                    <p class="fitness-sort">"newest first"</p>
                </header>
            )

            <section
                class="fitness-list"
                data-fitness-results=""
                aria-label="Filtered workout sets"
                aria-busy="true"
            >
                <article class="fitness-workout fitness-workout-loading" aria-hidden="true">
                    <div class="fitness-workout-stamp"></div>
                    <div class="fitness-sheet">
                        <div class="fitness-loading-line fitness-loading-line-title"></div>
                        <div class="fitness-loading-line"></div>
                        <div class="fitness-loading-line fitness-loading-line-short"></div>
                    </div>
                </article>
                <p class="fitness-js-note">
                    "The set log needs JavaScript to query the live workout database."
                </p>
            </section>

            <nav
                class="fitness-pager"
                data-fitness-pager=""
                aria-label="Workout log pages"
                hidden=""
            ></nav>
        </div>
        <script type="module" src=(FITNESS_JS)></script>
        back_link(href: "/interests", label: "all interests")
    ) }
}

#[route(GET "/interests/lifting")]
async fn legacy_lifting() -> Result {
    Err(redirect_permanent("/lifting").into())
}
