//! A no-JavaScript, local-date calendar view of the lifting archive's daily
//! volume points. The Worker groups sets by the workout's source-local date;
//! this module only lays those already-aggregated days into calendar weeks.

use std::collections::BTreeMap;

use topcoat::{
    Result,
    view::{class, component, view},
};

use super::{META_LABEL, data::CalendarDay};

const WEEK_COUNT: usize = 53;
const DAYS_PER_WEEK: usize = 7;
const CELL_COUNT: usize = WEEK_COUNT * DAYS_PER_WEEK;

// Tailwind vocab for the calendar. Utilities stay whole per line for the
// build-time class scanner.
const HEAT_NOTE: &str = "font-meta text-[0.7rem] leading-[1.55] text-muted";
/// Day squares tint oxide by the `--fitness-heat-alpha` each cell sets inline.
const HEAT_FILL: &str =
    "bg-[color-mix(in_srgb,var(--color-oxide)_var(--fitness-heat-alpha,0%),var(--color-card))]";
const LEGEND_CELL: &str = "w-[0.625rem] h-[0.625rem] sm:w-[0.72rem] sm:h-[0.72rem] \
     rounded-[0.12rem] border border-hairline/88";
const CELL: &str = "block rounded-[0.12rem] border \
     transition-[background-color,border-color,box-shadow,transform] duration-[140ms] ease-[ease]";
const CELL_BORDER: &str = "border-hairline/88";
/// A logged day whose sets scored zero points keeps a visible dashed ring.
const CELL_BORDER_ZERO: &str = "border-dashed \
     border-[color-mix(in_srgb,var(--color-oxide)_55%,var(--color-hairline))]";
const CELL_HOVER: &str = "hover:border-oxide \
     hover:shadow-[0_0_0_1px_color-mix(in_srgb,var(--color-oxide)_25%,transparent)] \
     hover:-translate-y-px focus-visible:z-[1] focus-visible:outline-solid \
     focus-visible:outline-2 focus-visible:outline-oxide focus-visible:outline-offset-2";

