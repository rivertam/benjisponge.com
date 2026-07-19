mod app;
mod components;
mod content;
mod emdash;
mod flight;
mod util;

#[tokio::main]
async fn main() {
    topcoat::start(app::router()).await.unwrap();
}
