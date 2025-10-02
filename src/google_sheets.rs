use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use google_sheets4 as sheets4;
use sheets4::Sheets;
use sheets4::oauth2::{read_application_secret, InstalledFlowAuthenticator, InstalledFlowReturnMethod};

use hyper::client::HttpConnector;
use hyper_rustls::HttpsConnectorBuilder;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetRow {
    pub unique_id: String,
    pub website: Option<String>,
}

pub async fn fetch_rows(sheet_url: &str, client_secret_path: &Path) -> Result<Vec<SheetRow>> {
    let spreadsheet_id = extract_spreadsheet_id(sheet_url)
        .ok_or_else(|| anyhow!("Could not parse spreadsheet id from URL"))?;

    let secret = read_application_secret(client_secret_path)
        .await
        .context("Failed to read Google OAuth client secret JSON")?;

    let token_cache = token_cache_path()?;

    let auth = InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::Interactive)
        .persist_tokens_to_disk(token_cache)
        .build()
        .await
        .context("Failed to build authenticator")?;

    let https = HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_or_http()
        .enable_http1()
        .enable_http2()
        .build();
    let client: hyper::Client<_, hyper::Body> = hyper::Client::builder().build::<_, hyper::Body>(https);

    let hub = Sheets::new(client, auth);

    // Get spreadsheet metadata to determine the first sheet name
    let (_resp, sheet) = hub
        .spreadsheets()
        .get(&spreadsheet_id)
        .doit()
        .await
        .context("Failed to fetch spreadsheet metadata")?;

    let sheet_title = sheet
        .sheets
        .as_ref()
        .and_then(|sheets| sheets.get(0))
        .and_then(|s| s.properties.as_ref())
        .and_then(|p| p.title.clone())
        .ok_or_else(|| anyhow!("Spreadsheet has no visible sheets"))?;

    // Read all columns from the first sheet
    let range = format!("{}!A:Z", sheet_title);
    let (_resp, values) = hub
        .spreadsheets()
        .values_get(&spreadsheet_id, &range)
        .doit()
        .await
        .context("Failed to fetch sheet values")?;

    let rows_json = values.values.unwrap_or_default();
    if rows_json.is_empty() {
        return Ok(vec![]);
    }

    // Convert JSON values to Strings for easier handling
    let rows: Vec<Vec<String>> = rows_json
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|v| v.as_str().map(|s| s.to_string()).unwrap_or_else(|| v.to_string()))
                .collect()
        })
        .collect();

    // Header detection
    let headers = &rows[0];
    let unique_idx = 0usize; // First column named Unique_ID per spec
    let website_idx = detect_website_column(headers);

    let mut out = Vec::new();
    for row in rows.into_iter().skip(1) {
        let unique_id = row.get(unique_idx).cloned().unwrap_or_default();
        let website = website_idx
            .and_then(|i| row.get(i).cloned())
            .filter(|s| !s.trim().is_empty());
        if unique_id.trim().is_empty() {
            continue;
        }
        out.push(SheetRow { unique_id, website });
    }

    Ok(out)
}

fn detect_website_column(headers: &Vec<String>) -> Option<usize> {
    let candidates = ["website", "url", "site", "website url", "web site", "homepage", "home page"]; 
    for (i, h) in headers.iter().enumerate() {
        let hnorm = h.to_ascii_lowercase();
        if candidates.iter().any(|c| hnorm.contains(c)) {
            return Some(i);
        }
    }
    None
}

fn token_cache_path() -> Result<PathBuf> {
    let proj = "ZoomInfoEmailFinder";
    let dirs = directories::ProjectDirs::from("com", "Alderac", proj)
        .ok_or_else(|| anyhow!("Could not compute config directory"))?;
    Ok(dirs.config_dir().join("oauth_tokens.json"))
}

fn extract_spreadsheet_id(url: &str) -> Option<String> {
    // Matches /spreadsheets/d/{id}/
    let re = regex::Regex::new(r"/spreadsheets/d/([a-zA-Z0-9-_]+)").ok()?;
    let caps = re.captures(url)?;
    Some(caps.get(1)?.as_str().to_string())
}
