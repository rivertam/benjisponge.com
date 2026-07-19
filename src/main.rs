mod app;
mod content;
mod design;
mod flight;

#[tokio::main]
async fn main() {
    // `--routes` prints the canonical route manifest, one "path<TAB>name"
    // per line, for scripts/snapshot — the capture list derives from the
    // same registries the site renders from, so it can't silently drift.
    if std::env::args().any(|arg| arg == "--routes") {
        for route in content::routes::site_routes() {
            println!("{route}\t{}", content::routes::route_name(&route));
        }
        return;
    }
    topcoat::start(app::router()).await.unwrap();
}
