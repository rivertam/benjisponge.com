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

#[page("/experience")]
async fn experience() -> Result {
    let body = view! {
        page_head(stamp: "timeline", title: "Experience", lede: "")
        <section class="mt-14 space-y-12">
            for role in ROLES.iter() {
                <article class="rail-row">
                    <p class="rail-stamp">(role.span)</p>
                    <div class="min-w-0">
                        <h2 class="font-display text-2xl leading-snug font-semibold">(role.title)</h2>
                        <p class="mt-1 text-ink2">(org_line(role))</p>
                        if !role.note.is_empty() {
                            <p class="mt-1 text-ink2">(role.note)</p>
                        }
                        <p class="mt-2 font-meta text-xs text-muted">(role.dates)</p>
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
                <p class="min-w-0 font-meta text-sm text-ink2">(SKILLS.join(" · "))</p>
            </div>
        </section>
    }?;
    view! { shell(title: "Experience — Ben Berman", body: body) }
}
