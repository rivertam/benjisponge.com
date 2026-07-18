//! Experience data for the resume page. Titles, orgs, and dates follow Ben's
//! 2026 resume PDF (LinkedIn fills the internships the PDF omits); the
//! bullets carry the resume's facts, reworded into the site's quieter voice.
//! Empty fields mean the sources list nothing there.

pub struct Role {
    /// Year span stamped in the margin rail.
    pub span: &'static str,
    pub title: &'static str,
    pub org: &'static str,
    pub place: &'static str,
    /// The full date range as listed.
    pub dates: &'static str,
    /// What the work actually was, one line per point. Empty for internships.
    pub bullets: &'static [&'static str],
    /// The tools of the era, rendered as a mono footnote. Empty to omit.
    pub stack: &'static str,
}

pub static ROLES: [Role; 6] = [
    Role {
        span: "2024–now",
        title: "Co-founder, Executive Software Lead / Board Member",
        org: "DigiChem",
        place: "New York",
        dates: "Aug 2024 – present",
        bullets: &[
            "Co-founded a chemical synthesis startup with two chemists; raised \
             a seed round, along with the investor relations and business \
             development that follow from one.",
            "Leads a team of three engineers building a custom ERP/MRP for \
             novel chemical synthesis and optimization — zero to one.",
            "Durable agentic LLM workflows run through the product itself; \
             built with heavy use of the agentic coding tools (Claude Code, \
             Cursor, Codex, pi).",
        ],
        stack: "React Router · TypeScript · Prisma · Postgres · DBOS · Railway · GitHub Actions",
    },
    Role {
        span: "2017–2023",
        title: "Software Engineer",
        org: "Standard Bots",
        place: "New York / remote",
        dates: "Sep 2017 – Mar 2023",
        bullets: &[
            "First dedicated software engineer; left a post-Series A company \
             of ten-plus engineers and twenty employees.",
            "Built and maintained the robotics engine — kinematics, control \
             systems, motion planning, vision.",
            "Built the platform that runs the robots: a React remote control, \
             a real-time server, WebRTC, ThreeJS, WebAssembly.",
        ],
        stack: "C++ · TypeScript · Rust · Linux · Postgres · Firebase · Python · Docker",
    },
    Role {
        span: "2015–2016",
        title: "Software Engineer / Lead Engineer",
        org: "A Plus",
        place: "New York",
        dates: "Jun 2015 – Dec 2016",
        bullets: &["Ran every part of a digital publishing site's engineering."],
        stack: "Ruby on Rails · React · JavaScript",
    },
    Role {
        span: "2014",
        title: "Software Engineering Intern",
        org: "Wolverine Trading",
        place: "Chicago",
        dates: "Summer 2014",
        bullets: &[],
        stack: "",
    },
    Role {
        span: "2012",
        title: "Software Engineering Intern",
        org: "Royal Caribbean",
        place: "",
        dates: "Summer 2012",
        bullets: &[],
        stack: "",
    },
    Role {
        span: "2009",
        title: "Research Intern",
        org: "Jackson Memorial Hospital",
        place: "",
        dates: "Summer 2009",
        bullets: &[],
        stack: "",
    },
];

pub struct School {
    pub span: &'static str,
    pub school: &'static str,
    pub degree: &'static str,
    pub note: &'static str,
}

pub static EDUCATION: School = School {
    span: "2011–2014",
    school: "Washington University in St. Louis",
    degree: "BS, Computer Science",
    note: "Minor in Design.",
};

pub static SKILLS: [&str; 5] = ["Software Design", "Robotics", "C++", "TypeScript", "Rust"];
