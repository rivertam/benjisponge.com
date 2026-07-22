const root = document.querySelector("[data-fitness]");

if (root) {
  const form = root.querySelector("[data-fitness-form]");
  const summary = root.querySelector("[data-fitness-summary]");
  const resultCount = root.querySelector("[data-fitness-result-count]");
  const results = root.querySelector("[data-fitness-results]");
  const pager = root.querySelector("[data-fitness-pager]");
  const activeFilters = root.querySelector("[data-fitness-active]");
  const clear = root.querySelector("[data-fitness-clear]");
  const exerciseSelect = form?.elements.namedItem("exercise");
  const more = form?.querySelector(".fitness-more");
  const primaryApi = root.dataset.fitnessApi || "/api/fitness";
  const fallbackApi = root.dataset.fitnessFallbackApi || "";
  const initialParams = new URLSearchParams(window.location.search);
  const initialExercise = initialParams.get("exercise");
  let page = positiveInteger(initialParams.get("page")) || 1;
  let requestController = null;
  let debounceTimer = null;
  let lastPayload = null;

  const valueLabels = new Map([
    ["squat-type", "squat-type"],
    ["horizontal-push", "horizontal push"],
    ["vertical-push", "vertical push"],
    ["horizontal-pull", "horizontal pull"],
    ["vertical-pull", "vertical pull"],
    ["olympic-lift", "Olympic lift"],
    ["elbow-flexion", "elbow flexion"],
    ["elbow-extension", "elbow extension"],
    ["shoulder-abduction", "shoulder abduction"],
    ["shoulder-flexion", "shoulder flexion"],
    ["shoulder-extension", "shoulder extension"],
    ["rear-delt", "rear delt"],
    ["knee-flexion", "knee flexion"],
    ["knee-extension", "knee extension"],
    ["hip-abduction", "hip abduction"],
    ["hip-adduction", "hip adduction"],
    ["calf-raise", "calf raise"],
    ["grip-wrist", "grip / wrist"],
    ["smith-machine", "smith machine"],
    ["medicine-ball", "medicine ball"],
    ["NORMAL_SET", "working"],
    ["WARMUP_SET", "warm-up"],
    ["FAILURE_SET", "failure"],
    ["PARTIAL_REPS_SET", "partial reps"],
    ["NEGATIVE_REPS_SET", "negative reps"],
    ["DROP_SET", "drop set"],
    ["has_record", "personal records"],
    ["has_superset", "supersets"],
    ["has_notes", "with notes"],
    ["incomplete", "incomplete rows"],
    ["suspicious", "suspect timers"],
  ]);

  const keyLabels = {
    q: "search",
    exercise: "exercise",
    movement: "movement",
    muscle: "muscle",
    equipment: "equipment",
    set_type: "set kind",
    from: "from",
    to: "through",
    time_of_day: "time",
    weekday: "day",
    min_load: "load ≥",
    max_load: "load ≤",
    min_reps: "reps ≥",
    max_reps: "reps ≤",
    max_effort: "RIR/RPE ≤",
  };

  if (form && summary && resultCount && results && pager && activeFilters && clear) {
    results.querySelector(".fitness-js-note")?.remove();
    if (exerciseSelect instanceof HTMLSelectElement && initialExercise) {
      const pending = document.createElement("option");
      pending.value = initialExercise;
      pending.textContent = initialExercise;
      exerciseSelect.append(pending);
    }
    restoreForm(initialParams);
    renderActive(initialParams);
    loadFacets(initialExercise);
    loadResults({ updateUrl: false });

    form.addEventListener("submit", (event) => {
      event.preventDefault();
      page = 1;
      loadResults({ updateUrl: true });
    });

    form.addEventListener("input", (event) => {
      page = 1;
      const target = event.target;
      const immediate =
        target instanceof HTMLSelectElement ||
        (target instanceof HTMLInputElement &&
          (target.type === "checkbox" || target.type === "date"));
      window.clearTimeout(debounceTimer);
      if (immediate) {
        loadResults({ updateUrl: true });
      } else {
        debounceTimer = window.setTimeout(
          () => loadResults({ updateUrl: true }),
          260,
        );
      }
    });

    clear.addEventListener("click", () => {
      form.reset();
      page = 1;
      if (more) more.open = false;
      loadResults({ updateUrl: true });
    });

    activeFilters.addEventListener("click", (event) => {
      const button = event.target.closest("button[data-filter-key]");
      if (!button) return;
      const params = formParams();
      deleteOne(params, button.dataset.filterKey, button.dataset.filterValue);
      restoreForm(params);
      page = 1;
      loadResults({ updateUrl: true });
    });

    pager.addEventListener("click", (event) => {
      const link = event.target.closest("a[data-fitness-page]");
      if (!link || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
      event.preventDefault();
      page = positiveInteger(link.dataset.fitnessPage) || 1;
      loadResults({ updateUrl: true });
      resultCount.scrollIntoView({ behavior: "smooth", block: "center" });
    });

    window.addEventListener("popstate", () => {
      const params = new URLSearchParams(window.location.search);
      page = positiveInteger(params.get("page")) || 1;
      restoreForm(params);
      loadResults({ updateUrl: false });
    });
  }

  async function loadFacets(selectedExercise) {
    try {
      const data = await getJson("facets", new URLSearchParams());
      const facets = data || {};
      const archive = facets.summary || {};
      if (summary) {
        const parts = [];
        if (Number.isFinite(archive.sets)) {
          parts.push(`${formatInteger(archive.sets)} sets`);
        }
        if (Number.isFinite(archive.workouts)) {
          parts.push(`${formatInteger(archive.workouts)} workouts`);
        }
        if (archive.min_date && archive.max_date) {
          parts.push(`${formatMonth(archive.min_date)}–${formatMonth(archive.max_date)}`);
        }
        summary.textContent = parts.length ? parts.join(" · ") : "No workouts synced yet.";
      }
      if (exerciseSelect instanceof HTMLSelectElement) {
        for (const option of facets.exercises || []) {
          if (!option || typeof option.value !== "string") continue;
          const element = document.createElement("option");
          element.value = option.value;
          element.textContent = Number.isFinite(option.count)
            ? `${option.value} · ${formatInteger(option.count)}`
            : option.value;
          if (![...exerciseSelect.options].some((existing) => existing.value === option.value)) {
            exerciseSelect.append(element);
          }
        }
        exerciseSelect.value = selectedExercise || "";
      }
    } catch (error) {
      console.error("fitness facets failed", error);
      if (summary) summary.textContent = "Workout archive · live totals unavailable";
    }
  }

  async function loadResults({ updateUrl }) {
    window.clearTimeout(debounceTimer);
    const params = formParams();
    if (page > 1) params.set("page", String(page));
    if (updateUrl) replaceUrl(params);
    renderActive(params);

    requestController?.abort();
    requestController = new AbortController();
    const signal = requestController.signal;
    results.setAttribute("aria-busy", "true");
    root.classList.add("fitness-updating");
    if (!lastPayload) resultCount.textContent = "Fetching sets…";

    try {
      const data = await getJson("sets", params, signal);
      if (signal.aborted) return;
      lastPayload = data;
      page = positiveInteger(data.page) || page;
      renderResults(data);
      renderPager(data);
    } catch (error) {
      if (signal.aborted) return;
      console.error("fitness sets failed", error);
      renderError(error);
    } finally {
      if (!signal.aborted) {
        results.setAttribute("aria-busy", "false");
        root.classList.remove("fitness-updating");
      }
    }
  }

  async function getJson(path, params, signal) {
    const bases = [primaryApi];
    const local = ["localhost", "127.0.0.1", "::1"].includes(window.location.hostname);
    if (local && fallbackApi && fallbackApi !== primaryApi) bases.push(fallbackApi);
    let lastError = null;
    for (const base of bases) {
      const endpoint = new URL(`${base.replace(/\/$/, "")}/${path}`, window.location.href);
      endpoint.search = params.toString();
      try {
        const response = await fetch(endpoint, {
          headers: { Accept: "application/json" },
          signal,
        });
        if (!response.ok) {
          let message = `${response.status} ${response.statusText}`;
          try {
            const body = await response.json();
            if (body && typeof body.error === "string") message = body.error;
          } catch {
            // Keep the HTTP status when the response is not JSON.
          }
          const error = new Error(message);
          error.status = response.status;
          throw error;
        }
        return await response.json();
      } catch (error) {
        if (signal?.aborted) throw error;
        if (error?.status >= 400 && error.status < 500 && error.status !== 404) {
          throw error;
        }
        lastError = error;
      }
    }
    throw lastError || new Error("Workout database unavailable");
  }

  function restoreForm(params) {
    form.reset();
    for (const control of form.elements) {
      if (!(control instanceof HTMLInputElement || control instanceof HTMLSelectElement)) {
        continue;
      }
      const values = params.getAll(control.name);
      if (control instanceof HTMLInputElement && control.type === "checkbox") {
        control.checked = values.includes(control.value);
      } else if (values.length) {
        control.value = values[0];
      }
    }
    if (more) {
      const advanced = [
        "from",
        "to",
        "time_of_day",
        "weekday",
        "muscle",
        "equipment",
        "set_type",
        "min_load",
        "max_load",
        "min_reps",
        "max_reps",
        "max_effort",
        "has_record",
        "has_superset",
        "has_notes",
        "incomplete",
        "duration",
      ];
      const detailedMovements = new Set([
        "dip",
        "fly",
        "elbow-flexion",
        "elbow-extension",
        "shoulder-abduction",
        "shoulder-flexion",
        "shoulder-extension",
        "rear-delt",
        "shrug",
        "knee-flexion",
        "knee-extension",
        "hip-abduction",
        "hip-adduction",
        "calf-raise",
        "grip-wrist",
        "throw",
      ]);
      more.open =
        advanced.some((key) => params.has(key)) ||
        params.getAll("movement").some((value) => detailedMovements.has(value));
    }
  }

  function formParams() {
    const params = new URLSearchParams();
    for (const [key, raw] of new FormData(form).entries()) {
      const value = String(raw).trim();
      if (!value) continue;
      if (key === "per_page" && value === "20") continue;
      params.append(key, value);
    }
    return params;
  }

  function replaceUrl(params) {
    const canonical = new URLSearchParams(params);
    if (page <= 1) canonical.delete("page");
    const query = canonical.toString();
    const url = `${window.location.pathname}${query ? `?${query}` : ""}${window.location.hash}`;
    window.history.replaceState(null, "", url);
  }

  function renderActive(params) {
    activeFilters.replaceChildren();
    let count = 0;
    for (const [key, value] of params) {
      if (key === "page" || key === "per_page") continue;
      count += 1;
      const button = element("button", "fitness-active-chip");
      button.type = "button";
      button.dataset.filterKey = key;
      button.dataset.filterValue = value;
      const keyLabel = keyLabels[key];
      const valueLabel = valueLabels.get(value) || valueLabels.get(key) || value;
      button.textContent = `${keyLabel ? `${keyLabel}: ` : ""}${valueLabel} ×`;
      button.setAttribute("aria-label", `Remove ${button.textContent.slice(0, -2)} filter`);
      activeFilters.append(button);
    }
    clear.hidden = count === 0;
    if (count === 0) {
      const quiet = element("span", "fitness-no-filters", "All sets are included.");
      activeFilters.append(quiet);
    }
  }

  function renderResults(data) {
    const workouts = Array.isArray(data.workouts) ? data.workouts : [];
    const totalSets = Number(data.total_sets) || 0;
    const totalWorkouts = Number(data.total_workouts) || 0;
    const visibleSets = workouts.reduce(
      (sum, workout) => sum + (Array.isArray(workout.sets) ? workout.sets.length : 0),
      0,
    );
    resultCount.textContent = totalSets
      ? `${formatInteger(totalSets)} matching sets across ${formatInteger(totalWorkouts)} workouts · ${formatInteger(visibleSets)} on this page`
      : "No sets match these filters.";
    results.replaceChildren();

    if (!workouts.length) {
      const empty = element("div", "fitness-empty");
      empty.append(
        element("p", "fitness-empty-title", totalSets ? "This page is empty." : "No matching sets."),
        element(
          "p",
          "fitness-empty-copy",
          totalSets
            ? "Try the previous page."
            : "Loosen a movement, date, or numeric filter and the log will reappear.",
        ),
      );
      const reset = element("button", "fitness-empty-reset", "clear every filter");
      reset.type = "button";
      reset.addEventListener("click", () => clear.click());
      empty.append(reset);
      results.append(empty);
      return;
    }

    for (const workout of workouts) results.append(renderWorkout(workout));
  }

  function renderWorkout(workout) {
    const article = element("article", "fitness-workout rail-row");
    const stamp = element("div", "fitness-workout-stamp rail-stamp");
    const timing = workoutTiming(workout.started_at_local, workout.duration_seconds);
    const date = document.createElement("time");
    date.dateTime = String(workout.started_at_local || "").replace(" ", "T");
    date.append(
      element("span", "fitness-stamp-date", timing.date),
      element("span", "fitness-stamp-time", timing.range),
    );
    date.title = "Workout-local start and end time from the source data";
    stamp.append(date);

    const sheet = element("div", "fitness-sheet");
    const sheetHead = element("header", "fitness-sheet-head");
    const heading = element("h2", "fitness-workout-title", workout.title || "Untitled workout");
    const sets = Array.isArray(workout.sets) ? workout.sets : [];
    const meta = element(
      "p",
      "fitness-workout-meta",
      `${formatDuration(Number(workout.duration_seconds) || 0)} · ${sets.length} ${sets.length === 1 ? "set" : "sets"}`,
    );
    if (workout.duration_suspicious) {
      const warning = element("span", "fitness-timer-warning", "timer outlier");
      warning.title = "This source workout was left running for at least four hours, or recorded as zero.";
      meta.append(" · ", warning);
    }
    sheetHead.append(heading, meta);
    sheet.append(sheetHead);

    for (const copy of [workout.description, workout.notes]) {
      if (copy) sheet.append(element("p", "fitness-workout-note", copy));
    }

    const groups = contiguousExerciseGroups(sets);
    for (const group of groups) {
      const section = element("section", "fitness-exercise-group");
      const groupHead = element("div", "fitness-exercise-head");
      groupHead.append(
        element("h3", "fitness-exercise-name", group.name),
        element(
          "span",
          "fitness-exercise-count",
          `${group.sets.length} ${group.sets.length === 1 ? "set" : "sets"}`,
        ),
      );
      const list = element("ol", "fitness-set-list");
      for (const set of group.sets) list.append(renderSet(set));
      section.append(groupHead, list);
      sheet.append(section);
    }

    article.append(stamp, sheet);
    return article;
  }

  function renderSet(set) {
    const item = element("li", "fitness-set-row");
    item.append(element("span", "fitness-set-ordinal", String(set.ordinal || 0).padStart(2, "0")));
    item.append(element("span", "fitness-set-prescription", prescription(set)));

    const details = element("span", "fitness-set-details");
    const bits = [];
    if (set.set_type) bits.push(setTypeLabel(set.set_type));
    if (set.effort_hundredths !== null && set.effort_hundredths !== undefined) {
      bits.push(`RIR/RPE ${formatScaled(set.effort_hundredths, 100)}`);
    }
    if (set.set_time_seconds !== null && set.set_time_seconds !== undefined) {
      bits.push(formatDuration(set.set_time_seconds));
    }
    if (set.distance_milli !== null && set.distance_milli !== undefined) {
      bits.push(`distance ${formatScaled(set.distance_milli, 1000)}`);
    }
    if (set.superset_id !== null && set.superset_id !== undefined) {
      bits.push(`superset ${set.superset_id}`);
    }
    if (
      set.reps === null &&
      set.distance_milli === null &&
      set.set_time_seconds === null
    ) {
      bits.push("incomplete");
    }
    details.textContent = bits.join(" · ");
    item.append(details);

    const records = element("span", "fitness-set-records");
    for (const record of Array.isArray(set.records) ? set.records : []) {
      const badge = element(
        "span",
        `fitness-record fitness-record-${record.level || "bronze"}`,
        `${record.level || "PR"} ${recordKindLabel(record.kind)}`,
      );
      records.append(badge);
    }
    item.append(records);

    if (set.exercise_note) {
      item.append(element("span", "fitness-set-note", set.exercise_note));
    }
    return item;
  }

  function renderPager(data) {
    const perPage = positiveInteger(data.per_page) || 20;
    const totalWorkouts = Number(data.total_workouts) || 0;
    const totalPages = Math.max(1, Math.ceil(totalWorkouts / perPage));
    const current = Math.min(Math.max(positiveInteger(data.page) || page, 1), totalPages);
    pager.replaceChildren();
    if (totalPages <= 1) {
      pager.hidden = true;
      return;
    }
    pager.hidden = false;
    pager.append(pageLink(current - 1, "← newer", current === 1));
    for (const part of pageWindow(current, totalPages)) {
      if (part === "…") {
        pager.append(element("span", "fitness-page-gap", part));
      } else if (part === current) {
        const here = element("span", "fitness-page-current", String(part));
        here.setAttribute("aria-current", "page");
        pager.append(here);
      } else {
        pager.append(pageLink(part, String(part), false));
      }
    }
    pager.append(pageLink(current + 1, "older →", current === totalPages));
  }

  function pageLink(target, label, disabled) {
    if (disabled) {
      const span = element("span", "fitness-page-link fitness-page-disabled", label);
      span.setAttribute("aria-disabled", "true");
      return span;
    }
    const link = element("a", "fitness-page-link", label);
    const params = formParams();
    if (target > 1) params.set("page", String(target));
    const href = new URL(window.location.pathname, window.location.href);
    href.search = params.toString();
    link.href = `${href.pathname}${href.search}`;
    link.dataset.fitnessPage = String(target);
    return link;
  }

  function renderError(error) {
    const invalidFilter = error?.status >= 400 && error.status < 500;
    if (invalidFilter) {
      resultCount.textContent = `A filter was rejected · ${error.message}`;
      if (lastPayload) return;
      results.replaceChildren();
      const notice = element("div", "fitness-empty fitness-error");
      notice.append(
        element("p", "fitness-empty-title", "That filter combination is not valid."),
        element("p", "fitness-empty-copy", error.message),
      );
      const reset = element("button", "fitness-empty-reset", "clear every filter");
      reset.type = "button";
      reset.addEventListener("click", () => clear.click());
      notice.append(reset);
      results.append(notice);
      pager.hidden = true;
      return;
    }
    resultCount.textContent = lastPayload
      ? "Workout database is unreachable · the last loaded page is still shown."
      : "Workout database is unreachable.";
    if (lastPayload) return;
    results.replaceChildren();
    const notice = element("div", "fitness-empty fitness-error");
    notice.append(
      element("p", "fitness-empty-title", "The set log did not load."),
      element("p", "fitness-empty-copy", "The filters are intact. Try the database again."),
    );
    const retry = element("button", "fitness-empty-reset", "retry");
    retry.type = "button";
    retry.addEventListener("click", () => loadResults({ updateUrl: false }));
    notice.append(retry);
    results.append(notice);
    pager.hidden = true;
  }

  function contiguousExerciseGroups(sets) {
    const groups = [];
    for (const set of sets) {
      const name = set.exercise_name || "Unknown exercise";
      const last = groups[groups.length - 1];
      if (last && last.name === name) last.sets.push(set);
      else groups.push({ name, sets: [set] });
    }
    return groups;
  }

  function prescription(set) {
    const hasLoad = set.weight_milli !== null && set.weight_milli !== undefined;
    const hasReps = set.reps !== null && set.reps !== undefined;
    if (hasLoad && hasReps) return `${formatScaled(set.weight_milli, 1000)} × ${set.reps}`;
    if (hasReps) return `${set.reps} reps`;
    if (hasLoad) return `load ${formatScaled(set.weight_milli, 1000)}`;
    if (set.distance_milli !== null && set.distance_milli !== undefined) {
      return `distance ${formatScaled(set.distance_milli, 1000)}`;
    }
    if (set.set_time_seconds !== null && set.set_time_seconds !== undefined) {
      return formatDuration(set.set_time_seconds);
    }
    return "not recorded";
  }

  function workoutTiming(localStart, durationSeconds) {
    const match = /^(\d{4})-(\d{2})-(\d{2})[ T](\d{2}):(\d{2}):(\d{2})$/.exec(
      String(localStart || ""),
    );
    if (!match) return { date: localStart || "unknown date", range: "time unavailable" };
    const [, year, month, day, hour, minute, second] = match;
    const start = new Date(
      Date.UTC(+year, +month - 1, +day, +hour, +minute, +second),
    );
    const end = new Date(start.getTime() + Math.max(0, Number(durationSeconds) || 0) * 1000);
    const dateFormat = new Intl.DateTimeFormat("en-US", {
      month: "short",
      day: "numeric",
      year: "numeric",
      timeZone: "UTC",
    });
    const timeFormat = new Intl.DateTimeFormat("en-US", {
      hour: "numeric",
      minute: "2-digit",
      timeZone: "UTC",
    });
    const sameDay = start.toISOString().slice(0, 10) === end.toISOString().slice(0, 10);
    return {
      date: dateFormat.format(start),
      range: sameDay
        ? `${timeFormat.format(start)}–${timeFormat.format(end)}`
        : `${timeFormat.format(start)}–${dateFormat.format(end)} ${timeFormat.format(end)}`,
    };
  }

  function pageWindow(current, total) {
    const pages = new Set([1, total]);
    for (let value = current - 2; value <= current + 2; value += 1) {
      if (value >= 1 && value <= total) pages.add(value);
    }
    const sorted = [...pages].sort((a, b) => a - b);
    const output = [];
    for (let index = 0; index < sorted.length; index += 1) {
      if (index && sorted[index] - sorted[index - 1] > 1) output.push("…");
      output.push(sorted[index]);
    }
    return output;
  }

  function deleteOne(params, key, value) {
    const kept = params.getAll(key).filter((entry) => entry !== value);
    params.delete(key);
    for (const entry of kept) params.append(key, entry);
  }

  function setTypeLabel(value) {
    return valueLabels.get(value) || String(value).toLowerCase().replaceAll("_", " ");
  }

  function recordKindLabel(value) {
    return value === "max-weight" ? "max load" : String(value || "PR").toUpperCase();
  }

  function formatDuration(seconds) {
    const total = Math.max(0, Number(seconds) || 0);
    const hours = Math.floor(total / 3600);
    const minutes = Math.floor((total % 3600) / 60);
    const secs = Math.floor(total % 60);
    if (hours) return `${hours}h ${String(minutes).padStart(2, "0")}m`;
    if (minutes) return `${minutes}m ${String(secs).padStart(2, "0")}s`;
    return `${secs}s`;
  }

  function formatScaled(value, scale) {
    const number = Number(value) / scale;
    return new Intl.NumberFormat("en-US", { maximumFractionDigits: 3 }).format(number);
  }

  function formatInteger(value) {
    return new Intl.NumberFormat("en-US").format(Number(value) || 0);
  }

  function formatMonth(value) {
    const match = /^(\d{4})-(\d{2})/.exec(String(value));
    if (!match) return value;
    const date = new Date(Date.UTC(+match[1], +match[2] - 1, 1));
    return new Intl.DateTimeFormat("en-US", {
      month: "short",
      year: "numeric",
      timeZone: "UTC",
    }).format(date);
  }

  function positiveInteger(value) {
    const number = Number(value);
    return Number.isInteger(number) && number > 0 ? number : null;
  }

  function element(tag, className, text) {
    const node = document.createElement(tag);
    if (className) node.className = className;
    if (text !== undefined) node.textContent = text;
    return node;
  }
}
