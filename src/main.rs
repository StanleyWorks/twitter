pub mod api;
pub mod server;
pub(crate) mod twitter;
pub mod config;

#[tokio::main]
async fn main() {
    server::run().await
}
