# ZoomInfoEmailFinder

Rust CLI tool that:
- Reads a Google Sheet (via OAuth 2.0 user consent)
- For each row (a company), crawls the company website only
- Extracts and selects the best contact email using simple heuristics
- Writes results to CSV with headers: `Unique_ID,Email`

Project status: scaffolded. Implementation follows the spec below.

Specs (from user requirements)
- Language: Rust (stable)
- Input: Google Sheets API with OAuth (interactive consent flow)
  - First column header: `Unique_ID`
  - Detect website column by common names ("Website", "URL", "Site", case-insensitive)
- Output: `./output/results.csv`, include rows with no email (blank value)
- Crawl scope: company website only (same hostname; treat `www.` variant as equivalent)
- Concurrency: 4 requests
- robots.txt: ignored (per spec)
- JS-rendered pages: attempt static HTML; fallback to headless browser if needed
- Heuristics for selection:
  1) Prefer same-domain emails as the website
  2) Rank by local-part: `contact@` > `info@` > `sales@` > others
  3) Deduplicate and validate; choose a stable tiebreaker (lexicographic)
- Errors: proceed to next row
- Config: via CLI flags or interactive prompts
- License: MIT

Planned CLI
```
zoominfo-email-finder --sheet-url <SHEET_URL> \
  --output ./output/results.csv \
  [--concurrency 4] [--max-pages 50] [--max-depth 3] \
  [--use-headless auto|always|never] \
  [--cache-dir ./.cache] [--cache-ttl-secs 604800]
```
If required arguments are missing, an interactive prompt will ask for them.

OAuth Setup (Google Sheets)
1) Create an OAuth 2.0 Client ID (Desktop app) in Google Cloud Console.
2) Download the client secrets JSON (client_secret_*.json).
3) Provide its path to the program by either:
   - CLI: `--client-secret /path/to/client_secret.json`, or
   - Env var: `GOOGLE_CLIENT_SECRET_PATH=/path/to/client_secret.json`
4) On first run, a browser window will open to grant access. Token is stored in
   your OS config directory (e.g., `~/.config/ZoomInfoEmailFinder/`).

Development
- Rust 1.88+
- Run: `cargo run -- --help`
- Tests: `cargo test`

Notes
- This tool intentionally ignores robots.txt as requested.
- Headless mode requires Google Chrome; if not available, fallback to static HTML parsing will be used.
- For domain matching, `example.com` and `www.example.com` are considered equivalent. Other subdomains are excluded by default.
