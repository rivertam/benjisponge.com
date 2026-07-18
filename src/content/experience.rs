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
    /// The tools of the era, rendered as chips. Each entry is a spec of the
    /// form `[accent:]Name[ url]` — `oxide:` for languages, `patina:` for
    /// frameworks and disciplines, no prefix for tools and infrastructure
    /// (steel); a trailing ` https://…` turns the chip into a link. Parsed
    /// by [`chip`].
    pub stack: &'static [&'static str],
}

/// A stack/skill chip, parsed from its spec string.
pub struct Chip<'a> {
    pub name: &'a str,
    pub class: &'static str,
    pub href: Option<&'a str>,
}

/// Parse a chip spec (see [`Role::stack`] for the format).
pub fn chip(spec: &str) -> Chip<'_> {
    let (class, rest) = match spec.split_once(':') {
        Some(("oxide", rest)) => ("chip chip-oxide", rest),
        Some(("patina", rest)) => ("chip chip-patina", rest),
        // Any other colon belongs to the name (or a URL's `https://`).
        _ => ("chip chip-steel", spec),
    };
    match rest.rsplit_once(' ') {
        Some((name, url)) if url.starts_with("https://") || url.starts_with("http://") => Chip {
            name,
            class,
            href: Some(url),
        },
        _ => Chip {
            name: rest,
            class,
            href: None,
        },
    }
}

pub static ROLES: [Role; 6] = [
    Role {
        span: "2024–now",
        title: "Co-founder, Executive Software Lead / Board Member",
        org: "DigiChem",
        place: "New York",
        dates: "Aug 2024 – present",
        bullets: &[
            "Co-founded a chemical synthesis startup with two chemists",
            "Raised a seed round with MVP",
            "Lead and mentored a team of three engineers building a custom ERP/MRP for \
             novel chemical synthesis and optimization from zero to one, along with a handful of domain-oriented pivots",
            "Product powered by LLM-powered durable workflows",
        ],
        stack: &[
            "oxide:TypeScript",
            "patina:React Router https://reactrouter.com",
            "patina:Prisma https://www.prisma.io",
            "patina:DBOS https://www.dbos.dev",
            "Postgres",
            "Railway https://railway.com",
            "AWS",
            "GitHub Actions",
            "Graphite https://graphite.dev",
        ],
    },
    Role {
        span: "2017–2023",
        title: "Software Engineer",
        org: "Standard Bots",
        place: "New York / hybrid",
        dates: "Sep 2017 – Mar 2023",
        bullets: &[
            "First dedicated software engineer; left a post-Series A company \
             of ten-plus engineers and twenty employees",
            "Built and maintained the robotics engine — kinematics, dynamics, control \
             systems, motion planning, vision",
            "Built the platform that runs the robots: a React remote control, \
             a real-time server, WebRTC, ThreeJS, WebAssembly",
        ],
        stack: &[
            "oxide:C++",
            "oxide:TypeScript",
            "oxide:Rust",
            "patina:React",
            "Linux",
            "WebRTC",
            "Firebase",
            "Docker",
            "patina:ROS https://www.ros.org",
        ],
    },
    Role {
        span: "2015–2016",
        title: "Software Engineer / Lead Engineer",
        org: "A Plus",
        place: "New York",
        dates: "Jun 2015 – Dec 2016",
        bullets: &[
            "Joined a team of 6",
            "As the company dwindled, became lead and only engineer, running the entire site's engineering",
        ],
        stack: &[
            "oxide:Ruby",
            "oxide:JavaScript",
            "patina:Ruby on Rails",
            "patina:React",
            "AWS",
        ],
    },
    Role {
        span: "2014",
        title: "Software Engineering Intern",
        org: "Wolverine Trading",
        place: "Chicago",
        dates: "Summer 2014",
        bullets: &[
            "Fun summer during which I learned how to drink",
            "Good environment but low-impact and not intellectually stimulating",
            "While I already knew a good amount about finance, they did have us take an options trading course in which I learned a lot",
        ],
        stack: &[],
    },
    Role {
        span: "2012",
        title: "Software Engineering Intern",
        org: "Royal Caribbean",
        place: "",
        dates: "Summer 2012",
        bullets: &["Kinda regret this one. Cruises are terrible for the environment!"],
        stack: &[],
    },
    Role {
        span: "2009",
        title: "Research Intern",
        org: "Jackson Memorial Hospital",
        place: "",
        dates: "Summer 2009",
        bullets: &[
            "Kinda regret this one too. A lotta mice died, fortunately none directly by my hand.",
        ],
        stack: &[],
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

/// Chip specs, same format as [`Role::stack`].
pub static SKILLS: [&str; 5] = [
    "patina:Software Design",
    "patina:Robotics",
    "oxide:C++",
    "oxide:TypeScript",
    "oxide:Rust",
];
