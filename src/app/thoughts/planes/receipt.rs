//! The flight climate receipt: printer chrome, itemized CO₂e bill, sea-ice
//! figure, QR share code and the comparison coupon. Ported from
//! `~/how-bad/src/components/{Receipt,IceGraphic,ReceiptComparisons}.tsx`.
//! The original's copy-for-friend / copy-as-image buttons are dropped per the
//! design doc; the shareable URL + QR carry that job.

use topcoat::{
    Result,
    context::Cx,
    router::{header, headers},
    view::{Unescaped, View, component, view},
};

use crate::flight::{
    airports::Airport,
    comparison_scale::{PickDomainRowsOptions, pick_domain_rows},
    emissions::{Cabin, FlightImpact, JET_FUEL_KG_PER_LITRE},
    format::{
        format_bar_value, format_ice, format_km, format_litres, format_tonnes, format_tonnes_smart,
        format_whole, format_years,
    },
    sources::cite,
};

/// Match IceGraphic's own floor so the row and the figure agree.
const ICE_SHOW_FLOOR_M2: f64 = 0.05;

const DOT_LEADER: &str =
    "................................................................................";

fn sky_note(factor: f64) -> String {
    let f = format!("×{factor:.1}");
    let skies = if factor >= 1.25 {
        format!(
            "heat-trapping cloud trails, priced to contrail-prone skies ({f} the global average, per km flown)"
        )
    } else if factor <= 0.75 {
        format!(
            "heat-trapping cloud trails, priced to contrail-lean skies ({f} the global average, per km flown)"
        )
    } else {
        format!("heat-trapping cloud trails, priced to about-average skies ({f})")
    };
    format!(
        "{skies}. Weather decides the day — ≈2.7% of flights cause 80% of contrail warming; science’s range on this line runs ⅓× to 1.7×"
    )
}

/// The share URL as a print-style QR SVG (margin 0; sized by CSS like the
/// original's `qrcode` npm output). Trusted markup: we generate it.
fn qr_svg(url: &str) -> String {
    let Ok(code) = qrcode::QrCode::new(url.as_bytes()) else {
        return String::new();
    };
    let svg = code
        .render::<qrcode::render::svg::Color>()
        .quiet_zone(false)
        .build();
    // Strip the XML prolog; this is inline HTML, not a standalone document.
    match svg.find("?>") {
        Some(i) => svg[i + 2..].to_string(),
        None => svg,
    }
}

#[component]
async fn row(
    label: &str,
    footnotes: Vec<&'static str>,
    value: View,
    hero: bool,
    fine: bool,
) -> Result {
    view! {
        <div class=(if hero {
            "receipt-row hero"
        } else if fine {
            "receipt-row fine"
        } else {
            "receipt-row"
        })>
            <span class="r-label">
                (label)
                " "
                for id in footnotes {
                    cite(id: id)
                }
            </span>
            <span class="r-dots" aria-hidden="true">(DOT_LEADER)</span>
            <span class="r-value">(value)</span>
        </div>
    }
}

struct IceRef {
    id: &'static str,
    w: f64,
    h: f64,
    area: f64,
    label: &'static str,
}

const ICE_MARGIN: f64 = 0.5;
const ICE_GAP: f64 = 0.7;
const ICE_LABEL_GAP: f64 = 0.62;

const MAT: IceRef = IceRef {
    id: "mat",
    w: 1.73,
    h: 0.61,
    area: 1.0,
    label: "yoga mat · 1 m²",
};
const BED: IceRef = IceRef {
    id: "bed",
    w: 2.03,
    h: 1.52,
    area: 3.0,
    label: "queen bed · 3 m²",
};
const SPOT: IceRef = IceRef {
    id: "spot",
    w: 4.8,
    h: 2.4,
    area: 12.0,
    label: "parking spot · 12 m²",
};
const ICE_REFERENCES: [IceRef; 3] = [MAT, BED, SPOT];

