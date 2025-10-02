use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{fmt, EnvFilter};

mod cli;
mod crawler;
mod email_extractor;
mod google_sheets;
mod http_client;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging with env filter, defaulting to info
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).init();

    let args = cli::Args::parse();

    // Entry point placeholder. Subsequent modules will implement:
    // 1) OAuth + Google Sheets fetch
    // 2) Website crawling + email extraction
    // 3) CSV output writing
    tracing::info!("Starting ZoomInfoEmailFinder");

    // TODO: wire up pipeline in later steps
    if args.show_config {
        println!("Config: concurrency={}, output={}", args.concurrency, args.output.display());
    }

    Ok(())
}
