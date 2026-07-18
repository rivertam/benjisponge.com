use topcoat::{Result, router::page, view::view};

use crate::{
    content::experience::{EDUCATION, ROLES, Role, SKILLS},
    design::{page_head, shell},
};

fn org_line(role: &Role) -> String {
    if role.place.is_empty() {
        role.org.to_string()
    } else {
        format!("{} · {}", role.org, role.place)
    }
}

/// Chip color encodes kind: languages oxidize, frameworks and disciplines
/// patina, tools and infrastructure stay steel.
fn chip_class(tech: &str) -> &'static str {
    match tech {
        "C++" | "TypeScript" | "Rust" | "Python" | "JavaScript" | "Ruby" => "chip chip-oxide",
        "React Router" | "React" | "Prisma" | "DBOS" | "Ruby on Rails" | "ROS"
        | "Software Design" | "Robotics" => "chip chip-patina",
        _ => "chip chip-steel",
    }
}

/// Project pages for the names a reader might not know on sight. Chips with
/// an entry here render as links; household names stay plain text.
fn chip_href(tech: &str) -> Option<&'static str> {
    match tech {
        "React Router" => Some("https://reactrouter.com"),
        "Prisma" => Some("https://www.prisma.io"),
        "DBOS" => Some("https://www.dbos.dev"),
        "Railway" => Some("https://railway.com"),
        "Graphite" => Some("https://graphite.dev"),
        "ROS" => Some("https://www.ros.org"),
        _ => None,
    }
}

#[page("/resume")]
async fn resume() -> Result {
    let body = view! {
        page_head(stamp: "timeline", title: "Résumé", lede: "")
        <section class="mt-14 space-y-12">
            for role in ROLES.iter() {
                <article class="rail-row">
                    <p class="rail-stamp">(role.span)</p>
                    <div class="min-w-0">
                        <h2 class="font-display text-2xl leading-snug font-semibold">(role.title)</h2>
                        <p class="mt-1 text-ink2">(org_line(role))</p>
                        if !role.bullets.is_empty() {
                            <ul class="role-bullets mt-3 max-w-prose space-y-1.5 text-ink2">
                                for bullet in role.bullets.iter() {
                                    <li>(*bullet)</li>
                                }
                            </ul>
                        }
                        <p class="mt-3 font-meta text-xs text-muted">(role.dates)</p>
                        if !role.stack.is_empty() {
                            <div class="mt-2 flex flex-wrap gap-1.5">
                                for tech in role.stack.iter() {
                                    if let Some(href) = chip_href(tech) {
                                        <a class=(chip_class(tech)) href=(href)>(*tech)</a>
                                    } else {
                                        <span class=(chip_class(tech))>(*tech)</span>
                                    }
                                }
                            </div>
                        }
                    </div>
                </article>
            }
        </section>

        <section class="mt-16 space-y-10 border-t border-hairline pt-10">
            <article class="rail-row">
                <p class="rail-stamp">(EDUCATION.span)</p>
                <div class="min-w-0">
                    <h2 class="font-display text-2xl leading-snug font-semibold">(EDUCATION.school)</h2>
                    <p class="mt-1 text-ink2">(EDUCATION.degree)</p>
                    <p class="mt-1 text-ink2">(EDUCATION.note)</p>
                </div>
            </article>
            <div class="rail-row">
                <p class="rail-stamp uppercase tracking-[0.18em]">"Skills"</p>
                <div class="flex min-w-0 flex-wrap gap-1.5">
                    for skill in SKILLS.iter() {
                        if let Some(href) = chip_href(skill) {
                            <a class=(chip_class(skill)) href=(href)>(*skill)</a>
                        } else {
                            <span class=(chip_class(skill))>(*skill)</span>
                        }
                    }
                </div>
            </div>
        </section>
    }?;
    view! { shell(title: "Résumé — Ben Berman", body: body) }
}
