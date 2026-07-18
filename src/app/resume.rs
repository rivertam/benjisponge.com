use topcoat::{Result, router::page, view::view};

use crate::{
    content::experience::{EDUCATION, ROLES, Role, SKILLS, chip},
    design::{page_head, shell},
};

fn org_line(role: &Role) -> String {
    if role.place.is_empty() {
        role.org.to_string()
    } else {
        format!("{} · {}", role.org, role.place)
    }
}

// The view! grammar wants plain calls, so the parsed [`Chip`] is reached
// through one accessor per field.
fn chip_class(spec: &str) -> &'static str {
    chip(spec).class
}

fn chip_name(spec: &str) -> &str {
    chip(spec).name
}

fn chip_href(spec: &str) -> Option<&str> {
    chip(spec).href
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
                                        <a class=(chip_class(tech)) href=(href)>(chip_name(tech))</a>
                                    } else {
                                        <span class=(chip_class(tech))>(chip_name(tech))</span>
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
                            <a class=(chip_class(skill)) href=(href)>(chip_name(skill))</a>
                        } else {
                            <span class=(chip_class(skill))>(chip_name(skill))</span>
                        }
                    }
                </div>
            </div>
        </section>
    }?;
    view! { shell(title: "Résumé — Ben Berman", body: body) }
}
