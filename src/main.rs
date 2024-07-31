#[tokio::main]
async fn main() {
    gui::run().await;
}

mod gui;
mod capture;
mod network;