use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::sync::Arc;
use tracing_subscriber::{fmt, EnvFilter};
use url::Url;
use indicatif::{ProgressBar, ProgressStyle};

mod cli;
mod crawler;
mod email_extractor;
mod google_sheets;
mod http_client;

use cli::Args;
use crawler::{crawl_site, CrawlConfig};
use email_extractor::choose_best_email;
use google_sheets::fetch_rows;
use http_client::HttpClient;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging with env filter, defaulting to info
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).init();

    let args = Args::parse().ensure_interactive()?;

    tracing::info!("Starting ZoomInfoEmailFinder");

    // Prepare output directory
    if let Some(parent) = args.output.parent() { fs::create_dir_all(parent).ok(); }

    // Fetch rows from public Google Sheets CSV export
    let sheet_url = args.sheet_url.clone().context("Sheet URL required")?;
    let rows = fetch_rows(&sheet_url).await?;
    let total = rows.len();
    tracing::info!("Fetched {} rows from sheet", total);

    // Prepare HTTP client with cache and concurrency
    let http = Arc::new(HttpClient::new(args.cache_dir.clone(), std::time::Duration::from_secs(args.cache_ttl_secs), args.concurrency)?);

    let cfg = CrawlConfig::new(args.max_pages, args.max_depth);

    // CSV writer
    let mut wtr = csv::Writer::from_path(&args.output)?;
    wtr.write_record(["Unique_ID", "Email"]).ok();

    // Progress bar for overall row processing
    let pb = ProgressBar::new(total as u64);
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} | {msg}")
            .unwrap()
            .progress_chars("=>-"),
    );

    for row in rows {
        pb.set_message(format!("{}", row.website.as_deref().unwrap_or("(no website)")));
        let unique = row.unique_id.clone();
        let website = row.website.clone().unwrap_or_default();
        let mut chosen: Option<String> = None;

        if let Ok(start) = normalize_start_url(&website) {
            match crawl_site(http.clone(), &start, &cfg).await {
                Ok(result) => {
                    let host = start.host_str().unwrap_or("");
                    chosen = choose_best_email(result.emails.iter(), host);
                }
                Err(e) => {
                    tracing::warn!("crawl error for {}: {}", website, e);
                }
            }
        } else {
            tracing::warn!("Invalid website URL for {}: {}", unique, website);
        }

        wtr.write_record([unique, chosen.unwrap_or_default()]).ok();
        pb.inc(1);
    }

    pb.finish_with_message("done");

    wtr.flush().ok();

    tracing::info!("Done. Wrote {}", args.output.display());

    Ok(())
}

fn normalize_start_url(raw: &str) -> Result<Url> {
    let mut s = raw.trim().to_string();
    if !s.starts_with("http://") && !s.starts_with("https://") {
        s = format!("https://{}", s);
    }
    let mut url = Url::parse(&s).context("parse url")?;
    url.set_fragment(None);
    Ok(url)
}
