#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tagisan::cli::run().await
}