/// The volume calendar. It is deliberately an owned prop: callers can pass
/// the successful calendar API payload straight through without adding a
/// browser runtime or serializing data into the page.
#[component]
pub(super) async fn calendar_heatmap(days: Vec<CalendarDay>) -> Result {
    let Some(calendar) = Calendar::from_days(&days) else {
        return view! {
            <section aria-labelledby="fitness-heatmap-title">
                <header
                    class="flex flex-wrap items-end justify-between gap-y-[0.8rem] gap-x-5"
                >
                    <div>
                        <p class=(META_LABEL)>"training volume"</p>
                        <h2
                            id="fitness-heatmap-title"
                            class="font-display text-2xl font-semibold"
                        >
                            "Volume points"
                        </h2>
                    </div>
                </header>
                <p class=(class!(HEAT_NOTE, "mt-[0.8rem]"))>
                    "No lifting days are available yet."
                </p>
            </section>
        };
    };

    let ending = calendar.latest.format_short();
    let subtitle = format!("53 weeks ending {ending} · filled days open the lifts logged that day");
    let navigation_label = format!(
        "Volume points by day for the 53 weeks ending {ending}. {} logged days are links to their lifts.",
        calendar.logged_days,
    );
    let legend_styles: Vec<String> = (0..=4).map(heat_style).collect();

    view! {
        <section aria-labelledby="fitness-heatmap-title">
            <header class="flex flex-wrap items-end justify-between gap-y-[0.8rem] gap-x-5">
                <div>
                    <p class=(META_LABEL)>"training volume"</p>
                    <h2
                        id="fitness-heatmap-title"
                        class="font-display text-2xl font-semibold"
                    >
                        "Volume points"
                    </h2>
                    <p class=(class!(HEAT_NOTE, "mt-[0.3rem]"))>(subtitle.as_str())</p>
                </div>
                <div
                    class="inline-flex items-center gap-[0.22rem] font-meta text-[0.61rem] \
                         leading-none uppercase text-muted"
                    aria-label="Volume-point intensity: 1 to 24, 25 to 44, 45 to 64, and 65 or more"
                >
                    <span class="mr-[0.12rem]">"less"</span>
                    for style in legend_styles.iter() {
                        <span
                            class=(class!(LEGEND_CELL, HEAT_FILL))
                            style=(style.as_str())
                            aria-hidden="true"
                        >

                        </span>
                    }
                    <span class="ml-[0.12rem]">"more"</span>
                </div>
            </header>

            // When the chart's minimum width overflows a narrow screen, the
            // rtl scroll direction starts the view at the newest (rightmost)
            // days; the chart flips back to ltr so its dates still run
            // normally.
            <div
                class="mt-[0.9rem] overflow-x-auto overscroll-x-contain pt-[0.1rem] \
                     pb-[0.45rem] [direction:rtl]"
            >
                <div
                    class="grid w-full min-w-[34rem] grid-cols-[1.45rem_minmax(0,1fr)] \
                         grid-rows-[1.1rem_auto] gap-x-[0.4rem] [direction:ltr]"
                >
                    <div
                        class="col-start-2 row-start-1 grid \
                             grid-cols-[repeat(53,minmax(0,1fr))] gap-x-[0.16rem] items-end \
                             font-meta text-[0.59rem] leading-none whitespace-nowrap \
                             text-muted sm:gap-x-[0.2rem]"
                        aria-hidden="true"
                    >
                        for label in calendar.month_labels.iter() {
                            <span style=(label.style.as_str())>
                                (label.label.as_str())
                            </span>
                        }
                    </div>
                    <div
                        class="col-start-1 row-start-2 grid \
                             grid-rows-[repeat(7,minmax(0,1fr))] items-center self-stretch \
                             text-right font-meta text-[0.58rem] leading-none text-muted"
                        aria-hidden="true"
                    >
                        <span></span>
                        <span>"M"</span>
                        <span></span>
                        <span>"W"</span>
                        <span></span>
                        <span>"F"</span>
                        <span></span>
                    </div>
                    <nav
                        class="col-start-2 row-start-2 grid \
                             grid-cols-[repeat(53,minmax(0,1fr))] \
                             grid-rows-[repeat(7,minmax(0,1fr))] grid-flow-col \
                             gap-[0.16rem] aspect-[53/7] sm:gap-[0.2rem]"
                        aria-label=(navigation_label.as_str())
                    >
                        for cell in calendar.cells.iter() {
                            if let Some(href) = &cell.href {
                                <a
                                    class=(class!(CELL, HEAT_FILL, cell.border, CELL_HOVER))
                                    href=(href.as_str())
                                    title=(cell.label.as_str())
                                    aria-label=(cell.label.as_str())
                                    style=(cell.style.as_str())
                                >

                                </a>
                            } else {
                                <span
                                    class=(class!(CELL, HEAT_FILL, cell.border))
                                    title=(cell.label.as_str())
                                    aria-hidden="true"
                                    style=(cell.style.as_str())
                                >

                                </span>
                            }
                        }
                    </nav>
                </div>
            </div>
            <p class=(class!(HEAT_NOTE, "mt-[0.1rem]"))>
                "Volume points use the same effort-point score as the set log. Hover a square for its exact total."
            </p>
        </section>
    }
}

struct Calendar {
    latest: Date,
    logged_days: usize,
    cells: Vec<HeatmapCell>,
    month_labels: Vec<MonthLabel>,
}

impl Calendar {
    fn from_days(days: &[CalendarDay]) -> Option<Self> {
        let mut points_by_day = BTreeMap::new();
        for day in days {
            let date = Date::parse(&day.date)?;
            let points = points_by_day.entry(date).or_insert(0_u32);
            *points = points.saturating_add(day.volume_points);
        }
        let latest = *points_by_day.last_key_value()?.0;
        // End on Saturday so every Sunday–Saturday column means what its
        // weekday labels say. Dates after the latest logged day remain visibly
        // empty padding in that final week.
        let end = latest.add_days((DAYS_PER_WEEK - 1 - latest.weekday_sunday0()) as i64);
        let start = end.add_days(-(CELL_COUNT as i64 - 1));
        let mut cells = Vec::with_capacity(CELL_COUNT);
        let mut logged_days = 0;
        for offset in 0..CELL_COUNT {
            let date = start.add_days(offset as i64);
            let points = points_by_day.get(&date).copied().unwrap_or(0);
            let has_lift = points_by_day.contains_key(&date);
            if has_lift {
                logged_days += 1;
            }
            cells.push(HeatmapCell::new(date, points, has_lift));
        }
        let month_labels = MonthLabel::from_cells(&cells);
        Some(Self {
            latest,
            logged_days,
            cells,
            month_labels,
        })
    }
}

