#[tokio::main]
async fn main() {
    if let Err(error) = atlasyr::run_migrations().await {
        eprintln!("database migration failed: {error}");
        std::process::exit(1);
    }
}
