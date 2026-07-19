//! The melted patch of September sea ice drawn to scale beside an everyday
//! object, on a 1 m grid with a scale bar. Shared by the receipt and the
//! charts' ice callout — both can render on the same page, so each caller
//! passes its own SVG `pattern_id` (pattern ids are document-global).

use topcoat::{
    Result,
    view::{component, view},
};

use crate::flight::format::format_ice;

/// Patches smaller than this render nothing; callers gate their surrounding
/// copy on the same floor so the row and the figure agree.
pub const ICE_SHOW_FLOOR_M2: f64 = 0.05;

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
pub async fn ice_figure(ice_m2: f64, pattern_id: &str) -> Result {
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
                        id=(pattern_id)
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
                    fill=(format!("url(#{pattern_id})"))
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
                <text
                    x=(ICE_MARGIN + 1.14)
                    y=(vb_h - 0.24)
                    font-size=(fs * 0.85)
                    fill="var(--ink-2)"
                >
                    "1 m"
                </text>
            </svg>
            <div class="ice-note">
                "september sea ice, drawn to scale — every tonne melts ≈ 3 m² — about one queen bed"
            </div>
        </div>
    }
}
