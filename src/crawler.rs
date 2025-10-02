use anyhow::Result;
use futures::stream::{FuturesUnordered, StreamExt};
use scraper::{Html, Selector};
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use url::Url;

use crate::http_client::HttpClient;
use crate::email_extractor::{extract_emails_from_html, normalize_email};

#[derive(Clone)]
pub struct CrawlConfig {
    pub max_pages: usize,
    pub max_depth: usize,
}

impl CrawlConfig {
    pub fn new(max_pages: usize, max_depth: usize) -> Self {
        Self { max_pages, max_depth }
    }
}

pub struct CrawlResult {
    pub emails: HashSet<String>,
}

pub async fn crawl_site(
    http: Arc<HttpClient>,
    start_url: &Url,
    cfg: &CrawlConfig,
) -> Result<CrawlResult> {
    let mut emails: HashSet<String> = HashSet::new();
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<(Url, usize)> = VecDeque::new();

    // Normalize start URL (strip fragments and queries)
    let mut base = start_url.clone();
    base.set_fragment(None);
    let start = base;

    queue.push_back((start.clone(), 0));
    visited.insert(url_key(&start));

    let (root_host, www_host) = host_variants(&start);

    let link_sel = Selector::parse("a[href]").unwrap();

    let mut pages_fetched = 0usize;

    // Concurrency handled by HttpClient, but we'll still issue multiple inflight tasks
    while !queue.is_empty() && pages_fetched < cfg.max_pages {
        let mut inflight = FuturesUnordered::new();
        while inflight.len() < 4 && !queue.is_empty() && pages_fetched + inflight.len() < cfg.max_pages {
            if let Some((u, depth)) = queue.pop_front() {
                inflight.push(fetch_and_parse(http.clone(), u, depth));
            }
        }

        if inflight.is_empty() {
            break;
        }

        while let Some(item) = inflight.next().await {
            if let Ok((url, depth, html_opt)) = item {
                pages_fetched += 1;
                if let Some(html) = html_opt {
                    // Extract emails
                    for e in extract_emails_from_html(&html) {
                        emails.insert(normalize_email(&e));
                    }
                    // Extract links and enqueue
                    if depth < cfg.max_depth {
                        let document = Html::parse_document(&html);
                        for next in extract_links(&document, &link_sel, &url) {
                            if url_in_scope(&next, &root_host, &www_host) {
                                let key = url_key(&next);
                                if visited.insert(key) {
                                    queue.push_back((next, depth + 1));
                                }
                            }
                        }
                    }
                }
            }
            if pages_fetched >= cfg.max_pages { break; }
            if inflight.len() < 4 && !queue.is_empty() && pages_fetched + inflight.len() < cfg.max_pages {
                if let Some((u, depth)) = queue.pop_front() {
                    inflight.push(fetch_and_parse(http.clone(), u, depth));
                }
            }
        }
    }

    Ok(CrawlResult { emails })
}

async fn fetch_and_parse(http: Arc<HttpClient>, url: Url, depth: usize) -> Result<(Url, usize, Option<String>)> {
    let body = http.get_text(&url).await?; // Ok(None) on error or non-success
    Ok((url, depth, body))
}

fn url_key(u: &Url) -> String {
    let mut k = u.clone();
    k.set_fragment(None);
    k.as_str().to_string()
}

fn extract_links(document: &Html, sel: &Selector, base: &Url) -> Vec<Url> {
    document
        .select(sel)
        .filter_map(|el| el.value().attr("href"))
        .filter_map(|href| base.join(href).ok())
        .collect()
}

fn host_variants(u: &Url) -> (String, String) {
    let host = u.host_str().unwrap_or("").to_string();
    let root = host.strip_prefix("www.").unwrap_or(&host).to_string();
    let www = format!("www.{}", root);
    (root, www)
}

fn url_in_scope(u: &Url, root_host: &str, www_host: &str) -> bool {
    if let Some(h) = u.host_str() {
        return h.eq_ignore_ascii_case(root_host) || h.eq_ignore_ascii_case(www_host);
    }
    false
}
