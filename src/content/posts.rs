//! Post registry for the thoughts log. Posts are Rust modules (see
//! `src/app/thoughts/`); this is just the index metadata, newest first.
//! URL = `/thoughts/{slug}`.

pub struct Post {
    pub slug: &'static str,
    pub title: &'static str,
    pub date: &'static str,
    pub teaser: &'static str,
}

pub static POSTS: [Post; 2] = [
    Post {
        slug: "how-bad-are-planes",
        title: "How bad are planes?",
        date: "2026-07-12",
        teaser: "The part of the fare you don't pay.",
    },
    Post {
        slug: "pesky-code",
        title: "Pesky code",
        date: "2025-08-14",
        teaser: "On what AI frees me up to truly love.",
    },
];
