//! Merged upstream patches, hand-picked from ~138. An aside, not a
//! headline: proof of interest in interesting things. Newest first.

pub struct Patch {
    /// Year merged, stamped in the margin rail.
    pub year: &'static str,
    pub repo: &'static str,
    /// The PR title, verbatim.
    pub title: &'static str,
    pub url: &'static str,
}

pub static PATCHES: [Patch; 10] = [
    Patch {
        year: "2019",
        repo: "actix/actix",
        title: "Add futures::{future, Stream} to actix::prelude::*",
        url: "https://github.com/actix/actix/pull/236",
    },
    Patch {
        year: "2019",
        repo: "actix/actix",
        title: "Add implementations for Message for Arc and Box",
        url: "https://github.com/actix/actix/pull/232",
    },
    Patch {
        year: "2018",
        repo: "rust-lang/rust",
        title: "Amend option.take examples",
        url: "https://github.com/rust-lang/rust/pull/52218",
    },
    Patch {
        year: "2018",
        repo: "yewstack/yew",
        title: "Add mount points",
        url: "https://github.com/yewstack/yew/pull/85",
    },
    Patch {
        year: "2018",
        repo: "nlohmann/json",
        title: "Better error 305",
        url: "https://github.com/nlohmann/json/pull/1221",
    },
    Patch {
        year: "2018",
        repo: "amethyst/amethyst",
        title: "Put in a better example for Component in the book",
        url: "https://github.com/amethyst/amethyst/pull/848",
    },
    Patch {
        year: "2018",
        repo: "Vincit/objection.js",
        title: "Overview for API#models and tabbed-example component",
        url: "https://github.com/Vincit/objection.js/pull/928",
    },
    Patch {
        year: "2017",
        repo: "yewstack/yew",
        title: "Add many console methods",
        url: "https://github.com/yewstack/yew/pull/55",
    },
    Patch {
        year: "2017",
        repo: "serverless/serverless",
        title: "Add missing apostrophe",
        url: "https://github.com/serverless/serverless/pull/3272",
    },
    Patch {
        year: "2016",
        repo: "yarnpkg/yarn",
        title: "Meaningful error message on malformed registry response",
        url: "https://github.com/yarnpkg/yarn/pull/1356",
    },
];