#[component]
async fn ice_graphic(ice_m2: f64) -> Result {
    if ice_m2 < ICE_SHOW_FLOOR_M2 {
        return view! { "" };
    }

    // The one everyday object nearest the ice patch in area — nearest as a
    // ratio, not a difference, since this is a comparison of scale.
    let obj = ICE_REFERENCES
        .iter()
        .reduce(|best, r| {
            if (ice_m2 / r.area).ln().abs() < (ice_m2 / best.area).ln().abs() {
                r
            } else {
                best
            }
        })
        .expect("at least one reference object");

    let side = ice_m2.sqrt();
    let obj_x = ICE_MARGIN + side + ICE_GAP;
    let obj_y = ICE_MARGIN + ((side - obj.h) / 2.0).max(0.0);

    let plot_h = side.max(obj_y - ICE_MARGIN + obj.h + ICE_LABEL_GAP);
    let vb_w = obj_x + obj.w + ICE_MARGIN;
    let vb_h = ICE_MARGIN + plot_h + 0.9;
    let fs = (vb_w * 0.031).max(0.26);
    let label_inside = side > fs * 4.2;

    let grid_x: Vec<i64> = (0..=vb_w.ceil() as i64).collect();
    let grid_y: Vec<i64> = (0..=vb_h.ceil() as i64).collect();
    let obj_short = obj.label.split(" ·").next().unwrap_or(obj.label);

    view! {
        <div class="ice-figure">
            <svg
                viewBox=(format!("0 0 {vb_w} {vb_h}"))
                role="img"
                aria-label=(format!(
                    "{} of Arctic sea ice drawn to scale beside a {}",
                    format_ice(ice_m2),
                    obj_short
                ))
            >
                <defs>
                    <pattern
                        id="ice-dots-pat"
                        patternUnits="userSpaceOnUse"
                        width="0.24"
                        height="0.24"
                    >
                        <circle cx="0.12" cy="0.12" r="0.055" fill="var(--save-soft)" />
                    </pattern>
                </defs>
                <g stroke="var(--hairline)" stroke-width="0.012">
                    for x in grid_x {
                        <line x1=(x) y1="0" x2=(x) y2=(vb_h) />
                    }
                    for y in grid_y {
                        <line x1="0" y1=(y) x2=(vb_w) y2=(y) />
                    }
                </g>

                <rect
                    class="ice-dots"
                    x=(ICE_MARGIN)
                    y=(ICE_MARGIN)
                    width=(side)
                    height=(side)
                    rx="0.06"
                    fill="url(#ice-dots-pat)"
                />
                <rect
                    x=(ICE_MARGIN)
                    y=(ICE_MARGIN)
                    width=(side)
                    height=(side)
                    rx="0.06"
                    fill="none"
                    stroke="var(--save)"
                    stroke-width="0.04"
                    stroke-dasharray="0.16 0.11"
                />
                <text
                    x=(ICE_MARGIN + side / 2.0)
                    y=(if label_inside {
                        ICE_MARGIN + side / 2.0 + fs * 0.36
                    } else {
                        ICE_MARGIN - 0.14
                    })
                    text-anchor="middle"
                    font-size=(fs)
                    font-weight="700"
                    fill="var(--ink)"
                    stroke="var(--card)"
                    stroke-width=(fs * 0.34)
                    paint-order="stroke"
                >
                    (format_ice(ice_m2))
                </text>

                <g>
                    <rect
                        x=(obj_x)
                        y=(obj_y)
                        width=(obj.w)
                        height=(obj.h)
                        rx="0.1"
                        fill="none"
                        stroke="var(--muted)"
                        stroke-width="0.045"
                    />
                    <text
                        x=(obj_x + obj.w / 2.0)
                        y=(obj_y + obj.h + fs * 1.1)
                        text-anchor="middle"
                        font-size=(fs * 0.85)
                        fill="var(--muted)"
                    >
                        (obj.label)
                    </text>
                </g>
                if obj.id == "bed" {
                    <line
                        x1=(obj_x + BED.w * 0.12)
                        y1=(obj_y + 0.42)
                        x2=(obj_x + BED.w * 0.88)
                        y2=(obj_y + 0.42)
                        stroke="var(--muted)"
                        stroke-width="0.03"
                    />
                }
                if obj.id == "spot" {
                    <text
                        x=(obj_x + SPOT.w / 2.0)
                        y=(obj_y + SPOT.h / 2.0 + fs * 0.4)
                        text-anchor="middle"
                        font-size=(fs * 1.15)
                        font-weight="700"
                        fill="var(--hairline)"
                    >
                        "P"
                    </text>
                }

                <g stroke="var(--ink-2)" stroke-width="0.028">
                    <line x1=(ICE_MARGIN) y1=(vb_h - 0.3) x2=(ICE_MARGIN + 1.0) y2=(vb_h - 0.3) />
                    <line x1=(ICE_MARGIN) y1=(vb_h - 0.39) x2=(ICE_MARGIN) y2=(vb_h - 0.21) />
                    <line
                        x1=(ICE_MARGIN + 1.0)
                        y1=(vb_h - 0.39)
                        x2=(ICE_MARGIN + 1.0)
                        y2=(vb_h - 0.21)
                    />
                </g>
                <text x=(ICE_MARGIN + 1.14) y=(vb_h - 0.24) font-size=(fs * 0.85) fill="var(--ink-2)">
                    "1 m"
                </text>
            </svg>
            <div class="ice-note">
                "september sea ice, drawn to scale — every tonne melts ≈ 3 m² — about one queen bed"
            </div>
        </div>
    }
}