struct HeatmapCell {
    date: Date,
    border: &'static str,
    href: Option<String>,
    label: String,
    style: String,
}

impl HeatmapCell {
    fn new(date: Date, points: u32, has_lift: bool) -> Self {
        let intensity = intensity(points);
        let border = if has_lift && points == 0 {
            CELL_BORDER_ZERO
        } else {
            CELL_BORDER
        };
        let date_label = date.format_long();
        let points_label = format!(
            "{points} volume {}",
            if points == 1 { "point" } else { "points" }
        );
        let label = if has_lift {
            format!("{date_label}: {points_label}. View lifts from this day.")
        } else {
            format!("{date_label}: no volume points")
        };
        Self {
            date,
            border,
            href: has_lift.then(|| {
                let iso = date.iso();
                format!("/lifting/log?from={iso}&to={iso}#set-log")
            }),
            label,
            style: heat_style(intensity),
        }
    }
}

struct MonthLabel {
    label: String,
    style: String,
}

impl MonthLabel {
    fn from_cells(cells: &[HeatmapCell]) -> Vec<Self> {
        let mut labels = Vec::new();
        let mut last_column = None;
        for (index, cell) in cells.iter().enumerate() {
            let column = index / DAYS_PER_WEEK;
            let starts_month = column == 0 || cell.date.day == 1;
            // A label needs about three week columns to remain legible. This
            // naturally skips a month that starts immediately after the chart
            // range begins while keeping later labels in their real column.
            let has_room = last_column.is_none_or(|previous| column >= previous + 3);
            if starts_month && has_room {
                labels.push(Self {
                    label: month_name(cell.date.month).to_string(),
                    style: format!("grid-column: {}", column + 1),
                });
                last_column = Some(column);
            }
        }
        labels
    }
}

/// Four fixed bands keep the color meaning stable when the archive grows or a
/// reader follows a day link. They place the audited archive's typical day in
/// the middle of the scale instead of letting one unusually long workout wash
/// every other cell out.
fn intensity(points: u32) -> u8 {
    match points {
        0 => 0,
        1..=24 => 1,
        25..=44 => 2,
        45..=64 => 3,
        _ => 4,
    }
}

fn heat_style(intensity: u8) -> String {
    let alpha = match intensity {
        0 => 0,
        1 => 18,
        2 => 36,
        3 => 62,
        _ => 92,
    };
    format!("--fitness-heat-alpha: {alpha}%")
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Date {
    year: i64,
    month: u32,
    day: u32,
}

impl Date {
    fn parse(value: &str) -> Option<Self> {
        let bytes = value.as_bytes();
        if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
            return None;
        }
        let year = decimal(&bytes[0..4])? as i64;
        let month = decimal(&bytes[5..7])?;
        let day = decimal(&bytes[8..10])?;
        if !(1..=12).contains(&month) {
            return None;
        }
        (1..=days_in_month(year, month))
            .contains(&day)
            .then_some(Self { year, month, day })
    }

    fn add_days(self, days: i64) -> Self {
        Self::from_days(days_from_civil(self.year, self.month, self.day) + days)
    }

    fn from_days(days: i64) -> Self {
        let (year, month, day) = civil_from_days(days);
        Self { year, month, day }
    }

    fn weekday_sunday0(self) -> usize {
        // 1970-01-01 was Thursday (4 when Sunday is zero).
        (days_from_civil(self.year, self.month, self.day) + 4).rem_euclid(7) as usize
    }

    fn iso(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    fn format_short(self) -> String {
        format!("{} {}, {}", month_name(self.month), self.day, self.year)
    }

    fn format_long(self) -> String {
        format!(
            "{}, {} {}, {}",
            weekday_name(self.weekday_sunday0()),
            month_name(self.month),
            self.day,
            self.year,
        )
    }
}

fn decimal(bytes: &[u8]) -> Option<u32> {
    bytes.iter().try_fold(0_u32, |value, byte| {
        byte.is_ascii_digit()
            .then(|| value * 10 + u32::from(*byte - b'0'))
    })
}

fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        4 | 6 | 9 | 11 => 30,
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        _ => 31,
    }
}

