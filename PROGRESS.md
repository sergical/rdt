# rdt Progress Tracker

## Current Status: MVP Complete

## Completed

### Phase 1: Foundation
- [x] Project scaffolding (cargo new)
- [x] CLI structure with clap
- [x] Config file handling (~/.config/rdt/)
- [x] Basic error handling with thiserror

### Phase 2: Read Operations + NLP
- [x] Search posts/comments
- [x] Get post details
- [x] Get comments
- [x] Subreddit info/posts
- [x] User info/posts
- [x] JSON output formatting
- [x] Layer 1: ezff-style regex patterns
- [ ] Layer 2: AWS Bedrock client + Claude Haiku fallback
- [x] Router: pattern match → direct, else → AI
- [x] Graceful fallback if Bedrock unavailable

## In Progress

- [x] Verify Bedrock model IDs
- [x] Test AI fallback with real AWS credentials (working with Haiku 4.5)

## Blocked

### Phase 3: Write Operations (requires Reddit API approval)
Reddit now requires pre-approval before creating apps. Submit request at:
https://support.reddithelp.com/hc/en-us/articles/42728983564564

- [ ] Create posts (text/link)
- [ ] Create comments
- [ ] Voting (up/down/clear)
- [ ] Delete own content

## Not Started

### Phase 4: Monitor
- [ ] Subreddit polling
- [ ] NDJSON streaming output
- [ ] Filter support (regex on title/body)
- [ ] Graceful shutdown (Ctrl+C)

### Phase 5: TUI Mode (Ratatui)
- [ ] Basic TUI shell with ratatui + crossterm
- [ ] Search results view (scrollable list)
- [ ] Post detail view
- [ ] Comments view (threaded/nested)
- [ ] Keyboard navigation (vim-style)
- [ ] Live subreddit feed view

### Phase 6: Polish
- [x] README with examples
- [ ] `--help` documentation improvements
- [ ] GitHub Actions CI
- [ ] Release binaries (Linux, macOS, Windows)
- [ ] Structured JSON error messages
- [ ] Integration tests
- [ ] Publish to crates.io

## Known Issues

1. ~~Pattern matching doesn't compose (e.g., "top rust from this week" only matches "top")~~ **FIXED** - reordered patterns so specific ones match first
2. ~~OAuth login not implemented (read-only public API for now)~~ **FIXED** - Browser OAuth flow implemented
3. ~~`--limit` flag ignored when NLP parsing is used~~ **FIXED** - CLI flags now override NLP-parsed values

## Notes

- Model ID for Bedrock: `us.anthropic.claude-haiku-4-5-20251001-v1:0`
- Reddit API rate limit: 100 req/min on free tier
