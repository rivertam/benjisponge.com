//! View models for workouts, set markers, records, and archive pagination.

use super::{
    data as fitness,
    filters::{Filters, SET_TYPES, lookup},
    format::{format_duration, format_scaled, workout_datetime, workout_timing},
};
use crate::util::urlencode;

pub(super) struct WorkoutCard<'a> {
    pub(super) href: String,
    pub(super) title: &'a str,
    pub(super) datetime: String,
    pub(super) date: String,
    pub(super) time_range: String,
    pub(super) duration: String,
    pub(super) duration_suspicious: bool,
    pub(super) description: Option<&'a str>,
    pub(super) notes: Option<&'a str>,
    pub(super) set_count: usize,
    pub(super) groups: Vec<ExerciseGroup<'a>>,
}

impl<'a> From<&'a fitness::Workout> for WorkoutCard<'a> {
    fn from(workout: &'a fitness::Workout) -> Self {
        let timing = workout_timing(
            &workout.started_at_local,
            &workout.ended_at_local,
            workout.eastern_offset_minutes,
            workout.end_eastern_offset_minutes,
        );
        Self {
            href: workout_url(&workout.path),
            title: &workout.title,
            datetime: workout_datetime(&workout.started_at_local, workout.eastern_offset_minutes),
            date: timing.date,
            time_range: timing.range,
            duration: format_duration(workout.duration_seconds),
            duration_suspicious: workout.duration_suspicious,
            description: workout.description.as_deref(),
            notes: workout.notes.as_deref(),
            set_count: workout.sets.len(),
            groups: exercise_groups(&workout.sets),
        }
    }
}

pub(super) fn workout_url(path: &str) -> String {
    format!("/lifting/{}", urlencode(path))
}

pub(super) struct ExerciseGroup<'a> {
    pub(super) name: &'a str,
    pub(super) volume_points: u32,
    pub(super) rows: Vec<SetRow<'a>>,
}

pub(super) struct SetRow<'a> {
    pub(super) marker_class: &'static str,
    pub(super) marker_text: Option<String>,
    pub(super) marker_title: String,
    pub(super) marker_label: String,
    pub(super) point_styles: Vec<String>,
    pub(super) prescription: String,
    pub(super) details: String,
    pub(super) records: Vec<RecordBadge>,
    pub(super) note: Option<&'a str>,
}

impl<'a> From<&'a fitness::Set> for SetRow<'a> {
    fn from(set: &'a fitness::Set) -> Self {
        let ordinal = format!("{:02}", set.ordinal);
        let (marker_class, marker_text, marker_title, marker_label, point_styles) =
            match set.set_type.as_str() {
                "WARMUP_SET" => (
                    "fitness-set-marker fitness-set-marker-warmup",
                    Some("W".to_string()),
                    format!("Set {ordinal} · warm-up"),
                    format!("Set {ordinal}, warm-up"),
                    Vec::new(),
                ),
                "FAILURE_SET" => (
                    "fitness-set-marker fitness-set-marker-failure",
                    Some("F".to_string()),
                    format!("Set {ordinal} · failure"),
                    format!("Set {ordinal}, failure"),
                    Vec::new(),
                ),
                _ => {
                    let points = effort_points(set.effort_hundredths);
                    let effort = set
                        .effort_hundredths
                        .map(|value| format!("RIR/RPE {}", format_scaled(value, 100)))
                        .unwrap_or_else(|| "RIR/RPE not recorded".to_string());
                    let styles = (0..points)
                        .map(|index| format!("--point-angle: {}deg", index * 360 / points.max(1)))
                        .collect();
                    (
                        "fitness-set-marker fitness-set-marker-points",
                        set.effort_hundredths.is_none().then_some(ordinal.clone()),
                        format!("Set {ordinal} · {effort} · {points} of 5 points"),
                        format!("Set {ordinal}, {effort}, {points} of 5 points"),
                        styles,
                    )
                }
            };

        Self {
            marker_class,
            marker_text,
            marker_title,
            marker_label,
            point_styles,
            prescription: prescription(set),
            details: set_details(set),
            records: set.records.iter().map(RecordBadge::from).collect(),
            note: set.exercise_note.as_deref(),
        }
    }
}