fn month_name(month: u32) -> &'static str {
    match month {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => "",
    }
}

fn weekday_name(weekday: usize) -> &'static str {
    match weekday {
        0 => "Sunday",
        1 => "Monday",
        2 => "Tuesday",
        3 => "Wednesday",
        4 => "Thursday",
        5 => "Friday",
        6 => "Saturday",
        _ => "",
    }
}

// Howard Hinnant's civil-calendar conversion, with day zero at 1970-01-01.
fn days_from_civil(mut year: i64, month: u32, day: u32) -> i64 {
    year -= i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let adjusted_month = i64::from(month) + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * adjusted_month + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn civil_from_days(mut days: i64) -> (i64, u32, u32) {
    days += 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_piece = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_piece + 2) / 5 + 1;
    let month = month_piece + if month_piece < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    (year, month as u32, day as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(date: &str, volume_points: u32) -> CalendarDay {
        CalendarDay {
            date: date.to_string(),
            volume_points,
        }
    }

    #[test]
    fn grid_is_53_complete_sunday_to_saturday_weeks_anchored_to_latest_day() {
        let calendar = Calendar::from_days(&[day("2026-07-21", 42)]).expect("calendar");

        assert_eq!(calendar.cells.len(), 371);
        assert_eq!(calendar.latest.iso(), "2026-07-21");
        assert_eq!(calendar.month_labels[0].label, "Jul");
        assert_eq!(calendar.month_labels[0].style, "grid-column: 1");
        assert!(
            calendar
                .month_labels
                .iter()
                .any(|label| label.label == "Jan")
        );
        assert_eq!(Date::parse("2025-07-20").unwrap().weekday_sunday0(), 0);
        assert_eq!(Date::parse("2026-07-25").unwrap().weekday_sunday0(), 6);
        assert_eq!(
            calendar.cells[0].label,
            "Sunday, Jul 20, 2025: no volume points"
        );
        assert_eq!(
            calendar.cells[366].label,
            "Tuesday, Jul 21, 2026: 42 volume points. View lifts from this day."
        );
        assert_eq!(
            calendar.cells[366].href.as_deref(),
            Some("/lifting/log?from=2026-07-21&to=2026-07-21#set-log")
        );
        assert!(calendar.cells[367].href.is_none());
    }

    #[test]
    fn intensity_bands_are_fixed_at_their_inclusive_edges() {
        assert_eq!(intensity(0), 0);
        assert_eq!(intensity(1), 1);
        assert_eq!(intensity(24), 1);
        assert_eq!(intensity(25), 2);
        assert_eq!(intensity(44), 2);
        assert_eq!(intensity(45), 3);
        assert_eq!(intensity(64), 3);
        assert_eq!(intensity(65), 4);
        assert_eq!(intensity(u32::MAX), 4);
    }

    #[test]
    fn duplicate_calendar_days_sum_without_losing_their_link() {
        let calendar =
            Calendar::from_days(&[day("2024-02-29", 20), day("2024-02-29", 25)]).expect("calendar");
        let leap_day = calendar
            .cells
            .iter()
            .find(|cell| {
                cell.href
                    .as_deref()
                    .is_some_and(|href| href.contains("2024-02-29"))
            })
            .expect("leap day cell");
        assert!(leap_day.label.contains("45 volume points"));
        assert_eq!(leap_day.style, heat_style(3));
    }
}
