mod app;
mod content;
mod design;
mod flight;

#[tokio::main]
async fn main() {
    topcoat::start(app::router()).await.unwrap();
}