pub(super) struct RecordBadge {
    pub(super) class: String,
    pub(super) label: String,
}

impl From<&fitness::Record> for RecordBadge {
    fn from(record: &fitness::Record) -> Self {
        let level = match record.level.as_str() {
            "gold" | "silver" | "bronze" => record.level.as_str(),
            _ => "bronze",
        };
        let rank = match level {
            "gold" => "PR",
            "silver" => "#2",
            _ => "#3",
        };
        let kind = match record.kind.as_str() {
            "max-weight" => "max load".to_string(),
            value => value.to_uppercase(),
        };
        Self {
            class: format!("fitness-record fitness-record-{level}"),
            label: format!("{kind} {rank}"),
        }
    }
}

pub(super) struct Pager {
    pub(super) newer: Option<String>,
    pub(super) older: Option<String>,
    pub(super) current: usize,
    pub(super) parts: Vec<Option<usize>>,
}

fn exercise_groups(sets: &[fitness::Set]) -> Vec<ExerciseGroup<'_>> {
    let mut groups = Vec::new();
    let mut start = 0;
    while start < sets.len() {
        let name = sets[start].exercise_name.as_str();
        let mut end = start + 1;
        while end < sets.len() && sets[end].exercise_name == name {
            end += 1;
        }
        let slice = &sets[start..end];
        groups.push(ExerciseGroup {
            name,
            volume_points: slice.iter().map(set_volume_points).sum(),
            rows: slice.iter().map(SetRow::from).collect(),
        });
        start = end;
    }
    groups
}

fn set_volume_points(set: &fitness::Set) -> u32 {
    match set.set_type.as_str() {
        "FAILURE_SET" => 6,
        "WARMUP_SET" => 0,
        _ => effort_points(set.effort_hundredths),
    }
}

/// Missing effort follows the intended low/default branch rather than
/// JavaScript's surprising `Number(null) == 0` coercion.
fn effort_points(effort: Option<u64>) -> u32 {
    match effort {
        Some(0) => 5,
        Some(100) => 4,
        Some(200) => 3,
        Some(_) | None => 2,
    }
}

fn prescription(set: &fitness::Set) -> String {
    match (set.weight_milli, set.reps) {
        (Some(load), Some(reps)) => format!("{} × {reps}", format_scaled(load, 1_000)),
        (None, Some(reps)) => format!("{reps} reps"),
        (Some(load), None) => format!("load {}", format_scaled(load, 1_000)),
        (None, None) => {
            if let Some(distance) = set.distance_milli {
                format!("distance {}", format_scaled(distance, 1_000))
            } else if let Some(seconds) = set.set_time_seconds {
                format_duration(seconds)
            } else {
                "not recorded".to_string()
            }
        }
    }
}

fn set_details(set: &fitness::Set) -> String {
    let mut details = Vec::new();
    if !matches!(
        set.set_type.as_str(),
        "NORMAL_SET" | "WARMUP_SET" | "FAILURE_SET" | ""
    ) {
        details.push(set_type_label(&set.set_type));
    }
    if let Some(effort) = set.effort_hundredths {
        details.push(format!("RIR/RPE {}", format_scaled(effort, 100)));
    }

    let load_or_reps_is_primary = set.weight_milli.is_some() || set.reps.is_some();
    if let Some(seconds) = set.set_time_seconds
        && (load_or_reps_is_primary || set.distance_milli.is_some())
    {
        details.push(format_duration(seconds));
    }
    if let Some(distance) = set.distance_milli
        && load_or_reps_is_primary
    {
        details.push(format!("distance {}", format_scaled(distance, 1_000)));
    }
    if let Some(superset) = set.superset_id {
        details.push(format!("superset {superset}"));
    }
    if set.reps.is_none() && set.distance_milli.is_none() && set.set_time_seconds.is_none() {
        details.push("incomplete".to_string());
    }
    details.join(" · ")
}

