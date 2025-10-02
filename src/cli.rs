use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum HeadlessMode {
    Auto,
    Always,
    Never,
}

#[derive(Parser, Debug)]
#[command(name = "zoominfo-email-finder", version, about = "Finds best contact emails for companies listed in a Google Sheet")] 
pub struct Args {
    /// Google Sheet share URL
    #[arg(long, env = "SHEET_URL")] 
    pub sheet_url: Option<String>,

    /// Output CSV path (headers: Unique_ID,Email)
    #[arg(long, default_value = "./output/results.csv")] 
    pub output: PathBuf,

    /// Path to Google OAuth client secret JSON (Desktop client)
    #[arg(long, env = "GOOGLE_CLIENT_SECRET_PATH")] 
    pub client_secret: Option<PathBuf>,

    /// Max concurrent HTTP requests
    #[arg(long, default_value_t = 4)]
    pub concurrency: usize,

    /// Max pages to crawl per website
    #[arg(long, default_value_t = 50)]
    pub max_pages: usize,

    /// Max depth to crawl from the start URL
    #[arg(long, default_value_t = 3)]
    pub max_depth: usize,

    /// Headless browser usage
    #[arg(long, value_enum, default_value_t = HeadlessMode::Auto)]
    pub use_headless: HeadlessMode,

    /// Cache directory for fetched pages
    #[arg(long, default_value = "./.cache")] 
    pub cache_dir: PathBuf,

    /// Cache TTL in seconds
    #[arg(long, default_value_t = 604800)]
    pub cache_ttl_secs: u64,

    /// Show computed configuration and exit
    #[arg(long, default_value_t = false)]
    pub show_config: bool,
}
