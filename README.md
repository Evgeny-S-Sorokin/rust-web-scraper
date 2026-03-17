## Usage

Run the scraper with a search query:

```bash
cargo run -- "your search query"
```

Select user agent (optional, defaults to 0):

```bash
cargo run -- "query" --user-agent 1
```

User agents:
- 0: Windows Chrome
- 1: macOS Chrome
- 2: Linux Chrome
- 3: iPhone Safari
- 4: iPad Safari
- 5: iPhone GSA