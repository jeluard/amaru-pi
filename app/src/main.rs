use amaru_pi::cli;
use std::{error::Error, io};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("AMARU_PI_LOGS_LEVEL")
                .unwrap_or_else(|_| EnvFilter::new("debug")),
        )
        .with_writer(io::stderr)
        .init();
    cli::handle().await
}
