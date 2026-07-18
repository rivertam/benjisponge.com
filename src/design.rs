//! Shared chrome: fonts, the document shell, and the margin rail.

use topcoat::{
    Result,
    font::{Font, fontsource::fontsource_font},
    view::{View, component, view},
};

pub const ZILLA_SLAB: Font = fontsource_font!(ZILLA_SLAB, host: Asset);
pub const FIRA_SANS: Font = fontsource_font!(FIRA_SANS, host: Asset);
pub const FIRA_MONO: Font = fontsource_font!(FIRA_MONO, host: Asset);

/// The full document: every page renders through this, so every page owns its
/// title. Pages invoke it as markup with the body as a prop:
/// `view! { shell(title: "…", body: body) }`.
#[component]
pub async fn shell(title: &str, body: View) -> Result {
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
            </head>
            <body class="min-h-screen bg-page font-body text-ink">
                <header class="mx-auto flex max-w-4xl items-baseline justify-between px-5 pt-6">
                    <a href="/" class="font-display text-lg font-semibold no-underline">"Ben Berman"</a>
                    <nav class="flex gap-5 font-meta text-sm">
                        <a href="/thoughts" class="no-underline hover:text-oxide">"thoughts"</a>
                        <a href="/experience" class="no-underline hover:text-oxide">"experience"</a>
                    </nav>
                </header>
                <main class="mx-auto max-w-4xl px-5 pb-16">(body)</main>
            </body>
        </html>
    }
}
