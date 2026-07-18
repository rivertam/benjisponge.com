mod experience;
mod thoughts;

use topcoat::{
    Result,
    asset::{AssetBundle, RouterBuilderAssetExt},
    router::{Router, RouterBuilderDiscoverExt, page},
    view::view,
};

use crate::{
    content::{experience::ROLES, posts::POSTS},
    design::shell,
};

pub fn router() -> Router {
    Router::builder()
        .assets(AssetBundle::load().unwrap())
        .discover()
        .build()
}

#[page("/")]
async fn home() -> Result {
    let current = &ROLES[0];
    let body = view! {
        // Hero: the three-word bio, huge. "Rust." takes the accent — it is,
        // after all, the color the palette is named for.
        <section class="mt-20 sm:mt-28">
            <h1 class="font-display text-[3.4rem] leading-[0.95] font-bold tracking-tight sm:text-[5.5rem]">
                "I like "
                <span class="text-oxide">"Rust."</span>
            </h1>
            <p class="mt-6 max-w-prose text-lg text-ink2">
                "Ben Berman — software engineer in New York; co-founder of \
                 DigiChem; this site is where he thinks out loud."
            </p>
        </section>

        <section class="mt-24 space-y-10 border-t border-hairline pt-8">
            <div class="rail-row">
                <h2 class="rail-stamp uppercase tracking-[0.18em]">"Thoughts"</h2>
                <div></div>
            </div>
            for post in POSTS.iter() {
                <article class="rail-row">
                    <p class="rail-stamp">(post.date)</p>
                    <div class="min-w-0">
                        <h3 class="font-display text-2xl leading-snug font-semibold">
                            <a class="oxlink" href=(format!("/thoughts/{}", post.slug))>(post.title)</a>
                        </h3>
                        <p class="mt-1.5 max-w-prose text-ink2">(post.teaser)</p>
                    </div>
                </article>
            }
        </section>

        <section class="mt-20 space-y-8 border-t border-hairline pt-8">
            <div class="rail-row">
                <h2 class="rail-stamp uppercase tracking-[0.18em]">"Experience"</h2>
                <div></div>
            </div>
            <div class="rail-row">
                <p class="rail-stamp">(current.span)</p>
                <div class="min-w-0">
                    <p class="font-display text-xl leading-snug font-semibold">(current.title)</p>
                    <p class="mt-1 text-ink2">(format!("{} · {}", current.org, current.place))</p>
                    <p class="mt-4">
                        <a class="oxlink font-meta text-sm" href="/experience">"full timeline →"</a>
                    </p>
                </div>
            </div>
        </section>
    }?;
    view! { shell(title: "Ben Berman", body: body) }
}
