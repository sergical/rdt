# rdt

```
██████╗ ██████╗ ████████╗██╗   ██╗██╗
██╔══██╗██╔══██╗╚══██╔══╝██║   ██║██║
██████╔╝██║  ██║   ██║   ██║   ██║██║
██╔══██╗██║  ██║   ██║   ██║   ██║██║
██║  ██║██████╔╝   ██║   ╚██████╔╝██║
╚═╝  ╚═╝╚═════╝    ╚═╝    ╚═════╝ ╚═╝
```

Reddit in your terminal. Like `gh` for GitHub, but for Reddit.

## Features

- **TUI mode** - browse Reddit interactively with `rdt tui`
- **JSON output** - pipe-friendly for scripts and automation
- **Natural language search** - `rdt search "rust in programming"` automatically searches r/programming
- **Smart query parsing** - common patterns resolved instantly, complex queries use AI

## Installation

### Homebrew (macOS/Linux)

```bash
brew install sergical/tap/rdt
```

### Cargo (Rust)

```bash
cargo install rdtcli
```

### From source

```bash
git clone https://github.com/sergical/rdt.git
cd rdt
cargo install --path .
```

### Pre-built binaries

Download from [GitHub Releases](https://github.com/sergical/rdt/releases):

- **macOS** (Apple Silicon): `rdt-aarch64-apple-darwin.tar.gz`
- **macOS** (Intel): `rdt-x86_64-apple-darwin.tar.gz`
- **Linux** (x64): `rdt-x86_64-unknown-linux-gnu.tar.gz`
- **Windows** (x64): `rdt-x86_64-pc-windows-msvc.zip`

## Usage

### Search

```bash
# Simple search
rdt search "rust async"

# Natural language (pattern matched)
rdt search "rust in programming"        # searches r/programming
rdt search "top rust"                   # sorts by top
rdt search "recent golang tutorials"    # sorts by new

# Explicit flags
rdt search "rust" --subreddit programming --sort top --limit 10
```

### Subreddits

```bash
rdt subreddit info rust
rdt subreddit posts rust --sort hot --limit 20
```

### Users

```bash
rdt user info spez
rdt user posts spez --limit 10
```

### Posts

```bash
rdt post get <post_id>
rdt post comments <post_id> --limit 50
```

### Auth

```bash
rdt auth status
rdt auth login   # Opens browser for OAuth (requires Reddit API approval)
rdt auth logout
```

## Output Format

All commands output JSON by default:

```json
{
  "query": "rust",
  "subreddit": "programming",
  "sort": "relevance",
  "posts": [
    {
      "id": "abc123",
      "title": "Why Rust is Great",
      "author": "rustacean",
      "subreddit": "programming",
      "url": "https://reddit.com/r/programming/comments/abc123/...",
      "score": 1234,
      "num_comments": 56,
      "created_utc": 1234567890.0
    }
  ],
  "count": 1
}
```

Use `--format table` for human-readable output (coming soon).

## Configuration

Config stored at `~/.config/rdt/config.toml`:

```toml
[reddit]
client_id = "your_client_id"  # Required for OAuth, optional for read-only

[aws]
region = "us-east-1"
bedrock_model_id = "us.anthropic.claude-haiku-4-5-20251001-v1:0"
```

## Natural Language Patterns

These patterns are matched instantly (no AI needed):

| Pattern | Example | Result |
|---------|---------|--------|
| `<query> in <subreddit>` | "rust in programming" | subreddit=programming |
| `top <query>` | "top rust tutorials" | sort=top |
| `recent <query>` | "recent news" | sort=new |
| `<query> from this week` | "rust from this week" | time=week |
| `<query> limit <n>` | "rust limit 5" | limit=5 |

Complex queries fall back to Claude Haiku on AWS Bedrock.

## Roadmap

### Read Operations
- [x] Search posts/comments
- [x] Get post details and comments
- [x] Subreddit info and posts
- [x] User info and posts
- [x] JSON output

### Natural Language Processing
- [x] Pattern matching for common queries
- [x] AI fallback (Claude Haiku on Bedrock)
- [ ] Evals for AI query parsing

### Write Operations 🚧
*Blocked: Requires [Reddit API approval](https://support.reddithelp.com/hc/en-us/articles/42728983564564)*
- [x] OAuth browser flow (ready)
- [ ] Create posts
- [ ] Create comments
- [ ] Voting

### TUI Mode
- [x] Interactive browser (`rdt tui`)
- [x] Search with NLP
- [x] Post list navigation
- [x] Comment viewing with expandable threads
- [x] Image preview support
- [x] Timestamps on posts and comments

### Future
- [ ] Monitor mode (subreddit polling)
- [ ] Table output format

## License

MIT
