// Placeholder: crawler will implement site-restricted BFS crawl with concurrency control
// and optional headless rendering.

pub struct CrawlConfig {
    pub max_pages: usize,
    pub max_depth: usize,
}

impl CrawlConfig {
    pub fn new(max_pages: usize, max_depth: usize) -> Self {
        Self { max_pages, max_depth }
    }
}
