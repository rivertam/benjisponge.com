//! The off-the-clock section: general interests, each with whatever public
//! evidence exists. Sourced from an actual investigation (YouTube uploads,
//! Reddit history, GitHub, TypeRacer) — every claim here is checkable.

pub struct Evidence {
    pub label: &'static str,
    pub url: &'static str,
}

pub struct Interest {
    pub stamp: &'static str,
    pub line: &'static str,
    pub links: &'static [Evidence],
}

const fn ev(label: &'static str, url: &'static str) -> Evidence {
    Evidence { label, url }
}

pub const INTERESTS: [Interest; 9] = [
    Interest {
        stamp: "drums",
        line: "Mediocre drummer — my words, from a 2018 ad seeking equally \
               mediocre bandmates. Recording turns out to be much harder than \
               playing.",
        links: &[
            ev(
                "Taylor Swift cover →",
                "https://www.youtube.com/watch?v=VaKI7J2M2Ms",
            ),
            ev(
                "Manchester Orchestra cover →",
                "https://www.youtube.com/watch?v=8lrjsP1KWrY",
            ),
            ev(
                "MIDI practice app →",
                "https://github.com/rivertam/drum-practice",
            ),
        ],
    },
    Interest {
        stamp: "swing",
        line: "Swing dancing, as a lead, at one point nearly every night. If \
               you stand still long enough I will try to recruit you.",
        links: &[],
    },
    Interest {
        stamp: "lifting",
        line: "Five days a week under a barbell, entirely plant-powered. \
               Happy to discuss either half of that sentence at length, \
               unprompted.",
        links: &[],
    },
    Interest {
        stamp: "keys",
        line: "Split-columnar keyboard person. Ten thousand strangers have \
               watched my Dactyl Manuform video; TypeRacer has me at 117wpm.",
        links: &[
            ev("the keyboard →", "https://www.youtube.com/watch?v=yZl30vWuERs"),
            ev(
                "TypeRacer →",
                "https://data.typeracer.com/pit/profile?user=rivertam",
            ),
        ],
    },
    Interest {
        stamp: "spire",
        line: "Slay the Spire at Ascension 20, with an annotated run \
               synopsis, because a win nobody can audit barely counts.",
        links: &[ev(
            "the synopsis →",
            "https://reddit.com/r/slaythespire/comments/jkqx35/annotated_run_synopsis_my_second_a20_win_only_a/",
        )],
    },
    Interest {
        stamp: "models",
        line: "Procedural cities with opinionated residents — a \
               react-three-fiber toy running Schelling-style agents.",
        links: &[
            ev("the video →", "https://www.youtube.com/watch?v=Bcd_9LvUr-8"),
            ev("the repo →", "https://github.com/rivertam/City"),
        ],
    },
    Interest {
        stamp: "puzzles",
        line: "A Rust crossword engine, so .puz files open in the terminal. \
               Nobody had asked for this.",
        links: &[ev("puzuzu →", "https://github.com/rivertam/puzuzu")],
    },
    Interest {
        stamp: "riding",
        line: "A motorcycle, ridden around Queens, with a completed safety \
               course and strong opinions on lane discipline.",
        links: &[],
    },
    Interest {
        stamp: "the dog",
        line: "There is a dog. There is, accordingly, a website computing \
               when we are the same age.",
        links: &[ev("saamd.com →", "https://www.saamd.com")],
    },
];