fn set_type_label(value: &str) -> String {
    lookup(SET_TYPES, value)
        .map(str::to_string)
        .unwrap_or_else(|| value.to_ascii_lowercase().replace('_', " "))
}

pub(super) fn make_pager(page: &fitness::SetPage, filters: &Filters) -> Option<Pager> {
    let pages = total_pages(page);
    if pages <= 1 {
        return None;
    }
    let current = page.page.clamp(1, pages);
    Some(Pager {
        newer: (current > 1).then(|| filters.page_url(current - 1)),
        older: (current < pages).then(|| filters.page_url(current + 1)),
        current,
        parts: page_window(current, pages),
    })
}

pub(super) fn total_pages(page: &fitness::SetPage) -> usize {
    (page.total_workouts as usize)
        .div_ceil(page.per_page.max(1))
        .max(1)
}

fn page_window(current: usize, total: usize) -> Vec<Option<usize>> {
    let mut pages = vec![1, total];
    pages.extend(current.saturating_sub(2).max(1)..=(current + 2).min(total));
    pages.sort_unstable();
    pages.dedup();
    let mut output = Vec::new();
    for page in pages {
        if output
            .last()
            .and_then(|value| *value)
            .is_some_and(|previous| page > previous + 1)
        {
            output.push(None);
        }
        output.push(Some(page));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set() -> fitness::Set {
        fitness::Set {
            ordinal: 1,
            exercise_name: "Plank".to_string(),
            exercise_note: None,
            superset_id: None,
            weight_milli: None,
            reps: None,
            effort_hundredths: None,
            distance_milli: None,
            set_time_seconds: None,
            set_type: "NORMAL_SET".to_string(),
            records: Vec::new(),
        }
    }

    fn workout() -> fitness::Workout {
        fitness::Workout {
            path: "2026-07-21T17-03-00-04-00".to_string(),
            title: "Lift".to_string(),
            started_at_local: "2026-07-21 17:03:00".to_string(),
            ended_at_local: "2026-07-21 18:03:00".to_string(),
            eastern_offset_minutes: -240,
            end_eastern_offset_minutes: -240,
            duration_seconds: 3_600,
            duration_suspicious: false,
            notes: None,
            description: None,
            sets: vec![set()],
        }
    }

    #[test]
    fn missing_effort_is_not_coerced_to_zero() {
        assert_eq!(effort_points(None), 2);
        assert_eq!(effort_points(Some(0)), 5);
    }

    #[test]
    fn pager_window_has_compact_gaps() {
        assert_eq!(
            page_window(6, 12),
            vec![
                Some(1),
                None,
                Some(4),
                Some(5),
                Some(6),
                Some(7),
                Some(8),
                None,
                Some(12)
            ]
        );
        assert_eq!(
            page_window(1, 12),
            vec![Some(1), Some(2), Some(3), None, Some(12)]
        );
    }

    #[test]
    fn fallback_time_and_distance_are_not_repeated_in_details() {
        let mut timed = set();
        timed.set_time_seconds = Some(75);
        timed.superset_id = Some(1);
        assert_eq!(prescription(&timed), "1m 15s");
        assert_eq!(set_details(&timed), "superset 1");

        let mut distance = set();
        distance.distance_milli = Some(2_500);
        distance.set_time_seconds = Some(60);
        assert_eq!(prescription(&distance), "distance 2.5");
        assert_eq!(set_details(&distance), "1m 00s");
    }

    #[test]
    fn workout_links_use_the_workers_canonical_public_path() {
        let workout = workout();
        let card = WorkoutCard::from(&workout);
        assert_eq!(card.href, "/lifting/2026-07-21T17-03-00-04-00");
    }

    #[test]
    fn worker_paths_are_escaped_as_one_url_segment() {
        assert_eq!(workout_url("manual:abc 123"), "/lifting/manual%3Aabc%20123");
    }
}
