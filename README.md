# ZoomInfoEmailFinder

Rust CLI tool that:
- Reads a publicly-accessible Google Sheet (no OAuth required)
- For each row (a company), crawls the company website only
- Extracts and selects the best contact email using simple heuristics
- Writes results to CSV with headers: `Unique_ID,Email`

Specs
- Language: Rust (stable)
- Input: Public Google Sheet via CSV export of a share link
  - First column header: `Unique_ID`
  - Detect website column by common names ("Website", "URL", "Site", case-insensitive)
  - Use a link like `https://docs.google.com/spreadsheets/d/<ID>/edit#gid=<gid>`
- Output: `./output/results.csv`, includes rows with no email (blank value)
- Crawl scope: company website only (same hostname; treat `www.` variant as equivalent)
- robots.txt: ignored (per spec)
- Concurrency:
  - Per-site HTTP concurrency: default 4 (configurable via `--concurrency`)
  - Row-level parallelism: default 4 (configurable via `--row-concurrency`)
- Timeouts: 3s connect, 5s total per HTTP request
- Heuristics for selection:
  1) Prefer same-domain emails as the website
  2) Rank by local-part: `contact@` > `info@` > `sales@` > others
  3) Deduplicate and validate; choose a stable tiebreaker (lexicographic)
- Errors: proceed to next row
- Config: via CLI flags or interactive prompts
- License: MIT

CLI
```
zoominfo-email-finder --sheet-url <SHEET_URL> \
  --output ./output/results.csv \
  [--row-concurrency 4] [--concurrency 4] \
  [--max-pages 50] [--max-depth 5] \
  [--cache-dir ./.cache] [--cache-ttl-secs 604800]
```
- If required arguments are missing, an interactive prompt will ask for them.
- You can also set `SHEET_URL` in the environment.

Examples
```
# Basic usage with a public Google Sheet share link
cargo run -- --sheet-url "https://docs.google.com/spreadsheets/d/XXXX/edit#gid=0"

# Faster: process 8 rows at a time, keep per-site concurrency at 4
cargo run -- --sheet-url "https://docs.google.com/spreadsheets/d/XXXX/edit#gid=0" \
  --row-concurrency 8 --concurrency 4

# Customize crawl limits and cache
cargo run -- --sheet-url "https://docs.google.com/spreadsheets/d/XXXX/edit#gid=0" \
  --max-pages 40 --max-depth 5 --cache-dir ./.cache --cache-ttl-secs 604800
```

How it works
- Downloads the first sheet (or a specific gid) via `gviz/tq?tqx=out:csv` export
- For each row, crawls only the company’s website domain up to `--max-depth` (default 5) and `--max-pages` per site
- Extracts emails from HTML and `mailto:` links, validates them, and selects the best match per heuristics
- Writes `Unique_ID,Email` to the output CSV in the same row order as the sheet

Progress & performance
- Shows an overall progress bar across rows
- Parallelizes row processing (bounded by `--row-concurrency`)
- Per-site request concurrency limited by `--concurrency`
- HTTP connect timeout: 3s; overall request timeout: 5s

Development
- Rust 1.88+
- Run: `cargo run -- --help`
- Tests: `cargo test`

Notes
- Intentionally ignores robots.txt as requested
- Currently uses static HTML fetching; headless rendering is not enabled by default
- For domain matching, `example.com` and `www.example.com` are considered equivalent; other subdomains are excluded by default
