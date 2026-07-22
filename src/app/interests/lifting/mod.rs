//! Fitness feature map and cross-layer invariants: `docs/fitness.md`.

mod data;
mod filters;
mod format;
mod heatmap;
mod results;

use topcoat::{
    Result,
    asset::{Asset, asset},
    context::Cx,
    router::{HeaderValue, header, not_found, page, parse_query_params, path_param, redirect, uri},
    view::{component, view},
};

use crate::{
    components::{back_link, page_head, rail_section, shell},
    content::interests::interest,
};

use self::{
    data as fitness,
    filters::{EQUIPMENT, Filters, LOG_PATH, MOVEMENT_DETAILS, MOVEMENTS, MUSCLES, SET_TYPES},
    format::{format_archive_summary, format_integer, plural},
    results::{WorkoutCard, make_pager, total_pages, workout_url},
};

const AUTO_FILTER_JS: Asset = asset!("./auto-filter.js");

#[path_param]
struct WorkoutPath(str);

#[page("/lifting")]
async fn lifting(cx: &Cx) -> Result {
    // Filtered archive links remain useful; the archive itself now lives at
    // `/lifting/log`.
    if let Some(query) = uri(cx).query() {
        return Err(redirect(&format!("{LOG_PATH}?{query}")).into());
    }

    let meta = interest("lifting");
    let (calendar, latest) = fitness::load_home().await;
    if let Err(error) = &calendar {
        eprintln!("fitness calendar fetch failed: {error}");
    }
    if let Err(error) = &latest {
        eprintln!("fitness latest workout fetch failed: {error}");
    }

    let calendar_days = calendar.ok().map(|calendar| calendar.days);
    let latest_error = latest.as_ref().err();
    let latest_workout = latest
        .as_ref()
        .ok()
        .and_then(|detail| detail.workout.as_ref());
    let next_lift_url = latest
        .as_ref()
        .ok()
        .and_then(|detail| detail.older_workout_path.as_deref())
        .map(workout_url);

    view! {
        ((header::CACHE_CONTROL, HeaderValue::from_static("no-store")))
        shell(
            title: meta.title,
            active: "interests",
            runtime: false,
            page_head(stamp: meta.slug, title: meta.title, lede: meta.teaser)
            <div class="fitness">
                rail_section(
                    class: "mt-10",
                    stamp: "volume",
                    if let Some(days) = calendar_days {
                        heatmap::calendar_heatmap(days: days)
                    } else {
                        <section class="fitness-calendar-error">
                            <p class="fitness-empty-copy">
                                "Daily volume is unavailable right now."
                            </p>
                        </section>
                    }
                )

                rail_section(
                    class: "mt-12",
                    stamp: "sets",
                    <header class="fitness-results-head" id="set-log">
                        <div>
                            <h2 class="font-display text-2xl font-semibold">
                                "Most recent lift"
                            </h2>
                            <p class="fitness-result-count">
                                "Each workout has its own linkable page."
                            </p>
                        </div>
                        <a class="fitness-full-log-link" href=(LOG_PATH)>
                            "search full log →"
                        </a>
                        if let Some(href) = &next_lift_url {
                            <a class="fitness-lift-link" href=(href.as_str())>
                                "see next lift →"
                            </a>
                        }
                    </header>
                )

                <section class="fitness-list" aria-label="Most recent workout">
                    if latest_error.is_some() {
                        <div class="fitness-empty fitness-error">
                            <p class="fitness-empty-title">
                                "The latest lift did not load."
                            </p>
                            <p class="fitness-empty-copy">
                                "Try the workout archive again in a moment."
                            </p>
                            <a class="fitness-empty-reset" href="/lifting#set-log">
                                "retry"
                            </a>
                        </div>
                    } else if let Some(workout) = latest_workout {
                        workout_sheet(workout: workout, permalink: true)
                    } else {
                        <div class="fitness-empty">
                            <p class="fitness-empty-title">"No lifts yet."</p>
                            <p class="fitness-empty-copy">
                                "The workout archive will appear here after its first import."
                            </p>
                        </div>
                    }
                </section>
            </div>
            back_link(href: "/interests", label: "all interests")
        )
    }
}

