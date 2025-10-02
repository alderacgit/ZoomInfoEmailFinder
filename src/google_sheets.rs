use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetRow {
    pub unique_id: String,
    pub website: Option<String>,
}

// Fetch public CSV export for the first sheet (or a specific gid if present)
pub async fn fetch_rows(sheet_url: &str) -> Result<Vec<SheetRow>> {
    let spreadsheet_id = extract_spreadsheet_id(sheet_url)
        .ok_or_else(|| anyhow!("Could not parse spreadsheet id from URL"))?;
    let gid = extract_gid(sheet_url);

    let mut export_url = format!(
        "https://docs.google.com/spreadsheets/d/{}/gviz/tq?tqx=out:csv",
        spreadsheet_id
    );
    if let Some(g) = gid { export_url.push_str(&format!("&gid={}", g)); }

    let resp = reqwest::get(&export_url).await.context("request CSV export")?;
    if !resp.status().is_success() {
        return Err(anyhow!("failed to fetch CSV export: {}", resp.status()));
    }
    let bytes = resp.bytes().await.context("read CSV bytes")?;

    let mut rdr = csv::Reader::from_reader(bytes.as_ref());
    let headers: Vec<String> = rdr
        .headers()
        .map(|h| h.iter().map(|s| s.to_string()).collect())
        .unwrap_or_default();

    let unique_idx = 0usize; // First column = Unique_ID
    let website_idx = detect_website_column(&headers);

    let mut out = Vec::new();
    for rec in rdr.records() {
        let rec = rec?;
        let unique_id = rec.get(unique_idx).unwrap_or("").to_string();
        let website = website_idx
            .and_then(|i| rec.get(i))
            .map(|s| s.to_string())
            .filter(|s| !s.trim().is_empty());
        if unique_id.trim().is_empty() { continue; }
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

fn extract_spreadsheet_id(url: &str) -> Option<String> {
    // Matches /spreadsheets/d/{id}/
    let re = regex::Regex::new(r"/spreadsheets/d/([a-zA-Z0-9-_]+)").ok()?;
    let caps = re.captures(url)?;
    Some(caps.get(1)?.as_str().to_string())
}

fn extract_gid(url: &str) -> Option<String> {
    // Looks for gid=digits in query or fragment
    let re = regex::Regex::new(r"[?#].*?gid=([0-9]+)").ok()?;
    let caps = re.captures(url)?;
    Some(caps.get(1)?.as_str().to_string())
}
