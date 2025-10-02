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

impl Args {
    pub fn ensure_interactive(mut self) -> anyhow::Result<Self> {
        if self.sheet_url.is_none() {
            if let Ok(url) = std::env::var("SHEET_URL") { self.sheet_url = Some(url); }
        }
        if self.client_secret.is_none() {
            if let Ok(path) = std::env::var("GOOGLE_CLIENT_SECRET_PATH") { self.client_secret = Some(PathBuf::from(path)); }
        }
        if self.sheet_url.is_none() {
            let ans: String = dialoguer::Input::new()
                .with_prompt("Enter Google Sheet URL")
                .interact_text()?;
            self.sheet_url = Some(ans);
        }
        if self.client_secret.is_none() {
            let ans: String = dialoguer::Input::new()
                .with_prompt("Path to Google OAuth client_secret.json")
                .interact_text()?;
            self.client_secret = Some(PathBuf::from(ans));
        }
        Ok(self)
    }
}
