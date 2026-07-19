//! Scale-aware comparison rows: given scaleKg (full bar width in kg CO2e),
//! each domain picks a unit/count so the bar fill is readable (~0.25x-3x scale).

use std::sync::LazyLock;

use crate::flight::format::{format_js_number, round_count, round_rate_count};
use crate::flight::reference_data::{ACTIVITIES, Activity};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ComparisonMode {
    #[default]
    Absolute,
    Rate,
}

/// Stable domain fills — keep in sync with --domain-* in index.css and paris slices.
pub const DOMAIN_COLORS: [(&str, &str); 4] = [
    ("transport", "var(--domain-transport)"),
    ("food", "var(--domain-food)"),
    ("home", "var(--domain-home)"),
    ("habits", "var(--domain-habits)"),
];

pub fn domain_color(domain: &str) -> Option<&'static str> {
    DOMAIN_COLORS
        .iter()
        .find(|(id, _)| *id == domain)
        .map(|(_, color)| *color)
}

#[derive(Debug, Clone, Copy)]
pub struct DomainUnit {
    pub id: &'static str,
    /// kg CO2e per one unit
    pub kg_per_unit: f64,
    /// e.g. "vegan meals"
    pub unit_label: &'static str,
    /// row title when count is applied
    pub label_for: fn(count: f64) -> String,
    /// row title for a per-day rate (mode == Rate)
    pub label_for_rate: fn(count_per_day: f64) -> String,
    pub source_ids: &'static [&'static str],
    pub detail: &'static str,
    /// When false, skip this unit in rate mode.
    pub rate_ok: bool,
}

#[derive(Debug, Clone)]
pub struct DomainRow {
    #[allow(dead_code)] // React list key in the original; kept for data parity
    pub id: String,
    pub domain: String,
    pub label: String,
    pub detail: String,
    pub bar_fill_kg: f64,
    /// Units picked for the row label. In absolute mode this is the total count
    /// (e.g. 40 miles). In rate mode this is the per-day rate (e.g. 5 miles/day).
    pub count: f64,
    pub unit_label: String,
    pub source_ids: Vec<String>,
}

struct DomainDef {
    id: &'static str,
    /// Prefer earlier units when they fit the friendly band.
    units: Vec<DomainUnit>,
}

fn activity(id: &str) -> &'static Activity {
    ACTIVITIES
        .iter()
        .find(|a| a.id == id)
        .expect("known activity id")
}

fn activity_unit(
    id: &str,
    label_for: fn(f64) -> String,
    label_for_rate: fn(f64) -> String,
) -> DomainUnit {
    let a = activity(id);
    DomainUnit {
        id: a.id,
        kg_per_unit: a.kg_per_unit,
        unit_label: a.unit_label,
        label_for,
        label_for_rate,
        source_ids: a.source_ids,
        detail: a.basis,
        rate_ok: true,
    }
}

/// ≈1 week / ≈3 weeks — keep the ≈, pluralize the unit noun.
fn approx_units(n: f64, singular: &str, plural: &str) -> String {
    if n == 1.0 {
        format!("≈1 {singular}")
    } else {
        format!("≈{} {plural}", format_js_number(n))
    }
}

/// Formats an already-daily count as "≈5 miles" / "≈0.6 miles", pluralizing the noun.
fn format_rate_count(n: f64, singular: &str, plural: &str) -> String {
    let count = round_rate_count(n);
    let formatted = if count >= 10.0 {
        format_js_number(round_count(count))
    } else if count == count.trunc() {
        format_js_number(count)
    } else {
        format!("{count:.1}")
    };
    let noun = if count == 1.0 { singular } else { plural };
    format!("≈{formatted} {noun}")
}

/// ≈5 miles a day / ≈0.6 miles a day — always a per-day rate.
fn approx_rate(n: f64, singular: &str, plural: &str) -> String {
    format!("{} a day", format_rate_count(n, singular, plural))
}

