use amaru_pi::cli;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    cli::handle().await
}