/// CVS-coupon zone printed at the foot of the receipt: the same
/// full-richness comparison rows as `ComparisonScale`, so a printed
/// receipt still carries the "compared to everything else" payoff.
#[component]
async fn receipt_comparisons(scale_kg: f64, compare_href: String) -> Result {
    let scale = if scale_kg > 0.0 { scale_kg } else { 1.0 };
    let rows = pick_domain_rows(scale, &PickDomainRowsOptions::default());

    view! {
        <div class="receipt-coupon">
            <div class="receipt-coupon-head">"★ SAVE THESE COMPARISONS ★"</div>
            <div class="receipt-items">
                for r in rows {
                    <div class="receipt-row fine">
                        <span class="r-label">(r.label)</span>
                        <span class="r-dots" aria-hidden="true">(DOT_LEADER)</span>
                        <span class="r-value">(format_bar_value(r.bar_fill_kg))</span>
                    </div>
                }
            </div>
            <p class="receipt-coupon-foot">
                <a href=(compare_href)>"see interactive scale →"</a>
            </p>
        </div>
    }
}

#[component]
pub async fn receipt(
    cx: &Cx,
    from: Airport,
    to: Airport,
    cabin: Cabin,
    round_trip: bool,
    impact: FlightImpact,
    share_path: String,
) -> Result {
    let staycation = impact.distance_km == 0.0;
    let legs_km = impact.distance_km * if round_trip { 2.0 } else { 1.0 };
    let seat_fraction = if staycation {
        1.0
    } else {
        (1.0 / impact.seat_share_of_aircraft).round()
    };
    let arrow = if round_trip { "⇄" } else { "→" };
    let show_ice = impact.ice_m2 >= ICE_SHOW_FLOOR_M2;

    // The QR encodes an absolute URL when the request names a host (the
    // public origin is whatever the visitor is browsing), the bare share
    // path otherwise.
    let hdrs = headers(cx);
    let qr_url = match hdrs.get(header::HOST).and_then(|h| h.to_str().ok()) {
        Some(host) => {
            let scheme = hdrs
                .get("x-forwarded-proto")
                .and_then(|v| v.to_str().ok())
                .unwrap_or(
                    if host.starts_with("localhost") || host.starts_with("127.") {
                        "http"
                    } else {
                        "https"
                    },
                );
            format!("{scheme}://{host}{share_path}")
        }
        None => share_path.clone(),
    };
    let qr = qr_svg(&qr_url);

    // The interactive scale is this page's Compare view.
    let compare_href = format!(
        "{share_path}{}view=compare",
        if share_path.contains('?') { "&" } else { "?" }
    );

    let route_line = format!(
        "{} ({}) {arrow} {} ({})",
        from.city, from.iata, to.city, to.iata
    );
    let route_meta = if staycation {
        "you’re already there — 0 km flown".to_string()
    } else {
        format!(
            "1 ticket · {} · {} · {} flown",
            cabin.as_str(),
            if round_trip { "round trip" } else { "one way" },
            format_km(legs_km)
        )
    };
    let code_text = format!(
        "{} {arrow} {} · {}",
        from.iata,
        to.iata,
        cabin.as_str().to_uppercase()
    );

    let whole_flight_value = view! { (format_tonnes(impact.aircraft_tonnes_co2e)) }?;
    let hero_value = view! { (format_tonnes(impact.tonnes_co2e)) }?;
    let fuel_value = view! {
        <span class="r-value-sub">
            (format!("{} ·", format_litres(impact.fuel_kg / JET_FUEL_KG_PER_LITRE)))
        </span>
        " "
        (format_tonnes_smart(impact.co2_tonnes))
    }?;
    let contrail_value = view! { (format_tonnes_smart(impact.contrail_tonnes)) }?;
    let nox_value = view! { (format_tonnes_smart(impact.nox_other_tonnes)) }?;
    let wtt_value = view! { (format_tonnes_smart(impact.wtt_tonnes)) }?;
    let ice_value = view! { (format_ice(impact.ice_m2)) }?;

    view! {
        <section class="printer" aria-label="Flight climate receipt">
            <div class="printer-slot">
                <span class="printer-led" aria-hidden="true"></span>
            </div>
            <div class="print-window">
                <div class="print-paper">
                    <div class="print-feed">
                        <div class="receipt">
                            <div class="receipt-head">
                                <div class="receipt-title">"Flight Receipt"</div>
                                <div class="receipt-tag">"per ticket · one seat's share of the flight"</div>
                            </div>
                            <div class="receipt-route">
                                <div class="route-line">(route_line)</div>
                                <div class="route-meta">(route_meta)</div>
                            </div>
                            <div class="receipt-items">
                                if !staycation {
                                    row(
                                        label: "Whole flight (aircraft)",
                                        footnotes: vec!["myclimate", "lee2021"],
                                        value: whole_flight_value,
                                        hero: false,
                                        fine: true,
                                    )
                                    <div class="receipt-allocation">
                                        (format!(
                                            "Your {} ticket ≈ 1/{} of the aircraft — the plane's full impact, split by seat\u{a0}",
                                            cabin.as_str(),
                                            format_whole(seat_fraction)
                                        ))
                                        cite(id: "myclimate")
                                    </div>
                                }
                                row(
                                    label: if staycation { "CO₂ equivalent" } else { "CO₂ equivalent · your ticket" },
                                    footnotes: vec!["myclimate", "lee2021"],
                                    value: hero_value,
                                    hero: true,
                                    fine: false,
                                )
                                if !staycation {
                                    <div class="co2-note">
                                        "itemized below — CO₂ plus the flight's other warming effects, each converted to the CO₂ that would warm the planet the same over 100 years (that's the “e”; a 20-year clock would run the altitude lines ≈4× higher) "
                                        cite(id: "defra2025")
                                    </div>
                                    row(
                                        label: "Jet fuel burned",
                                        footnotes: vec!["myclimate", "jetfuel-density"],
                                        value: fuel_value,
                                        hero: false,
                                        fine: true,
                                    )
                                    row(
                                        label: "Contrail cirrus, expected",
                                        footnotes: vec!["lee2021", "teoh2024"],
                                        value: contrail_value,
                                        hero: false,
                                        fine: true,
                                    )
                                    <div class="co2-note">(sky_note(impact.sky_factor))</div>
                                    row(
                                        label: "NOx & other altitude effects",
                                        footnotes: vec!["lee2021"],
                                        value: nox_value,
                                        hero: false,
                                        fine: true,
                                    )
                                    row(
                                        label: "Making the fuel",
                                        footnotes: vec!["myclimate"],
                                        value: wtt_value,
                                        hero: false,
                                        fine: true,
                                    )
                                }
                                if show_ice {
                                    row(
                                        label: "Arctic sea ice melted",
                                        footnotes: vec!["seaice"],
                                        value: ice_value,
                                        hero: false,
                                        fine: false,
                                    )
                                    if !staycation {
                                        <div class="co2-note">
                                            "3 m² per tonne of CO₂, the jet-fuel line only — decades of satellite ice maps plotted against humanity's cumulative CO₂ fall on a straight line"
                                        </div>
                                    }
                                    ice_graphic(ice_m2: impact.ice_m2)
                                }
                            </div>
                            <div class="receipt-total">
                                <div class="receipt-row">
                                    <span class="r-label">
                                        "Travel allowance used "
                                        cite(id: "budgets")
                                    </span>
                                    <span class="r-dots" aria-hidden="true">(DOT_LEADER)</span>
                                    <span class="r-value">(format_years(impact.travel_budget_years))</span>
                                </div>
                                <span class="note">
                                    "years of the ≈0.43 t/yr the 2030 target leaves one person for "
                                    <em>"all"</em>
                                    " travel on a 1.5 °C path — "
                                    <a href="#allowance">"explained below ↓"</a>
                                </span>
                            </div>
                            <div class="receipt-code">
                                <div class="qr" aria-hidden="true">
                                    (Unescaped::new_unchecked(qr))
                                </div>
                                <div class="code-text">(code_text)</div>
                            </div>
                            if staycation {
                                <div class="receipt-foot">"*** NO CHARGE — ENJOY THE STAYCATION ***"</div>
                            } else {
                                receipt_comparisons(
                                    scale_kg: impact.tonnes_co2e * 1000.0,
                                    compare_href: compare_href,
                                )
                            }
                        </div>
                    </div>
                </div>
            </div>
        </section>
    }
}
