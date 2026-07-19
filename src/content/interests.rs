//! The off-the-clock interests: each one gets a page under
//! `/off-the-clock/{stamp}` (the stamp doubles as the URL slug), plus the
//! nav dropdown and the index page. Sourced from an actual investigation
//! (YouTube uploads, Reddit history, GitHub, TypeRacer) — every claim here
//! is checkable.

pub struct Evidence {
    pub label: &'static str,
    pub url: &'static str,
}

pub struct Interest {
    /// Margin stamp on the index page AND the URL slug — keep it lowercase,
    /// one word.
    pub stamp: &'static str,
    /// Page title.
    pub title: &'static str,
    /// The one-liner: index row text, page lede.
    pub line: &'static str,
    /// Page body paragraphs.
    pub blurb: &'static [&'static str],
    pub links: &'static [Evidence],
}

const fn ev(label: &'static str, url: &'static str) -> Evidence {
    Evidence { label, url }
}

pub const INTERESTS: [Interest; 8] = [
    Interest {
        stamp: "drums",
        title: "Drums",
        line: "Mediocre drummer. Recording turns out to be much harder than \
               playing.",
        blurb: &[
            "I had my first lesson at summer camp, which is to say I've been \
             playing long enough that I should be better. These days it's a \
             Tama pancake kit — quiet enough for an apartment, portable \
             enough that I have set it up in a park.",
            "Two covers survive public scrutiny.",
        ],
        links: &[
            ev(
                "Taylor Swift cover →",
                "https://www.youtube.com/watch?v=VaKI7J2M2Ms",
            ),
            ev(
                "Manchester Orchestra cover →",
                "https://www.youtube.com/watch?v=8lrjsP1KWrY",
            ),
        ],
    },
    Interest {
        stamp: "swing",
        title: "Swing dancing",
        line: "Swing dancing (lead and follow but mostly lead)",
        blurb: &[
            "I started with group classes in midtown in 2023 and it \
             immediately ate my evenings — there was a stretch where I was \
             at a social most nights of the week.",
            "The pitch, which I will deliver unprompted: you rotate partners \
             every few minutes, nobody knows you, and it is the single most \
             efficient way to make a week better. If you're in New York, \
             take the intro class. If you're not, your city has a scene too.",
        ],
        links: &[],
    },
    Interest {
        stamp: "lifting",
        title: "Lifting",
        line: "Deadlift PR 345 lbs, Squat PR 235 lbs, Bench PR like 165 but I never 1RM it",
        blurb: &[
            "Five days a week, mostly the big compounds, entirely \
             plant-powered. The numbers above are not impressive and I am at \
             peace with that; the streak is the point.",
        ],
        links: &[],
    },
    Interest {
        stamp: "keys",
        title: "Keyboards",
        line: "Split-columnar keyboard person. Ten thousand strangers have \
               watched my Dactyl Manuform video; TypeRacer has me at 117wpm.",
        blurb: &[
            "The keyboard is a Dactyl Manuform from ohkeycaps — marble case, \
             lubed 67g Zilents, SA keycaps. I made a showcase video in 2021 \
             and ten thousand switch-curious strangers have watched it \
             since, which makes it my most successful publication in any \
             medium.",
            "The typing speed is real and independently auditable: 117wpm \
             average, 165 peak.",
        ],
        links: &[
            ev(
                "the keyboard →",
                "https://www.youtube.com/watch?v=yZl30vWuERs",
            ),
            ev(
                "TypeRacer →",
                "https://data.typeracer.com/pit/profile?user=rivertam",
            ),
        ],
    },
    Interest {
        stamp: "spire",
        title: "Slay the Spire",
        line: "Slay the Spire at Ascension 20, with an annotated run \
               synopsis, because a win nobody can audit barely counts.",
        blurb: &[
            "Ascension 20 is the highest difficulty the game offers. The \
             writeup exists because the win deserved documentation more than \
             it deserved celebration.",
            "I also maintain opinions about the RNG that I can support with \
             screenshots.",
        ],
        links: &[ev(
            "the synopsis →",
            "https://reddit.com/r/slaythespire/comments/jkqx35/annotated_run_synopsis_my_second_a20_win_only_a/",
        )],
    },
    Interest {
        stamp: "models",
        title: "Toy models",
        line: "Procedural cities with opinionated residents — a \
               react-three-fiber toy running Schelling-style agents.",
        blurb: &[
            "A procedural city in react-three-fiber: tensor-field streets, \
             lots, buildings, parks, and agents with Schelling-style \
             preferences about who they live near.",
            "There is a fifteen-minute video in which I explain all of this \
             with the confidence of someone who has not yet found the bugs.",
        ],
        links: &[
            ev("the video →", "https://www.youtube.com/watch?v=Bcd_9LvUr-8"),
            ev("the repo →", "https://github.com/rivertam/City"),
        ],
    },
    Interest {
        stamp: "puzzles",
        title: "Crosswords",
        line: "A Rust crossword engine, so .puz files open in the terminal. \
               Nobody had asked for this.",
        blurb: &[
            "puzuzu parses AcrossLite .puz files and gives you a solving TUI \
             — in Rust, published to npm, demo recording in the README.",
            "It has three GitHub stars and I earned every one of them.",
        ],
        links: &[ev("puzuzu →", "https://github.com/rivertam/puzuzu")],
    },
    Interest {
        stamp: "felix",
        title: "Felix",
        line: "There is a dog. There is, accordingly, a website computing \
               when we are the same age.",
        blurb: &[
            "The dog is Felix. The website is saamd.com — Same Age As My \
             Dog — which computes the one day on which a dog and its person \
             are, in dog years, the same age.",
            "This date matters to no one, which is why it needed a \
             calculator.",
        ],
        links: &[ev("saamd.com →", "https://www.saamd.com")],
    },
];
