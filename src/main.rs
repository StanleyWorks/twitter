pub mod api;
pub mod server;

#[tokio::main]
async fn main() {
    server::run().await
}
