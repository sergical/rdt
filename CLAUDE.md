# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run Commands

```bash
cargo build                    # Build debug
cargo build --release          # Build release
cargo run -- <args>            # Run with args
cargo install --path .         # Install locally
```

No tests are currently configured.

## Architecture

rdt is a Reddit CLI designed for AI agent consumption. It outputs JSON by default and supports natural language queries.

### Module Structure

- **`src/main.rs`** - CLI definition using clap derive. Defines all commands: `search`, `post`, `subreddit`, `user`, `auth`
- **`src/cli/`** - Command handlers (one file per command group)
- **`src/api/client.rs`** - Reddit API client. Supports both public API (`.json` suffix) and OAuth endpoints
- **`src/api/models.rs`** - Reddit API response types and output summaries
- **`src/nlp/`** - Natural language query parsing
  - `patterns.rs` - Regex-based pattern matching for common queries (instant, no AI)
  - `router.rs` - Decides between pattern matching and AI fallback
- **`src/config.rs`** - Config at `~/.config/rdt/config.toml`
- **`src/error.rs`** - Error types using thiserror

### Query Processing Flow

1. User query → `NlpRouter::parse_query()`
2. First tries `PatternMatcher::try_match()` for instant pattern matching
3. If no pattern match and query is complex, falls back to Claude Haiku on AWS Bedrock
4. `SearchParams` passed to `RedditClient::search()`

### Key Types

- `SearchParams` - Structured search parameters extracted from queries
- `Listing<T>` - Reddit's paginated response wrapper
- `PostSummary`, `CommentSummary`, etc. - Simplified output types

## Configuration

Config file: `~/.config/rdt/config.toml`

```toml
[reddit]
client_id = "..."
client_secret = "..."

[aws]
region = "us-east-1"
bedrock_model_id = "us.anthropic.claude-haiku-4-5-20251001-v1:0"
```

AWS credentials come from standard AWS SDK chain (env vars, `~/.aws/credentials`, IAM role).
