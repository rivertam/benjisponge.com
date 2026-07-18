mod experience;
mod thoughts;

use topcoat::{
    Result,
    asset::{AssetBundle, RouterBuilderAssetExt},
    router::{Router, RouterBuilderDiscoverExt, page},
    view::view,
};

use crate::design::shell;

pub fn router() -> Router {
    Router::builder()
        .assets(AssetBundle::load().unwrap())
        .discover()
        .build()
}

#[page("/")]
async fn home() -> Result {
    let body = view! {
        <h1 class="mt-16 font-display text-6xl font-bold">"I like Rust."</h1>
        <p class="mt-4 max-w-prose text-ink2">
            "Ben Berman — software engineer in New York. Skeleton page."
        </p>
    }?;
    view! { shell(title: "Ben Berman", body: body) }
}