/// Transport: concrete countable units (no average-period ladders).
fn transport_units() -> Vec<DomainUnit> {
    let gas = activity("gas-car");
    let mi = gas.kg_per_unit;
    vec![
        DomainUnit {
            id: "gas-miles",
            kg_per_unit: mi,
            unit_label: "miles in a gas car",
            label_for: |n| {
                format!(
                    "Driving {}",
                    approx_units(n, "gas-car mile", "gas-car miles")
                )
            },
            label_for_rate: |n| format!("Driving {}", approx_rate(n, "mile", "miles")),
            source_ids: gas.source_ids,
            detail: gas.basis,
            rate_ok: true,
        },
        DomainUnit {
            id: "gas-tanks",
            kg_per_unit: 12.0 * 8.9,
            unit_label: "12-gallon tanks of gasoline",
            label_for: |n| format!("Burning {}", approx_units(n, "tank of gas", "tanks of gas")),
            label_for_rate: |n| {
                format!("Burning {}", approx_rate(n, "tank of gas", "tanks of gas"))
            },
            source_ids: gas.source_ids,
            detail: "EPA: ≈8.9 kg CO₂ per gallon; 12-gallon tank",
            rate_ok: false,
        },
        DomainUnit {
            id: "coast-drives",
            kg_per_unit: 2790.0 * 0.4,
            unit_label: "New York → L.A. drives in a gas car",
            label_for: |n| format!("Driving NYC → LA {}", approx_units(n, "time", "times")),
            label_for_rate: |n| format!("Driving NYC → LA {}", approx_rate(n, "time", "times")),
            source_ids: gas.source_ids,
            detail: "≈2,790 mi × EPA ≈400 g CO₂/mi",
            rate_ok: false,
        },
    ]
}

fn food_units() -> Vec<DomainUnit> {
    let burger_kg = 3.6;
    let avg = activity("diet-average");
    let vegan = activity("diet-vegan");
    vec![
        DomainUnit {
            id: "burgers",
            kg_per_unit: burger_kg,
            unit_label: "quarter-pound hamburgers",
            label_for: |n| approx_units(n, "quarter-pound hamburger", "quarter-pound hamburgers"),
            label_for_rate: |n| {
                approx_rate(n, "quarter-pound hamburger", "quarter-pound hamburgers")
            },
            source_ids: &["meat"],
            detail: "≈3.6 kg CO2e each, US-typical beef intensity",
            rate_ok: true,
        },
        DomainUnit {
            id: "vegan-meals",
            kg_per_unit: vegan.kg_per_unit,
            unit_label: "vegan meals",
            label_for: |n| approx_units(n, "vegan meal", "vegan meals"),
            label_for_rate: |n| approx_rate(n, "vegan meal", "vegan meals"),
            source_ids: vegan.source_ids,
            detail: vegan.basis,
            rate_ok: true,
        },
        DomainUnit {
            id: "avg-meals",
            kg_per_unit: avg.kg_per_unit,
            unit_label: "average-diet meals",
            label_for: |n| approx_units(n, "average-diet meal", "average-diet meals"),
            label_for_rate: |n| approx_rate(n, "average-diet meal", "average-diet meals"),
            source_ids: avg.source_ids,
            detail: avg.basis,
            rate_ok: true,
        },
    ]
}

fn habit_units() -> Vec<DomainUnit> {
    vec![
        activity_unit(
            "chatgpt",
            |n| approx_units(n, "ChatGPT query", "ChatGPT queries"),
            |n| approx_rate(n, "ChatGPT query", "ChatGPT queries"),
        ),
        activity_unit(
            "straw",
            |n| approx_units(n, "plastic straw", "plastic straws"),
            |n| approx_rate(n, "plastic straw", "plastic straws"),
        ),
        activity_unit(
            "soda",
            |n| approx_units(n, "can of Diet Coke", "cans of Diet Coke"),
            |n| approx_rate(n, "can of Diet Coke", "cans of Diet Coke"),
        ),
        activity_unit(
            "bottle",
            |n| approx_units(n, "plastic water bottle", "plastic water bottles"),
            |n| approx_rate(n, "plastic water bottle", "plastic water bottles"),
        ),
    ]
}

