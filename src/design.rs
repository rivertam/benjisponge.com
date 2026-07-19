//! Shared chrome: fonts, the document shell, the margin rail, and page heads.
//!
//! The shell is "mill and oxide": cool steel paper, iron ink, one accent — the
//! color literally named rust. The signature is the margin rail (`.rail-row` /
//! `.rail-stamp` in `styles/site.css`): a narrow left column of Fira Mono
//! metadata, like the stamped margin of an engineering logbook.

use topcoat::{
    Result,
    font::{Font, fontsource::fontsource_font},
    view::{View, component, view},
};

use crate::content::{interests::INTERESTS, logbook::LOG};

pub const ZILLA_SLAB: Font = fontsource_font!(ZILLA_SLAB, host: Asset);
pub const FIRA_SANS: Font = fontsource_font!(FIRA_SANS, host: Asset);
pub const FIRA_MONO: Font = fontsource_font!(FIRA_MONO, host: Asset);

/// The full document: every page renders through this, so every page owns its
/// title. Pages invoke it as markup with the body as a prop:
/// `view! { shell(title: "…", active: "…", body: body) }`.
///
/// `active` names the nav item the page lives under — `"log"`, `"resume"`,
/// `"interests"`, or `""` for none — and gets the oxide underline.
#[component]
pub async fn shell(title: &str, active: &str, body: View) -> Result {
    let nav = |item: &str| {
        if active == item {
            "nav-active"
        } else {
            "quiet-link"
        }
    };
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <title>(title)</title>
                topcoat::dev::script()
                topcoat::runtime::script()
                <link rel="stylesheet" href=(topcoat::tailwind::stylesheet!())>
                topcoat::font::link(font: ZILLA_SLAB)
                topcoat::font::link(font: FIRA_SANS)
                topcoat::font::link(font: FIRA_MONO)
                <link
                    rel="alternate"
                    type="application/rss+xml"
                    title="Ben Berman — logbook"
                    href="/feed.xml"
                >
            </head>
            <body class="flex min-h-screen flex-col bg-page font-body text-ink">
                <header class="mx-auto flex w-full max-w-4xl items-baseline justify-between px-5 pt-6">
                    <a
                        href="/"
                        class="font-display text-lg font-semibold text-ink no-underline hover:text-oxide"
                    >"Ben Berman"</a>
                    <nav class="flex gap-6 font-meta text-sm">
                        <a href="/" class=(nav("log"))>"log"</a>
                        <a href="/resume" class=(nav("resume"))>"résumé"</a>
                        <details class="nav-dd">
                            <summary class=(nav("interests"))>"interests"</summary>
                            <div class="nav-dd-menu">
                                <a class="quiet-link" href="/interests">"all interests →"</a>
                                for interest in INTERESTS.iter() {
                                    <a
                                        class="quiet-link"
                                        href=(format!("/interests/{}", interest.slug))
                                    >(interest.slug)</a>
                                }
                            </div>
                        </details>
                    </nav>
                </header>
                <main class="mx-auto w-full max-w-4xl flex-1 px-5 pb-20">(body)</main>
                <footer class="mx-auto w-full max-w-4xl px-5 pb-8">
                    <div class="flex flex-wrap items-baseline justify-between gap-x-6 gap-y-2 border-t border-hairline pt-4 font-meta text-xs text-muted">
                        <span class="flex flex-wrap gap-x-5 gap-y-2">
                            <a
                                href="https://www.linkedin.com/in/benmberman"
                                class="quiet-link"
                            >"LinkedIn"</a>
                            <a href="https://github.com/rivertam" class="quiet-link">"GitHub"</a>
                            <a
                                href="https://www.reddit.com/user/BenjiSponge"
                                class="quiet-link"
                            >"Reddit"</a>
                        </span>
                        <span>
                            (format!("entry № {:04} of {:04} · ", LOG.len(), LOG.len()))
                            "made with "
                            <a href="https://github.com/tokio-rs/topcoat" class="quiet-link">"topcoat"</a>
                        </span>
                    </div>
                </footer>
            </body>
        </html>
    }
}

/// A page's opening rail row: a mono stamp in the margin, a Zilla Slab title,
/// and an optional one-line lede (pass `""` to omit).
#[component]
pub async fn page_head(stamp: &str, title: &str, lede: &str) -> Result {
    view! {
        <header class="rail-row mt-16">
            <p class="rail-stamp uppercase tracking-[0.18em]">(stamp)</p>
            <div class="min-w-0">
                <h1 class="font-display text-4xl font-bold tracking-tight">(title)</h1>
                if !lede.is_empty() {
                    <p class="mt-3 max-w-prose text-ink2">(lede)</p>
                }
            </div>
        </header>
    }
}
