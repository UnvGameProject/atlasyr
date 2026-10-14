#[tokio::main]
async fn main() {
    if let Err(error) = atlasyr::run().await {
        eprintln!("application startup failed: {error}");
        std::process::exit(1);
    }
}
