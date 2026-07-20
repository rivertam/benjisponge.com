//! The source cards and citation superscripts for the planes post.
//!
//! Ported from `~/how-bad` `src/lib/sources.ts` (the cards) and
//! `src/components/{Cite,SourcesProvider}.tsx` (how cites render and the
//! sources copy). The original opened the cards in a slide-over drawer;
//! here every cite opens its source card in place as a native popover
//! (declarative `popovertarget`, so still no JavaScript), and the footer
//! keeps just the credit and the methodology notes.

use std::sync::LazyLock;
use std::sync::atomic::{AtomicUsize, Ordering};

use topcoat::{
    Result,
    view::{component, view},
};

use crate::components::{ext_link, inline_popover};

pub struct Source {
    pub id: &'static str,
    /// The citation number shown in superscripts and on the source cards:
    /// 1-based position in [`SOURCES`], which is ordered by first appearance
    /// on the page.
    pub num: usize,
    pub kicker: &'static str,
    pub title: &'static str,
    pub url: &'static str,
}

struct Def {
    id: &'static str,
    kicker: &'static str,
    title: &'static str,
    url: &'static str,
}

/// Ordered by first appearance on the page; the array index + 1 is the
/// citation number shown in superscripts and on the source cards.
static SOURCE_DEFS: &[Def] = &[
    Def {
        id: "myclimate",
        kicker: "Fuel model",
        title: "myclimate flight calculator methodology (2015 document, in force through 2018, archived): fuel per passenger, CO₂ and well-to-tank rates",
        url: "https://web.archive.org/web/20181121123919/https://www.myclimate.org/fileadmin/user_upload/myclimate_-_home/01_Information/01_About_myclimate/09_Calculation_principles/Documents/myclimate-flight-calculator-documentation_EN.pdf",
    },
    Def {
        id: "lee2021",
        kicker: "Altitude effects",
        title: "Lee et al. (2021): aviation’s non-CO₂ forcing — contrail cirrus ≈0.63× and NOx & aerosols ≈0.11× the CO₂ at GWP100 (Table 5)",
        url: "https://www.sciencedirect.com/science/article/pii/S1352231020305689",
    },
    Def {
        id: "defra2025",
        kicker: "The 100-year clock",
        title: "UK Government GHG Conversion Factors 2025 (methodology, §8.43–8.45): the ×1.7 altitude uplift on aviation CO₂, GWP100, after Lee et al.",
        url: "https://assets.publishing.service.gov.uk/media/6846b0870392ed9b784c0187/2025-GHG-CF-methodology-paper.pdf",
    },
    Def {
        id: "teoh2024",
        kicker: "Contrail geography",
        title: "Teoh et al. (2024): contrail forcing per km flown, by region (Table 2) — and the ≈2.7% of flights behind 80% of it",
        url: "https://acp.copernicus.org/articles/24/6071/2024/",
    },
    Def {
        id: "seaice",
        kicker: "Arctic sea ice",
        title: "Notz & Stroeve (2016): ~3 m² of September sea ice lost per tonne of CO₂",
        url: "https://www.science.org/doi/10.1126/science.aag2345",
    },
    Def {
        id: "jetfuel-density",
        kicker: "Fuel density",
        title: "Measurement Canada, Volume Correction Factors — Jet A/A-1 (Table 54B): standard density 800 kg/m³ at 15 °C",
        url: "https://www.ic.gc.ca/eic/site/mc-mc.nsf/vwapj/VCF_JetA.pdf/$file/VCF_JetA.pdf",
    },
    Def {
        id: "cars",
        kicker: "Gas car footprints",
        title: "US EPA: greenhouse gas emissions from a typical passenger vehicle",
        url: "https://www.epa.gov/greenvehicles/greenhouse-gas-emissions-typical-passenger-vehicle",
    },
    Def {
        id: "compact-mpg",
        kicker: "Compact-car MPG",
        title: "US DOE/EPA fueleconomy.gov: 2025 Toyota Corolla — 35 mpg EPA combined",
        url: "https://www.fueleconomy.gov/feg/bymodel/2025_Toyota_Corolla.shtml",
    },
    Def {
        id: "sports-mpg",
        kicker: "Sports-car MPG",
        title: "US DOE/EPA fueleconomy.gov: 2025 Porsche 911 Carrera — 21 mpg EPA combined (Corvette 19, Mustang GT 19)",
        url: "https://www.fueleconomy.gov/feg/bymodel/2025_Porsche_911.shtml",
    },
    Def {
        id: "hummer",
        kicker: "The Hummer",
        title: "Cars.com: Hummer H2 specifications — the 32-gallon fuel tank",
        url: "https://www.cars.com/research/hummer-h2-2003/specs/",
    },
    Def {
        id: "budgets",
        kicker: "1.5 °C, per person",
        title: "1.5-Degree Lifestyles technical report (IGES / Aalto / D-mat, 2019): per-person footprint targets; the mobility split is Annex D, Table D.1",
        url: "https://www.iges.or.jp/en/publication_documents/pub/technicalreport/en/6719/15_Degree_Lifestyles_MainReport.pdf",
    },
    Def {
        id: "paris",
        kicker: "The treaty",
        title: "UNFCCC: the Paris Agreement — “well below 2 °C,” pursuing 1.5 °C",
        url: "https://unfccc.int/process-and-meetings/the-paris-agreement",
    },
    Def {
        id: "parisview",
        kicker: "What scientists expect",
        title: "The Guardian (2024): survey of IPCC authors — only 6% expect the 1.5 °C limit to hold",
        url: "https://www.theguardian.com/environment/article/2024/may/08/world-scientists-climate-failure-survey-global-temperature",
    },
    Def {
        id: "consensus",
        kicker: "The consensus",
        title: "NASA: 97%+ of publishing climate scientists agree humans are warming the planet",
        url: "https://science.nasa.gov/climate-change/scientific-consensus/",
    },
    Def {
        id: "ev",
        kicker: "EV footprints",
        title: "US DOE (AFDC): emissions from electric vehicles on the U.S. grid",
        url: "https://afdc.energy.gov/vehicles/electric-emissions",
    },
    Def {
        id: "diets",
        kicker: "Diet footprints",
        title: "Scarborough et al. (2014): dietary emissions of meat-eaters, vegetarians and vegans",
        url: "https://link.springer.com/article/10.1007/s10584-014-1169-1",
    },
    Def {
        id: "meat",
        kicker: "Meat footprints",
        title: "Poore & Nemecek (2018), Science: per-food footprints — beef, poultry and the rest",
        url: "https://www.science.org/doi/10.1126/science.aaq0216",
    },
    Def {
        id: "heller",
        kicker: "US burger LCA",
        title: "Heller & Keoleian (2018), University of Michigan CSS18-10: US quarter-pound beef patty ~3.7 kg CO2e cradle-to-distribution (Thoma et al. beef)",
        url: "https://css.umich.edu/sites/default/files/publication/CSS18-10.pdf",
    },
    Def {
        id: "heating",
        kicker: "Home heating",
        title: "US EIA: Residential Energy Consumption Survey — household natural gas use",
        url: "https://www.eia.gov/consumption/residential/",
    },
    Def {
        id: "heatpump",
        kicker: "The electric swap",
        title: "US DOE: heat pump systems — several times more efficient than combustion heat",
        url: "https://www.energy.gov/energysaver/heat-pump-systems",
    },
    Def {
        id: "fashion",
        kicker: "Fast fashion",
        title: "UNEP: putting the brakes on fast fashion — the industry’s climate share",
        url: "https://www.unep.org/news-and-stories/story/putting-brakes-fast-fashion",
    },
    Def {
        id: "jeans",
        kicker: "One pair of jeans",
        title: "Levi Strauss & Co. (2015): lifecycle assessment of the 501 — ≈33 kg CO₂e",
        url: "https://www.levistrauss.com/wp-content/uploads/2015/03/Full-LCA-Results-Deck-FINAL.pdf",
    },
    Def {
        id: "pirg",
        kicker: "The 53-garment year",
        title: "US PIRG (2024): the average American buys ≈53 new items of clothing a year",
        url: "https://pirg.org/articles/how-many-clothes-are-too-many/",
    },
    Def {
        id: "foodwaste",
        kicker: "Food waste",
        title: "FAO: Food Wastage Footprint — the climate cost of uneaten food",
        url: "https://www.fao.org/4/i3347e/i3347e.pdf",
    },
    Def {
        id: "compost",
        kicker: "Composting",
        title: "US EPA (2023): quantifying methane emissions from landfilled food waste",
        url: "https://www.epa.gov/land-research/quantifying-methane-emissions-landfilled-food-waste",
    },
    Def {
        id: "grid",
        kicker: "Grid intensity",
        title: "US EIA: CO₂ emitted per kilowatt-hour of U.S. electricity",
        url: "https://www.eia.gov/tools/faqs/faq.php?id=74&t=11",
    },
    Def {
        id: "items",
        kicker: "Everyday item footprints",
        title: "Berners-Lee, “How Bad Are Bananas? The Carbon Footprint of Everything” (rev. ed. 2020, Profile Books) — bottles, lattes, showers",
        url: "https://profilebooks.com/work/how-bad-are-bananas/",
    },
    Def {
        id: "straw-lca",
        kicker: "Plastic straws",
        title: "Lifecycle studies of disposable straws: ≈1.5 g CO₂e per polypropylene straw",
        url: "https://pmc.ncbi.nlm.nih.gov/articles/PMC8897272/",
    },
    Def {
        id: "soda",
        kicker: "Soda footprints",
        title: "Carbon Trust × Coca-Cola LCA (2009): a 330 ml can of Diet Coke ≈ 150 g CO₂e",
        url: "https://trellis.net/article/canned-diet-coke-offers-smaller-carbon-footprint/",
    },
    Def {
        id: "phone",
        kicker: "One new phone",
        title: "Apple product environmental reports: ≈70 kg CO₂e per flagship iPhone",
        url: "https://www.apple.com/environment/",
    },
    Def {
        id: "streaming",
        kicker: "Streaming video",
        title: "Carbon Trust (2021): the carbon impact of video streaming — ≈55 g CO₂e/hour",
        url: "https://www.carbontrust.com/our-work-and-impact/guides-reports-and-tools/carbon-impact-of-video-streaming",
    },
    Def {
        id: "ai-openai",
        kicker: "AI query energy",
        title: "OpenAI / Sam Altman (2025), company figure: the average ChatGPT query uses ≈0.34 Wh",
        url: "https://blog.samaltman.com/the-gentle-singularity",
    },
    Def {
        id: "ai-google",
        kicker: "AI query emissions",
        title: "Google (2025), company figure: a median Gemini text prompt measures 0.24 Wh, 0.03 g CO₂e",
        url: "https://cloud.google.com/blog/products/infrastructure/measuring-the-environmental-impact-of-ai-inference",
    },
    Def {
        id: "ai-agent",
        kicker: "Agentic coding energy",
        title: "Couch (2026): electricity use of AI coding agents — Claude per-token energy back-figured from billing rates, anchored to Epoch AI’s GPT-4o measurement",
        url: "https://simonpcouch.com/blog/2026-01-20-cc-impact/",
    },
    Def {
        id: "ai-epoch",
        kicker: "The per-query anchor",
        title: "Epoch AI (2025): how much energy does ChatGPT use? — ≈0.3 Wh per GPT-4o query, ≈1 second of one H100 at ≈1,050 W",
        url: "https://epoch.ai/gradient-updates/how-much-energy-does-chatgpt-use",
    },
    Def {
        id: "seaice-guardian",
        kicker: "Arctic sea ice, explained",
        title: "The Guardian on the sea-ice figure and what it means",
        url: "https://www.theguardian.com/environment/2016/nov/03/your-carbon-footprint-destroys-30-square-metres-of-arctic-sea-ice-a-year",
    },
    Def {
        id: "why-ice",
        kicker: "Why sea ice matters",
        title: "NSIDC: quick facts on Arctic sea ice",
        url: "https://nsidc.org/learn/parts-cryosphere/sea-ice",
    },
    Def {
        id: "albedo",
        kicker: "Albedo feedback",
        title: "NASA Earth Observatory: the Arctic is absorbing more sunlight",
        url: "https://science.nasa.gov/earth/earth-observatory/the-arctic-is-absorbing-more-sunlight-84930",
    },
    Def {
        id: "credit-trf",
        kicker: "Original site credit",
        title: "Thomson Reuters Foundation (2019, via Euronews): Shame Plane and its creator",
        url: "https://www.euronews.com/2019/08/08/flight-shame-website-spells-out-emissions-toll-on-global-climate",
    },
    Def {
        id: "credit-fastco",
        kicker: "Original site credit",
        title: "Fast Company (2019): the designer behind Shame Plane",
        url: "https://www.fastcompany.com/90378092/this-site-will-show-you-exactly-how-ashamed-you-should-be-of-flying",
    },
    Def {
        id: "airports",
        kicker: "Airport data",
        title: "OurAirports open data (public domain)",
        url: "https://ourairports.com/data/",
    },
];