#[page("/lifting/log")]
async fn lifting_log(cx: &Cx) -> Result {
    let raw = match parse_query_params::<Vec<(String, String)>>(cx) {
        Ok(raw) => raw,
        Err(_) => return Err(redirect(LOG_PATH).into()),
    };
    let Some(filters) = Filters::normalize(raw) else {
        return Err(redirect(LOG_PATH).into());
    };
    let canonical = filters.query();
    if uri(cx).query().is_some_and(|query| query != canonical) {
        return Err(redirect(&filters.url(false)).into());
    }

    let meta = interest("lifting");
    let api_pairs = filters.api_pairs();
    let (facets, sets) = fitness::load(&api_pairs).await;
    if let Err(error) = &facets {
        eprintln!("fitness facets fetch failed: {error}");
    }
    if let Err(error) = &sets {
        eprintln!("fitness sets fetch failed: {error}");
    }
    if let Ok(page) = &sets {
        let last_page = total_pages(page);
        if page.page > last_page {
            return Err(redirect(&filters.page_url(last_page)).into());
        }
    }

    let archive_summary = facets
        .as_ref()
        .map(format_archive_summary)
        .unwrap_or_else(|_| "Workout archive · live totals unavailable".to_string());
    let selected_exercise = filters.value("exercise");
    let mut exercise_options = Vec::new();
    let selected_exercise_missing = match &facets {
        Ok(data) => !data
            .exercises
            .iter()
            .any(|option| option.value == selected_exercise),
        Err(_) => true,
    };
    if !selected_exercise.is_empty() && selected_exercise_missing {
        exercise_options.push((selected_exercise.to_string(), selected_exercise.to_string()));
    }
    if let Ok(data) = &facets {
        exercise_options.extend(data.exercises.iter().map(|option| {
            (
                option.value.clone(),
                format!("{} · {}", option.value, format_integer(option.count)),
            )
        }));
    }

    let active_filters = filters.active();
    let result_summary = match &sets {
        Ok(page) if page.total_sets > 0 => {
            let visible_sets = page
                .workouts
                .iter()
                .map(|workout| workout.sets.len() as u64)
                .sum::<u64>();
            format!(
                "{} matching sets across {} workouts · {} on this page",
                format_integer(page.total_sets),
                format_integer(page.total_workouts),
                format_integer(visible_sets),
            )
        }
        Ok(_) => "No sets match these filters.".to_string(),
        Err(error) => error
            .rejected_message()
            .map(|message| format!("A filter was rejected · {message}"))
            .unwrap_or_else(|| "Workout database is unreachable.".to_string()),
    };
    let pager = sets
        .as_ref()
        .ok()
        .and_then(|page| make_pager(page, &filters));
    let retry_url = filters.url(true);

    view! {
        ((header::CACHE_CONTROL, HeaderValue::from_static("no-store")))
        shell(
            title: meta.title,
            active: "interests",
            runtime: false,
            page_head(stamp: meta.slug, title: meta.title, lede: meta.teaser)
            <div class="fitness">
                rail_section(
                    class: "mt-10",
                    stamp: "filters",
                    <section
                        class="fitness-filter-card"
                        aria-labelledby="fitness-filter-title"
                    >
                        <div class="fitness-filter-head">
                            <div>
                                <p id="fitness-filter-title" class="fitness-kicker">
                                    "the whole archive"
                                </p>
                                <p class="fitness-summary">(archive_summary.as_str())</p>
                            </div>
                            if !active_filters.is_empty() {
                                <a class="fitness-clear" href=(LOG_PATH)>
                                    "clear filters"
                                </a>
                            }
                        </div>

                        <form
                            class="fitness-form"
                            action="/lifting/log#set-log"
                            method="get"
                            data-lifting-filters=""
                        >
                            <div class="fitness-primary-grid">
                                <label
                                    class="fitness-field fitness-search-field"
                                    for="fitness-search"
                                >
                                    <span>"search"</span>
                                    <input
                                        id="fitness-search"
                                        name="q"
                                        type="search"
                                        value=(filters.value("q"))
                                        placeholder="exercise, workout, or note"
                                        autocomplete="off"
                                    >
                                </label>
                                <label class="fitness-field" for="fitness-exercise">
                                    <span>"exact exercise"</span>
                                    <select id="fitness-exercise" name="exercise">
                                        <option value="" selected=(selected_exercise.is_empty())>
                                            "all exercises"
                                        </option>
                                        for option in exercise_options.iter() {
                                            <option
                                                value=(option.0.as_str())
                                                selected=(selected_exercise == option.0)
                                            >
                                                (option.1.as_str())
                                            </option>
                                        }
                                    </select>
                                </label>
                            </div>

                            <fieldset class="fitness-facet fitness-movement">
                                <legend>"movement pattern"</legend>
                                <div class="fitness-chip-grid">
                                    for (value, label) in MOVEMENTS {
                                        <label class="fitness-check-chip">
                                            <input
                                                type="checkbox"
                                                name="movement"
                                                value=(*value)
                                                checked=(filters.contains("movement", value))
                                            >
                                            <span>(*label)</span>
                                        </label>
                                    }
                                </div>
                                <p class="fitness-field-note">
                                    "Squat-type includes squats, lunges, split squats, step-ups, and leg presses."
                                </p>
                            </fieldset>

                            <details class="fitness-more" open=(filters.advanced())>
                                <summary>"more filters"</summary>
                                <div class="fitness-more-body">
                                    <div class="fitness-range-grid">
                                        <label class="fitness-field" for="fitness-from">
                                            <span>"from"</span>
                                            <input
                                                id="fitness-from"
                                                name="from"
                                                type="date"
                                                value=(filters.value("from"))
                                            >
                                        </label>
                                        <label class="fitness-field" for="fitness-to">
                                            <span>"through"</span>
                                            <input
                                                id="fitness-to"
                                                name="to"
                                                type="date"
                                                value=(filters.value("to"))
                                            >
                                        </label>
                                        <label class="fitness-field" for="fitness-daypart">
                                            <span>"time of day"</span>
                                            <select id="fitness-daypart" name="time_of_day">
                                                <option
                                                    value=""
                                                    selected=(filters.value("time_of_day").is_empty())
                                                >
                                                    "any time"
                                                </option>
                                                <option
                                                    value="morning"
                                                    selected=(filters.value("time_of_day") == "morning")
                                                >
                                                    "morning · 5–11"
                                                </option>
                                                <option
                                                    value="afternoon"
                                                    selected=(filters.value("time_of_day") == "afternoon")
                                                >
                                                    "afternoon · 12–4"
                                                </option>
                                                <option
                                                    value="evening"
                                                    selected=(filters.value("time_of_day") == "evening")
                                                >
                                                    "evening · 5–8"
                                                </option>
                                                <option
                                                    value="night"
                                                    selected=(filters.value("time_of_day") == "night")
                                                >
                                                    "night · 9–4"
                                                </option>
                                            </select>
                                        </label>
                                        <label class="fitness-field" for="fitness-weekday">
                                            <span>"weekday"</span>
                                            <select id="fitness-weekday" name="weekday">
                                                <option
                                                    value=""
                                                    selected=(filters.value("weekday").is_empty())
                                                >
                                                    "any day"
                                                </option>
                                                <option
                                                    value="mon"
                                                    selected=(filters.value("weekday") == "mon")
                                                >
                                                    "Monday"
                                                </option>
                                                <option
                                                    value="tue"
                                                    selected=(filters.value("weekday") == "tue")
                                                >
                                                    "Tuesday"
                                                </option>
                                                <option
                                                    value="wed"
                                                    selected=(filters.value("weekday") == "wed")
                                                >
                                                    "Wednesday"
                                                </option>
                                                <option
                                                    value="thu"
                                                    selected=(filters.value("weekday") == "thu")
                                                >
                                                    "Thursday"
                                                </option>
                                                <option
                                                    value="fri"
                                                    selected=(filters.value("weekday") == "fri")
                                                >
                                                    "Friday"
                                                </option>
                                                <option
                                                    value="sat"
                                                    selected=(filters.value("weekday") == "sat")
                                                >
                                                    "Saturday"
                                                </option>
                                                <option
                                                    value="sun"
                                                    selected=(filters.value("weekday") == "sun")
                                                >
                                                    "Sunday"
                                                </option>
                                            </select>
                                        </label>
                                    </div>

                                    <fieldset class="fitness-facet">
                                        <legend>"movement detail"</legend>
                                        <div class="fitness-chip-grid fitness-chip-grid-compact">
                                            for (value, label) in MOVEMENT_DETAILS {
                                                <label class="fitness-check-chip">
                                                    <input
                                                        type="checkbox"
                                                        name="movement"
                                                        value=(*value)
                                                        checked=(filters.contains("movement", value))
                                                    >
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
                                                    <input
                                                        type="checkbox"
                                                        name="muscle"
                                                        value=(*value)
                                                        checked=(filters.contains("muscle", value))
                                                    >
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
                                                    <input
                                                        type="checkbox"
                                                        name="equipment"
                                                        value=(*value)
                                                        checked=(filters.contains("equipment", value))
                                                    >
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
                                                    <input
                                                        type="checkbox"
                                                        name="set_type"
                                                        value=(*value)
                                                        checked=(filters.contains("set_type", value))
                                                    >
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
                                                    value=(filters.value("min_load"))
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
                                                    value=(filters.value("max_load"))
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
                                                    value=(filters.value("min_reps"))
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
                                                    value=(filters.value("max_reps"))
                                                    placeholder="max"
                                                >
                                            </label>
                                        </fieldset>
                                        <label
                                            class="fitness-field fitness-effort"
                                            for="fitness-effort"
                                        >
                                            <span>"max RIR/RPE"</span>
                                            <input
                                                id="fitness-effort"
                                                name="max_effort"
                                                type="number"
                                                inputmode="decimal"
                                                min="0"
                                                step="0.5"
                                                value=(filters.value("max_effort"))
                                                placeholder="any"
                                            >
                                        </label>
                                    </div>
                                    <p class="fitness-field-note">
                                        "Load is shown exactly as exported; the source file does not include its unit."
                                    </p>

                                    <div class="fitness-flags">
                                        <label>
                                            <input
                                                type="checkbox"
                                                name="has_record"
                                                value="true"
                                                checked=(filters.contains("has_record", "true"))
                                            >
                                            <span>"personal records"</span>
                                        </label>
                                        <label>
                                            <input
                                                type="checkbox"
                                                name="has_superset"
                                                value="true"
                                                checked=(filters.contains("has_superset", "true"))
                                            >
                                            <span>"supersets"</span>
                                        </label>
                                        <label>
                                            <input
                                                type="checkbox"
                                                name="has_notes"
                                                value="true"
                                                checked=(filters.contains("has_notes", "true"))
                                            >
                                            <span>"with notes"</span>
                                        </label>
                                        <label>
                                            <input
                                                type="checkbox"
                                                name="incomplete"
                                                value="true"
                                                checked=(filters.contains("incomplete", "true"))
                                            >
                                            <span>"incomplete rows"</span>
                                        </label>
                                        <label>
                                            <input
                                                type="checkbox"
                                                name="duration"
                                                value="suspicious"
                                                checked=(filters.contains("duration", "suspicious"))
                                            >
                                            <span>"suspect timers only"</span>
                                        </label>
                                    </div>

                                    <label
                                        class="fitness-field fitness-page-size"
                                        for="fitness-page-size"
                                    >
                                        <span>"workouts per page"</span>
                                        <select id="fitness-page-size" name="per_page">
                                            <option value="10" selected=(filters.per_page() == "10")>
                                                "10"
                                            </option>
                                            <option value="20" selected=(filters.per_page() == "20")>
                                                "20"
                                            </option>
                                            <option value="40" selected=(filters.per_page() == "40")>
                                                "40"
                                            </option>
                                        </select>
                                    </label>
                                </div>
                            </details>

                            <div class="fitness-form-foot">
                                <div
                                    class="fitness-active-filters"
                                    aria-label="Active filters"
                                >
                                    if active_filters.is_empty() {
                                        <span class="fitness-no-filters">
                                            "All sets are included."
                                        </span>
                                    }
                                    for filter in active_filters.iter() {
                                        <a
                                            class="fitness-active-chip"
                                            href=(filter.href.as_str())
                                            aria-label=(filter.aria_label.as_str())
                                        >
                                            (format!("{} ×", filter.label))
                                        </a>
                                    }
                                </div>
                                <button class="fitness-apply" type="submit">
                                    "apply filters"
                                </button>
                            </div>
                        </form>
                        <script type="module" src=(AUTO_FILTER_JS)></script>
                    </section>
                )

                rail_section(
                    class: "mt-12",
                    stamp: "sets",
                    <header class="fitness-results-head" id="set-log">
                        <div>
                            <h2 class="font-display text-2xl font-semibold">
                                "Set log"
                            </h2>
                            <p class="fitness-result-count">
                                (result_summary.as_str())
                            </p>
                        </div>
                        <p class="fitness-sort">"newest first"</p>
                    </header>
                )

                <section class="fitness-list" aria-label="Filtered workout sets">
                    if let Err(error) = &sets {
                        <div class="fitness-empty fitness-error">
                            if let Some(message) = error.rejected_message() {
                                <p class="fitness-empty-title">
                                    "That filter combination is not valid."
                                </p>
                                <p class="fitness-empty-copy">(message)</p>
                                <a class="fitness-empty-reset" href="/lifting/log#set-log">
                                    "clear every filter"
                                </a>
                            } else {
                                <p class="fitness-empty-title">
                                    "The set log did not load."
                                </p>
                                <p class="fitness-empty-copy">
                                    "The filters are intact. Try the database again."
                                </p>
                                <a class="fitness-empty-reset" href=(retry_url.as_str())>
                                    "retry"
                                </a>
                            }
                        </div>
                    }
                    if let Ok(page) = &sets && page.workouts.is_empty() {
                        <div class="fitness-empty">
                            <p class="fitness-empty-title">
                                if page.total_sets > 0 {
                                    "This page is empty."
                                } else {
                                    "No matching sets."
                                }
                            </p>
                            <p class="fitness-empty-copy">
                                if page.total_sets > 0 {
                                    "Try a previous page."
                                } else {
                                    "Loosen a movement, date, or numeric filter and the log will reappear."
                                }
                            </p>
                            <a class="fitness-empty-reset" href="/lifting/log#set-log">
                                "clear every filter"
                            </a>
                        </div>
                    }
                    if let Ok(page) = &sets {
                        for workout in page.workouts.iter() {
                            workout_sheet(workout: workout, permalink: true)
                        }
                    }
                </section>

                if let Some(pager) = &pager {
                    <nav class="fitness-pager" aria-label="Workout log pages">
                        if let Some(href) = &pager.newer {
                            <a class="fitness-page-link" href=(href.as_str())>
                                "← newer"
                            </a>
                        } else {
                            <span
                                class="fitness-page-link fitness-page-disabled"
                                aria-disabled="true"
                            >
                                "← newer"
                            </span>
                        }
                        for part in pager.parts.iter() {
                            if let Some(number) = part {
                                if *number == pager.current {
                                    <span class="fitness-page-current" aria-current="page">
                                        (number.to_string())
                                    </span>
                                } else {
                                    <a
                                        class="fitness-page-link"
                                        href=(filters.page_url(*number))
                                    >
                                        (number.to_string())
                                    </a>
                                }
                            } else {
                                <span class="fitness-page-gap">"…"</span>
                            }
                        }
                        if let Some(href) = &pager.older {
                            <a class="fitness-page-link" href=(href.as_str())>
                                "older →"
                            </a>
                        } else {
                            <span
                                class="fitness-page-link fitness-page-disabled"
                                aria-disabled="true"
                            >
                                "older →"
                            </span>
                        }
                    </nav>
                }
            </div>
            back_link(href: "/interests", label: "all interests")
        )
    }
}

#[page("/lifting/{workout_path}")]
async fn lift_detail(cx: &Cx) -> Result {
    let workout_path = path_param::<WorkoutPath>(cx);
    if uri(cx).query().is_some() {
        return Err(redirect(&workout_url(workout_path)).into());
    }

    let detail = fitness::load_workout_by_path(workout_path).await;
    if matches!(&detail, Err(error) if error.is_not_found()) {
        return Err(not_found().into());
    }
    if let Err(error) = &detail {
        eprintln!("fitness workout fetch failed: {error}");
    }

    let meta = interest("lifting");
    let workout = detail
        .as_ref()
        .ok()
        .and_then(|detail| detail.workout.as_ref());
    let page_title = workout
        .map(|workout| format!("{} · {}", workout.title, meta.title))
        .unwrap_or_else(|| meta.title.to_string());
    let newer_lift_url = detail
        .as_ref()
        .ok()
        .and_then(|detail| detail.newer_workout_path.as_deref())
        .map(workout_url);
    let older_lift_url = detail
        .as_ref()
        .ok()
        .and_then(|detail| detail.older_workout_path.as_deref())
        .map(workout_url);

    view! {
        ((header::CACHE_CONTROL, HeaderValue::from_static("no-store")))
        shell(
            title: page_title.as_str(),
            active: "interests",
            runtime: false,
            page_head(
                stamp: "lift",
                title: "Workout",
                lede: "A complete, linkable entry from my workout archive."
            )
            <div class="fitness">
                <section class="fitness-list" aria-label="Workout">
                    if let Some(workout) = workout {
                        workout_sheet(workout: workout, permalink: false)
                    } else {
                        <div class="fitness-empty fitness-error">
                            <p class="fitness-empty-title">"This lift did not load."</p>
                            <p class="fitness-empty-copy">
                                "Try the latest workout or the full archive again in a moment."
                            </p>
                            <a class="fitness-empty-reset" href="/lifting">
                                "latest lift"
                            </a>
                        </div>
                    }
                </section>

                if newer_lift_url.is_some() || older_lift_url.is_some() {
                    <nav class="fitness-lift-nav" aria-label="Workout navigation">
                        if let Some(href) = &newer_lift_url {
                            <a class="fitness-lift-link" href=(href.as_str())>
                                "← newer lift"
                            </a>
                        } else {
                            <span></span>
                        }
                        <a
                            class="fitness-lift-link fitness-latest-link"
                            href="/lifting"
                        >
                            "latest lift"
                        </a>
                        if let Some(href) = &older_lift_url {
                            <a class="fitness-lift-link" href=(href.as_str())>
                                "see next lift →"
                            </a>
                        } else {
                            <span></span>
                        }
                    </nav>
                }
            </div>
            back_link(href: "/lifting", label: "latest lift")
        )
    }
}

#[component]
async fn workout_sheet(workout: &fitness::Workout, permalink: bool) -> Result {
    let workout = WorkoutCard::from(workout);
    let workout_link_label = format!("Open {} workout", workout.title);
    view! {
        <article class="fitness-workout rail-row">
            <div class="fitness-workout-stamp rail-stamp">
                <time
                    datetime=(workout.datetime.as_str())
                    title="Eastern start and end time from the workout archive"
                >
                    <span class="fitness-stamp-date">(workout.date.as_str())</span>
                    <span class="fitness-stamp-time">
                        (workout.time_range.as_str())
                    </span>
                </time>
            </div>
            <div class="fitness-sheet">
                <header class="fitness-sheet-head">
                    <h3 class="fitness-workout-title">
                        if permalink {
                            <a
                                class="fitness-workout-link"
                                href=(workout.href.as_str())
                                aria-label=(workout_link_label.as_str())
                            >
                                (workout.title)
                            </a>
                        } else {
                            (workout.title)
                        }
                    </h3>
                    <p class="fitness-workout-meta">
                        (format!(
                            "{} · {} {}", workout.duration, workout.set_count,
                            plural(workout.set_count, "set", "sets"),
                        ))
                        if workout.duration_suspicious {
                            " · "
                            <span
                                class="fitness-timer-warning"
                                title="This source workout was left running for at least four hours, or recorded as zero."
                            >
                                "timer outlier"
                            </span>
                        }
                    </p>
                </header>
                if let Some(description) = workout.description {
                    <p class="fitness-workout-note">(description)</p>
                }
                if let Some(notes) = workout.notes {
                    <p class="fitness-workout-note">(notes)</p>
                }
                for group in workout.groups.iter() {
                    <section class="fitness-exercise-group">
                        <div class="fitness-exercise-head">
                            <h4 class="fitness-exercise-name">(group.name)</h4>
                            <span class="fitness-exercise-count">
                                (format!(
                                    "{} {} · {} volume points", group.rows.len(), plural(group
                                    .rows.len(), "set", "sets"), format_integer(group
                                    .volume_points),
                                ))
                            </span>
                        </div>
                        <ol class="fitness-set-list">
                            for row in group.rows.iter() {
                                <li class="fitness-set-row">
                                    <span
                                        class=(row.marker_class)
                                        role="img"
                                        title=(row.marker_title.as_str())
                                        aria-label=(row.marker_label.as_str())
                                    >
                                        if let Some(text) = &row.marker_text {
                                            (text.as_str())
                                        }
                                        for style in row.point_styles.iter() {
                                            <span
                                                class="fitness-set-point fitness-set-point-filled"
                                                style=(style.as_str())
                                                aria-hidden="true"
                                            >
                                                "★"
                                            </span>
                                        }
                                    </span>
                                    <span class="fitness-set-prescription">
                                        (row.prescription.as_str())
                                    </span>
                                    <span class="fitness-set-details">
                                        (row.details.as_str())
                                    </span>
                                    <span class="fitness-set-records">
                                        for record in row.records.iter() {
                                            <span class=(record.class.as_str())>
                                                (record.label.as_str())
                                            </span>
                                        }
                                    </span>
                                    if let Some(note) = row.note {
                                        <span class="fitness-set-note">(note)</span>
                                    }
                                </li>
                            }
                        </ol>
                    </section>
                }
            </div>
        </article>
    }
}
