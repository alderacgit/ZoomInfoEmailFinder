use anyhow::Result;
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::Semaphore;
use tokio::time::Duration;
use url::Url;

#[derive(Clone)]
pub struct HttpClient {
    client: Client,
    cache_dir: PathBuf,
    ttl: Duration,
    semaphore: Arc<Semaphore>,
}

impl HttpClient {
    pub fn new(cache_dir: PathBuf, ttl: Duration, concurrency: usize) -> Result<Self> {
        let client = Client::builder()
            .redirect(reqwest::redirect::Policy::limited(10))
            .brotli(true)
            .gzip(true)
            .user_agent("ZoomInfoEmailFinder/0.1")
            .build()?;
        Ok(Self {
            client,
            cache_dir,
            ttl,
            semaphore: Arc::new(Semaphore::new(concurrency.max(1))),
        })
    }

    pub async fn get_text(&self, url: &Url) -> Result<Option<String>> {
        let key = cache_key(url);
        let body_path = self.cache_dir.join(format!("{}.body", key));
        let meta_path = self.cache_dir.join(format!("{}.json", key));
        fs::create_dir_all(&self.cache_dir).await.ok();

        if let Ok(meta_bytes) = fs::read(&meta_path).await {
            if let Ok(meta) = serde_json::from_slice::<CacheMeta>(&meta_bytes) {
                let now = now_secs();
                if now.saturating_sub(meta.saved_at) <= self.ttl.as_secs() {
                    if let Ok(body) = fs::read_to_string(&body_path).await {
                        return Ok(Some(body));
                    }
                }
            }
        }

        let _permit = self.semaphore.acquire().await.unwrap();
        let resp = self.client.get(url.clone()).send().await;
        let resp = match resp {
            Ok(r) => r,
            Err(_) => return Ok(None),
        };
        if !resp.status().is_success() {
            return Ok(None);
        }
        let text = resp.text().await.unwrap_or_default();

        // Write cache
        if let Err(e) = write_cache(&body_path, &meta_path, &text).await {
            tracing::debug!("cache write error: {}", e);
        }

        Ok(Some(text))
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CacheMeta {
    saved_at: u64,
}

async fn write_cache(body_path: &Path, meta_path: &Path, text: &str) -> Result<()> {
    if let Some(parent) = body_path.parent() {
        fs::create_dir_all(parent).await.ok();
    }
    let mut f = fs::File::create(body_path).await?;
    f.write_all(text.as_bytes()).await?;

    let meta = CacheMeta { saved_at: now_secs() };
    let meta_bytes = serde_json::to_vec(&meta)?;
    let mut mf = fs::File::create(meta_path).await?;
    mf.write_all(&meta_bytes).await?;
    Ok(())
}

fn now_secs() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn cache_key(url: &Url) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url.as_str().as_bytes());
    let bytes = hasher.finalize();
    hex::encode(bytes)
}
