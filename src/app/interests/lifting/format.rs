//! Display formatting for archive totals, set values, and local workout time.

use super::data as fitness;

pub(super) fn format_archive_summary(facets: &fitness::Facets) -> String {
    let mut parts = vec![
        format!("{} sets", format_integer(facets.summary.sets)),
        format!("{} workouts", format_integer(facets.summary.workouts)),
    ];
    if let (Some(from), Some(to)) = (&facets.summary.min_date, &facets.summary.max_date) {
        parts.push(format!("{}–{}", format_month(from), format_month(to)));
    }
    parts.join(" · ")
}

pub(super) fn format_integer(value: impl Into<u64>) -> String {
    let digits = value.into().to_string();
    let mut output = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, character) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            output.push(',');
        }
        output.push(character);
    }
    output
}

pub(super) fn format_scaled(value: u64, scale: u64) -> String {
    let whole = value / scale;
    let remainder = value % scale;
    let mut output = format_integer(whole);
    if remainder > 0 {
        let width = scale.ilog10() as usize;
        let fraction = format!("{remainder:0width$}")
            .trim_end_matches('0')
            .to_string();
        output.push('.');
        output.push_str(&fraction);
    }
    output
}

pub(super) fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3_600;
    let minutes = seconds % 3_600 / 60;
    let seconds = seconds % 60;
    if hours > 0 {
        format!("{hours}h {minutes:02}m")
    } else if minutes > 0 {
        format!("{minutes}m {seconds:02}s")
    } else {
        format!("{seconds}s")
    }
}

fn format_month(value: &str) -> String {
    let Some(date) = LocalDateTime::parse_date_prefix(value) else {
        return value.to_string();
    };
    format!("{} {}", month_name(date.month), date.year)
}

pub(super) struct Timing {
    pub(super) date: String,
    pub(super) range: String,
}

pub(super) fn workout_timing(raw: &str, duration_seconds: u64) -> Timing {
    let Some(start) = LocalDateTime::parse(raw) else {
        return Timing {
            date: if raw.is_empty() {
                "unknown date".to_string()
            } else {
                raw.to_string()
            },
            range: "time unavailable".to_string(),
        };
    };
    let end = start.add_seconds(duration_seconds);
    let range = if start.same_date(end) {
        format!("{}–{}", start.format_time(), end.format_time())
    } else {
        format!(
            "{}–{} {}",
            start.format_time(),
            end.format_date(),
            end.format_time(),
        )
    };
    Timing {
        date: start.format_date(),
        range,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LocalDateTime {
    year: i64,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
}

impl LocalDateTime {
    fn parse(raw: &str) -> Option<Self> {
        if raw.len() != 19
            || !matches!(raw.as_bytes().get(4), Some(b'-'))
            || !matches!(raw.as_bytes().get(7), Some(b'-'))
            || !matches!(raw.as_bytes().get(10), Some(b' ' | b'T'))
            || !matches!(raw.as_bytes().get(13), Some(b':'))
            || !matches!(raw.as_bytes().get(16), Some(b':'))
        {
            return None;
        }
        let value = Self {
            year: raw[0..4].parse().ok()?,
            month: raw[5..7].parse().ok()?,
            day: raw[8..10].parse().ok()?,
            hour: raw[11..13].parse().ok()?,
            minute: raw[14..16].parse().ok()?,
            second: raw[17..19].parse().ok()?,
        };
        (value.month >= 1
            && value.month <= 12
            && value.day >= 1
            && value.day <= days_in_month(value.year, value.month)
            && value.hour < 24
            && value.minute < 60
            && value.second < 60)
            .then_some(value)
    }

    fn parse_date_prefix(raw: &str) -> Option<Self> {
        (raw.len() >= 7).then_some(())?;
        let value = Self {
            year: raw[0..4].parse().ok()?,
            month: raw[5..7].parse().ok()?,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0,
        };
        (raw.as_bytes().get(4) == Some(&b'-') && (1..=12).contains(&value.month)).then_some(value)
    }

    fn add_seconds(self, seconds: u64) -> Self {
        let start_seconds =
            u64::from(self.hour) * 3_600 + u64::from(self.minute) * 60 + u64::from(self.second);
        let elapsed = start_seconds + seconds;
        let days = days_from_civil(self.year, self.month, self.day) + (elapsed / 86_400) as i64;
        let day_seconds = elapsed % 86_400;
        let (year, month, day) = civil_from_days(days);
        Self {
            year,
            month,
            day,
            hour: (day_seconds / 3_600) as u32,
            minute: (day_seconds % 3_600 / 60) as u32,
            second: (day_seconds % 60) as u32,
        }
    }

    fn same_date(self, other: Self) -> bool {
        (self.year, self.month, self.day) == (other.year, other.month, other.day)
    }

    fn format_date(self) -> String {
        format!("{} {}, {}", month_name(self.month), self.day, self.year)
    }

    fn format_time(self) -> String {
        let suffix = if self.hour < 12 { "AM" } else { "PM" };
        let hour = match self.hour % 12 {
            0 => 12,
            hour => hour,
        };
        format!("{hour}:{:02} {suffix}", self.minute)
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

fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        4 | 6 | 9 | 11 => 30,
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        _ => 31,
    }
}

// Howard Hinnant's civil-calendar conversions, with day zero at 1970-01-01.
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

pub(super) fn plural<'a>(count: usize, one: &'a str, many: &'a str) -> &'a str {
    if count == 1 { one } else { many }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_formatters_match_the_old_browser_output() {
        assert_eq!(format_integer(5_561_u64), "5,561");
        assert_eq!(format_scaled(102_500, 1_000), "102.5");
        assert_eq!(format_scaled(100_000, 1_000), "100");
        assert_eq!(format_scaled(50, 100), "0.5");
        assert_eq!(format_duration(7_534), "2h 05m");
    }

    #[test]
    fn local_workout_times_cross_days_without_inventing_a_timezone() {
        let timing = workout_timing("2024-02-29 23:45:00", 5_400);
        assert_eq!(timing.date, "Feb 29, 2024");
        assert_eq!(timing.range, "11:45 PM–Mar 1, 2024 1:15 AM");
    }
}