fn home_units() -> Vec<DomainUnit> {
    let ac = activity("ac");
    vec![
        DomainUnit {
            id: "ac-hours",
            kg_per_unit: ac.kg_per_unit,
            unit_label: "hours of central A/C",
            label_for: |n| format!("{} of central A/C", approx_units(n, "hour", "hours")),
            label_for_rate: |n| {
                format!(
                    "{} of central A/C a day",
                    format_rate_count(n, "hour", "hours")
                )
            },
            source_ids: ac.source_ids,
            detail: ac.basis,
            rate_ok: true,
        },
        DomainUnit {
            id: "hot-showers",
            kg_per_unit: 1.7,
            unit_label: "hot showers",
            label_for: |n| approx_units(n, "hot shower", "hot showers"),
            label_for_rate: |n| approx_rate(n, "hot shower", "hot showers"),
            source_ids: ac.source_ids,
            detail: "≈1.7 kg CO₂e per shower (existing site factor)",
            rate_ok: true,
        },
        DomainUnit {
            id: "ac-800h",
            kg_per_unit: 800.0 * 1.1,
            unit_label: "blocks of ≈800 central A/C hours",
            label_for: |n| {
                if n == 1.0 {
                    "≈800 hours of central A/C".to_string()
                } else {
                    approx_units(n, "block of ≈800 A/C hours", "blocks of ≈800 A/C hours")
                }
            },
            label_for_rate: |n| {
                format!(
                    "{} of central A/C a day",
                    format_rate_count(n, "hour", "hours")
                )
            },
            source_ids: ac.source_ids,
            detail: "≈800 A/C hours × ≈1.1 kg/hour",
            rate_ok: false,
        },
    ]
}

static DOMAINS: LazyLock<Vec<DomainDef>> = LazyLock::new(|| {
    vec![
        DomainDef {
            id: "transport",
            units: transport_units(),
        },
        DomainDef {
            id: "food",
            units: food_units(),
        },
        DomainDef {
            id: "home",
            units: home_units(),
        },
        DomainDef {
            id: "habits",
            units: habit_units(),
        },
    ]
});

const MIN_RATIO: f64 = 0.25;
const MAX_RATIO: f64 = 3.5;
/// Prefer counts in this band when stepping units.
const MIN_COUNT: f64 = 0.5;
const MAX_COUNT: f64 = 8.0;

fn pick_unit<'a>(scale_kg: f64, units: &[&'a DomainUnit]) -> (&'a DomainUnit, f64, f64) {
    if scale_kg.is_nan() || scale_kg <= 0.0 {
        let unit = units[0];
        return (unit, 1.0, unit.kg_per_unit);
    }

    let mut best: Option<(&DomainUnit, f64, f64, f64)> = None;

    for &unit in units {
        if unit.kg_per_unit.is_nan() || unit.kg_per_unit <= 0.0 {
            continue;
        }
        let raw = scale_kg / unit.kg_per_unit;
        let count = round_count(raw).max(1.0);
        let fill_kg = count * unit.kg_per_unit;
        let ratio = fill_kg / scale_kg;
        let in_band = (MIN_RATIO..=MAX_RATIO).contains(&ratio);
        let count_ok = (MIN_COUNT..=MAX_COUNT * 1.5).contains(&raw);
        // Prefer in-band fills; among those, prefer friendlier raw counts.
        let score = (if in_band { 0.0 } else { 100.0 })
            + (if count_ok { 0.0 } else { 20.0 })
            + ratio.ln().abs()
            + (raw.max(0.01) / 2.0).ln().abs() * 0.1;

        if best.is_none_or(|b| score < b.3) {
            best = Some((unit, count, fill_kg, score));
        }
    }

    match best {
        Some((unit, count, fill_kg, _)) => (unit, count, fill_kg),
        None => (units[0], 1.0, units[0].kg_per_unit),
    }
}

