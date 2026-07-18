//! Experience data, sourced from LinkedIn (2026-07-18). Facts only — titles,
//! orgs, date ranges, places as listed; nothing invented. Empty fields mean
//! LinkedIn lists nothing there.

pub struct Role {
    /// Year span stamped in the margin rail.
    pub span: &'static str,
    pub title: &'static str,
    pub org: &'static str,
    pub place: &'static str,
    /// The full date range as listed.
    pub dates: &'static str,
    /// One plain factual line, where the source gives one.
    pub note: &'static str,
}

pub static ROLES: [Role; 6] = [
    Role {
        span: "2024–now",
        title: "Co-Founder / Lead Software Engineer",
        org: "DigiChem",
        place: "New York",
        dates: "Aug 2024 – present",
        note: "",
    },
    Role {
        span: "2017–2023",
        title: "Software Engineer",
        org: "Standard Bots",
        place: "New York / remote",
        dates: "Sep 2017 – Mar 2023",
        note: "C++ and TypeScript.",
    },
    Role {
        span: "2015–2016",
        title: "Software Engineer",
        org: "A Plus",
        place: "New York",
        dates: "Jun 2015 – Dec 2016",
        note: "",
    },
    Role {
        span: "2014",
        title: "Software Engineering Intern",
        org: "Wolverine Trading",
        place: "Chicago",
        dates: "Summer 2014",
        note: "",
    },
    Role {
        span: "2012",
        title: "Software Engineering Intern",
        org: "Royal Caribbean",
        place: "",
        dates: "Summer 2012",
        note: "",
    },
    Role {
        span: "2009",
        title: "Research Intern",
        org: "Jackson Memorial Hospital",
        place: "",
        dates: "Summer 2009",
        note: "",
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