pub static SOURCES: LazyLock<Vec<Source>> = LazyLock::new(|| {
    SOURCE_DEFS
        .iter()
        .enumerate()
        .map(|(i, d)| Source {
            id: d.id,
            num: i + 1,
            kicker: d.kicker,
            title: d.title,
            url: d.url,
        })
        .collect()
});

pub fn source(id: &str) -> Option<&'static Source> {
    SOURCES.iter().find(|s| s.id == id)
}

/// "science.org →" — a popover's outbound-link label, cut from its URL.
fn host_label(url: &str) -> String {
    let host = url
        .split_once("://")
        .map_or(url, |(_, rest)| rest)
        .split('/')
        .next()
        .unwrap_or(url);
    format!("{} →", host.strip_prefix("www.").unwrap_or(host))
}

/// Every cite popover on a page needs its own DOM id, and the same source
/// is cited from several places; a process-wide counter keeps ids unique
/// across the initial render and any shard re-renders.
static CITE_SEQ: AtomicUsize = AtomicUsize::new(0);

/// A citation superscript: still the small number, but it opens the source
/// card in place as an inline popover — kicker, title, outbound link —
/// instead of jumping to a list at the foot of the page. The panel lives
/// inside the `<sup>` so consecutive cites stay adjacent for the CSS comma.
#[component]
pub async fn cite(id: &str) -> Result {
    let s = source(id).unwrap_or_else(|| panic!("Unknown source anchor: {id}"));
    let pid = format!("cite-{}-{}", s.id, CITE_SEQ.fetch_add(1, Ordering::Relaxed));
    let anchor_name = format!("anchor-name: --inline-popover-{pid};");
    let position_anchor = format!("position-anchor: --inline-popover-{pid};");
    let link_label = host_label(s.url);
    view! {
        <sup class="cite">
            <button
                type="button"
                class="inline-popover-trigger"
                popovertarget=(pid.as_str())
                style=(anchor_name.as_str())
                aria-label=(format!("Source: {}", s.title))
            >(s.num)</button>
            <span
                id=(pid.as_str())
                class="inline-popover-panel cite-panel"
                popover="auto"
                style=(position_anchor.as_str())
            >
                <button
                    type="button"
                    class="inline-popover-close"
                    popovertarget=(pid.as_str())
                    popovertargetaction="hide"
                    aria-label="Close popover"
                >"×"</button>
                <span class="inline-popover-kicker">(s.kicker)</span>
                <span class="inline-popover-preview">(s.title)</span>
                ext_link(class: "quiet-link", href: s.url, label: link_label.as_str())
            </span>
        </sup>
    }
}