/// Mirrors pick_unit but scores against a daily rate: counts below 1/day are
/// allowed (no max(1, ...)) and rounding uses round_rate_count.
fn pick_rate_unit<'a>(daily_kg: f64, units: &[&'a DomainUnit]) -> (&'a DomainUnit, f64, f64) {
    if daily_kg.is_nan() || daily_kg <= 0.0 {
        let unit = units[0];
        return (unit, 0.0, 0.0);
    }

    let mut best: Option<(&DomainUnit, f64, f64, f64)> = None;

    for &unit in units {
        if unit.kg_per_unit.is_nan() || unit.kg_per_unit <= 0.0 {
            continue;
        }
        let raw = daily_kg / unit.kg_per_unit;
        let count_per_day = round_rate_count(raw);
        let daily_fill_kg = count_per_day * unit.kg_per_unit;
        let ratio = daily_fill_kg / daily_kg;
        let in_band = (MIN_RATIO..=MAX_RATIO).contains(&ratio);
        let count_ok = (MIN_COUNT..=MAX_COUNT * 1.5).contains(&raw);
        let score = (if in_band { 0.0 } else { 100.0 })
            + (if count_ok { 0.0 } else { 20.0 })
            + ratio.ln().abs()
            + (raw.max(0.01) / 2.0).ln().abs() * 0.1;

        if best.is_none_or(|b| score < b.3) {
            best = Some((unit, count_per_day, daily_fill_kg, score));
        }
    }

    match best {
        Some((unit, count_per_day, daily_fill_kg, _)) => (unit, count_per_day, daily_fill_kg),
        None => (units[0], 0.0, 0.0),
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PickDomainRowsOptions<'a> {
    /// Unit ids to skip (e.g. hide burgers when the subject is already hamburgers).
    pub exclude_unit_ids: &'a [&'a str],
    /// Absolute (default) picks a one-off count; Rate picks a per-day rate.
    pub mode: ComparisonMode,
    /// Days in the bar period; used to convert scale_kg to a daily rate when mode == Rate.
    pub period_days: Option<f64>,
}

pub fn pick_domain_rows(scale_kg: f64, options: &PickDomainRowsOptions) -> Vec<DomainRow> {
    let excluded = options.exclude_unit_ids;
    let mode = options.mode;
    let mut rows: Vec<DomainRow> = Vec::new();

    for domain in DOMAINS.iter() {
        let units: Vec<&DomainUnit> = domain
            .units
            .iter()
            .filter(|u| !excluded.contains(&u.id))
            .collect();
        if units.is_empty() {
            continue;
        }

        if mode == ComparisonMode::Rate {
            let raw_period_days = options.period_days.unwrap_or(1.0);
            let period_days = if raw_period_days > 0.0 {
                raw_period_days
            } else {
                1.0
            };
            let daily_kg = scale_kg / period_days;
            let rate_units: Vec<&DomainUnit> =
                units.iter().copied().filter(|u| u.rate_ok).collect();
            if rate_units.is_empty() {
                continue;
            }
            let (unit, count_per_day, daily_fill_kg) = pick_rate_unit(daily_kg, &rate_units);
            rows.push(DomainRow {
                id: format!("{}:{}", domain.id, unit.id),
                domain: domain.id.to_string(),
                label: (unit.label_for_rate)(count_per_day),
                detail: unit.detail.to_string(),
                bar_fill_kg: daily_fill_kg * period_days,
                count: count_per_day,
                unit_label: unit.unit_label.to_string(),
                source_ids: unit.source_ids.iter().map(|s| s.to_string()).collect(),
            });
            continue;
        }

        let (unit, count, fill_kg) = pick_unit(scale_kg, &units);
        rows.push(DomainRow {
            id: format!("{}:{}", domain.id, unit.id),
            domain: domain.id.to_string(),
            label: (unit.label_for)(count),
            detail: unit.detail.to_string(),
            bar_fill_kg: fill_kg,
            count,
            unit_label: unit.unit_label.to_string(),
            source_ids: unit.source_ids.iter().map(|s| s.to_string()).collect(),
        });
    }

    rows
}