/// The sources footer: the credit for the original Shame Plane and the
/// methodology notes. The source cards themselves open in place from each
/// [`cite`] superscript, so there is no list here to jump to. Copy ported
/// from the original's sources drawer.
#[component]
pub async fn sources_section() -> Result {
    view! {
        <footer class="sources-section" id="sources">
            <h2>"Sources"</h2>
            <p class="about-copy">
                "This page was originally a revival of "
                inline_popover(
                    id: "shameplane-origin",
                    label: "Shame Plane",
                    <span class="inline-popover-preview">
                        "The original enter-a-flight, watch-the-ice-melt site. I don't know
                         what happened to the original, but here's the Wayback Machine's copy
                    "</span>
                    ext_link(
                        class: "quiet-link",
                        href: "https://web.archive.org/web/2020/https://shameplane.com/",
                        label: "shameplane.com (archived) →"
                    )
                )
                " (2019–2024), the small site that made this calculation famous. Reporting at the \
                 time — the Thomson Reuters Foundation "
                cite(id: "credit-trf")
                " and Fast Company "
                cite(id: "credit-fastco")
                " — credits Stockholm web and typeface designer "
                <strong>"Victor Muller"</strong>
                ", who built it as a personal project about his own flying; the site signed off \
                 with a link to his studio page, "
                <a href="https://web.archive.org/web/20210127130235/https://grafikprogram.com/">
                    "Grafik + Program"
                </a>
                ". This version keeps his data and sources, fixes three calculation bugs (an \
                 operator typo that silently dropped the distance-squared term, a hard model \
                 cutoff where the documentation interpolates, and cabin-class weights swapped \
                 between the short- and long-haul models), and retires the shame: it’s a bill, \
                 not a verdict. No affiliation, no funding, nothing for sale."
            </p>
            <details class="methodology">
                <summary>"How the numbers are computed"</summary>
                <ul>
                    <li>
                        "Per-passenger "
                        <em>"fuel"</em>
                        " follows the myclimate flight formula "
                        <code>"F = (a·x² + b·x + c) / (S·PLF) · (1−CF) · CW"</code>
                        ", where "
                        <code>"x"</code>
                        " is great-circle distance plus a detour constant (50 km short-haul / \
                         125 km long-haul). Flights under 1,500 km use the short-haul curve, \
                         over 2,500 km the long-haul curve, linearly interpolated between. \
                         Burning a kg of jet fuel emits 3.15 kg of CO₂; making and delivering \
                         that kg adds 0.51 kg CO₂e (“making the fuel”). "
                        cite(id: "myclimate")
                        " The 2018 parameters are kept from the original site; fleets have grown \
                         somewhat more efficient since, so per-seat numbers likely run slightly \
                         high."
                    </li>
                    <li>
                        "The altitude lines price aviation’s non-CO₂ warming on the 100-year \
                         clock (GWP100): contrail cirrus at 0.63× the combustion CO₂, and net \
                         NOx + aerosols + water vapour at 0.11×, both from Lee et al. (2021), \
                         Table 5 "
                        cite(id: "lee2021")
                        " — the same table the UK’s official 2025 conversion factors round to \
                         their ×1.7 aviation uplift, likewise applied to combustion CO₂ only. "
                        cite(id: "defra2025")
                        " GWP100 keeps flights in the same currency as every other number on \
                         this page; a 20-year clock would run the altitude lines ≈4× larger."
                    </li>
                    <li>
                        "The contrail line is then re-priced by where the route flies. Teoh et \
                         al. (2024) publish contrail climate forcing per km flown in eleven \
                         regions; divided by the global mean these become the receipt’s “sky \
                         factor” — North Atlantic ≈2.4×, Europe ≈1.4×, China ≈0.28×, unlisted \
                         airspace 1.0 — averaged along the great circle. "
                        cite(id: "teoh2024")
                        " Contrails are weather: ≈2.7% of flights cause 80% of the forcing, so \
                         the line is this route’s expected value, not a measurement of your \
                         flight — and its central estimate is genuinely uncertain (Lee’s 5–95% \
                         band spans ≈⅓×–1.7×; Teoh’s newer simulation lands near the bottom of \
                         it). Premium cabins scale every line alike — a floor-space allocation \
                         of the whole aircraft, the same convention as the UK seating-class \
                         factors; the plane, not the seat, makes the contrail."
                    </li>
                    <li>
                        "Three corrections to the original Shame Plane implementation: its "
                        <code>"a·x²"</code>
                        " term was silently dropped by a JavaScript operator bug ("
                        <code>"^"</code>
                        " is XOR, not power), understating long-haul flights by roughly 10–15%; \
                         it used a hard 2,000 km cutoff instead of the documented interpolation; \
                         and it swapped the cabin-class weights between the haul models — the \
                         myclimate table gives short-haul economy 0.96 / business 1.26 and \
                         long-haul economy 0.80 / business 1.54 (lie-flat business claims more \
                         of a wide-body’s floor), so the original overbilled long-haul economy \
                         by ≈20% and underbilled long-haul business by ≈18%."
                    </li>
                    <li>
                        "Arctic sea ice: 3 m² of September sea-ice loss per tonne of CO₂ (Notz \
                         & Stroeve, Science, 2016). The coefficient is defined per tonne of CO₂ \
                         — not CO₂e — so the receipt bills it on the jet-fuel CO₂ line only; \
                         contrails don’t melt receipt ice. "
                        cite(id: "seaice")
                    </li>
                    <li>
                        "Travel allowance: the 1.5-Degree Lifestyles report’s 2030 milestone \
                         leaves 0.425 tCO₂e per person per year for all mobility — 17% of the \
                         2.5 t/yr footprint target (Annex D, Table D.1); that’s the ≈0.43 t/yr \
                         on the receipt, and “travel allowance used” divides this flight by it. \
                         The report calls the per-domain split indicative — a person who spends \
                         less of their total on housing or diet can spend more of it on travel. "
                        cite(id: "budgets")
                    </li>
                    <li>
                        "The comparison chart sets one CO2e scale as full bar width: a flight, \
                         an order, or — on the hub — one person’s 2030 1.5 °C year (≈2.5 t), \
                         drawn as a stacked bar of indicative domain shares (food ≈0.725 t, \
                         housing, travel ≈0.425 t, goods, services, leisure). "
                        cite(id: "budgets")
                        " Each domain row — transport, food, home, habits — picks a concrete \
                         unit and count so the bar stays readable (miles, tanks of gas, NYC→LA \
                         drives, meals, A/C hours) instead of “average American” period ladders. \
                         Header chips rewrite a row into a make-up story (skip driving, switch \
                         those miles to an EV, swap meals to vegan, sweat out the A/C, skip the \
                         soda). When a habit has a frequency (e.g. burgers per week), rows \
                         switch to per-day rates and a Day/Week/Month chip sets how much of \
                         that rate the bar shows; the hub and one-off orders stay absolute \
                         counts. Factors: gas car ≈0.40 kg/mi "
                        cite(id: "cars")
                        "; EV ≈0.10 kg/mi "
                        cite(id: "ev")
                        "; Scarborough meals "
                        cite(id: "diets")
                        "; hamburger ≈3.6 kg US-typical "
                        cite(id: "meat")
                        cite(id: "heller")
                        "; A/C, bottles, soda, ChatGPT as cited on their rows. \
                         Order-of-magnitude by design."
                    </li>
                    <li>
                        "AI queries: OpenAI puts the average ChatGPT query at ≈0.34 Wh — ≈0.13 \
                         g CO₂e at the US grid average, the figure used here. "
                        cite(id: "ai-openai")
                        " Google’s measured median for a Gemini text prompt is lower still \
                         (0.24 Wh, 0.03 g CO₂e market-based). "
                        cite(id: "ai-google")
                        " Both numbers are company-reported and exclude model training; the \
                         chart deliberately uses the higher of the two. Either way, a year of \
                         heavy chatbot use is invisible next to a flight. The “vibe coding this \
                         site” line prices this site’s own build — ≈15 hours of Claude (Fable \
                         5) at max effort. Per-token energy follows Couch’s coding-agent \
                         estimate, which hangs Anthropic’s billing rates on Epoch AI’s measured \
                         ≈0.3 Wh GPT-4o query "
                        cite(id: "ai-agent")
                        cite(id: "ai-epoch")
                        "; at Fable’s price tier, ≈1.5M generated tokens (≈2.9 kWh) plus ≈80M \
                         re-read, nearly all prompt-cache hits (≈3.1 kWh), come to ≈6 kWh — an \
                         H100 loafing along at ≈40% for the full 15 hours — ≈2.2 kg CO₂e at \
                         the U.S. grid average "
                        cite(id: "grid")
                        ". Call it a year and a half of the thirty-queries-a-day habit per \
                         build, and still a rounding error on the flight."
                    </li>
                    <li>
                        "“Externalities billed separately” is the literal situation, not just a \
                         joke: a web of postwar aviation treaties keeps jet fuel on \
                         international flights effectively untaxed, and most international \
                         fares still reach the gate without a carbon price — flights within \
                         Europe, covered by the EU and UK emissions trading schemes, are the \
                         main exception."
                    </li>
                </ul>
            </details>
        </footer>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_labels_cut_scheme_www_and_path() {
        assert_eq!(
            host_label("https://www.science.org/doi/10.1126/science.aag2345"),
            "science.org →"
        );
        assert_eq!(
            host_label("https://acp.copernicus.org/articles/24/6071/2024/"),
            "acp.copernicus.org →"
        );
        // Every source URL yields a real host, so no popover ships a bare
        // arrow.
        for s in SOURCES.iter() {
            assert!(host_label(s.url).len() > 2, "{}: empty host label", s.id);
        }
    }
}