pub fn comparison_rows(scale_kg: f64, mode: ComparisonMode) -> Vec<DomainRow> {
    pick_domain_rows(
        scale_kg,
        &PickDomainRowsOptions {
            mode,
            ..Default::default()
        },
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MakeupKind {
    Abstain,
    Switch,
}

#[derive(Debug, Clone)]
pub struct MakeupChipDef {
    pub id: &'static str,
    pub domain: &'static str,
    #[allow(dead_code)] // unused at runtime in the original too; kept for data parity
    pub label: &'static str,
    pub kind: MakeupKind,
    pub dirty_kg_per_unit: f64,
    pub clean_kg_per_unit: Option<f64>,
    pub unit_label: &'static str,
    pub label_for: fn(count: f64) -> String,
    pub label_for_rate: fn(count_per_day: f64) -> String,
    pub detail: String,
    pub source_ids: Vec<String>,
}

pub fn list_makeup_chips() -> Vec<MakeupChipDef> {
    let gas = activity("gas-car");
    let ev = activity("ev");
    let avg = activity("diet-average");
    let ac = activity("ac");
    let soda = activity("soda");
    let vegan = activity("diet-vegan");

    let chips: Vec<MakeupChipDef> = vec![
        MakeupChipDef {
            id: "drive-less",
            domain: "transport",
            label: "Drive less",
            kind: MakeupKind::Abstain,
            dirty_kg_per_unit: gas.kg_per_unit,
            clean_kg_per_unit: None,
            unit_label: "gas-car miles skipped",
            label_for: |n| {
                format!(
                    "Skip driving {}",
                    approx_units(n, "gas-car mile", "gas-car miles")
                )
            },
            label_for_rate: |n| format!("Skip driving {}", approx_rate(n, "mile", "miles")),
            detail: gas.basis.to_string(),
            source_ids: gas.source_ids.iter().map(|s| s.to_string()).collect(),
        },
        MakeupChipDef {
            id: "go-ev",
            domain: "transport",
            label: "Go EV",
            kind: MakeupKind::Switch,
            dirty_kg_per_unit: gas.kg_per_unit,
            clean_kg_per_unit: Some(ev.kg_per_unit),
            unit_label: "miles switched to an EV",
            label_for: |n| format!("Switch {} to an EV", approx_units(n, "mile", "miles")),
            label_for_rate: |n| format!("Switch {} to an EV", approx_rate(n, "mile", "miles")),
            detail: format!("{}; EV {}", gas.basis, ev.basis),
            source_ids: gas
                .source_ids
                .iter()
                .chain(ev.source_ids.iter())
                .map(|s| s.to_string())
                .collect(),
        },
        MakeupChipDef {
            id: "eat-plants",
            domain: "food",
            label: "Eat plants",
            kind: MakeupKind::Switch,
            dirty_kg_per_unit: avg.kg_per_unit,
            clean_kg_per_unit: Some(vegan.kg_per_unit),
            unit_label: "meals swapped to vegan",
            label_for: |n| format!("Swap {} to vegan", approx_units(n, "meal", "meals")),
            label_for_rate: |n| format!("Swap {} to vegan", approx_rate(n, "meal", "meals")),
            detail: "average-diet meal → vegan meal (Scarborough)".to_string(),
            source_ids: avg.source_ids.iter().map(|s| s.to_string()).collect(),
        },
        MakeupChipDef {
            id: "sweat-it-out",
            domain: "home",
            label: "Sweat it out",
            kind: MakeupKind::Abstain,
            dirty_kg_per_unit: ac.kg_per_unit,
            clean_kg_per_unit: None,
            unit_label: "A/C hours skipped",
            label_for: |n| format!("Skip {} of central A/C", approx_units(n, "hour", "hours")),
            label_for_rate: |n| {
                format!(
                    "Skip {} of central A/C a day",
                    format_rate_count(n, "hour", "hours")
                )
            },
            detail: ac.basis.to_string(),
            source_ids: ac.source_ids.iter().map(|s| s.to_string()).collect(),
        },
        MakeupChipDef {
            id: "skip-soda",
            domain: "habits",
            label: "Skip the soda",
            kind: MakeupKind::Abstain,
            dirty_kg_per_unit: soda.kg_per_unit,
            clean_kg_per_unit: None,
            unit_label: "Diet Cokes skipped",
            label_for: |n| {
                format!(
                    "Skip {}",
                    approx_units(n, "can of Diet Coke", "cans of Diet Coke")
                )
            },
            label_for_rate: |n| {
                format!(
                    "Skip {}",
                    approx_rate(n, "can of Diet Coke", "cans of Diet Coke")
                )
            },
            detail: soda.basis.to_string(),
            source_ids: soda.source_ids.iter().map(|s| s.to_string()).collect(),
        },
    ];

    chips
        .into_iter()
        .filter(|c| {
            c.kind == MakeupKind::Abstain
                || c.clean_kg_per_unit
                    .is_some_and(|clean| c.dirty_kg_per_unit > clean)
        })
        .collect()
}

pub fn pick_makeup_row(
    scale_kg: f64,
    chip: &MakeupChipDef,
    mode: ComparisonMode,
    period_days: Option<f64>,
) -> DomainRow {
    let delta = match chip.kind {
        MakeupKind::Switch => chip.dirty_kg_per_unit - chip.clean_kg_per_unit.unwrap_or(0.0),
        MakeupKind::Abstain => chip.dirty_kg_per_unit,
    };
    if delta.is_nan() || delta <= 0.0 || scale_kg.is_nan() || scale_kg <= 0.0 {
        return DomainRow {
            id: format!("{}:makeup:{}", chip.domain, chip.id),
            domain: chip.domain.to_string(),
            label: (chip.label_for)(0.0),
            detail: chip.detail.clone(),
            bar_fill_kg: 0.0,
            count: 0.0,
            unit_label: chip.unit_label.to_string(),
            source_ids: chip.source_ids.clone(),
        };
    }

    if mode == ComparisonMode::Rate {
        let raw_period_days = period_days.unwrap_or(1.0);
        let period_days = if raw_period_days > 0.0 {
            raw_period_days
        } else {
            1.0
        };
        let daily_kg = scale_kg / period_days;
        let count_per_day = round_rate_count(daily_kg / delta);
        let daily_fill = count_per_day * delta;
        return DomainRow {
            id: format!("{}:makeup:{}", chip.domain, chip.id),
            domain: chip.domain.to_string(),
            label: (chip.label_for_rate)(count_per_day),
            detail: chip.detail.clone(),
            bar_fill_kg: daily_fill * period_days,
            count: count_per_day,
            unit_label: chip.unit_label.to_string(),
            source_ids: chip.source_ids.clone(),
        };
    }

    let count = round_count(scale_kg / delta).max(1.0);
    let fill_kg = count * delta;
    DomainRow {
        id: format!("{}:makeup:{}", chip.domain, chip.id),
        domain: chip.domain.to_string(),
        label: (chip.label_for)(count),
        detail: chip.detail.clone(),
        bar_fill_kg: fill_kg,
        count,
        unit_label: chip.unit_label.to_string(),
        source_ids: chip.source_ids.clone(),
    }
}
