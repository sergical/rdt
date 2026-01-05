use crate::api::client::RedditClient;
use crate::error::Result;
use crate::nlp::router::{NlpRouter, SearchParams};
use crate::output::format_output;

// CLI defaults (must match main.rs)
const DEFAULT_SORT: &str = "relevance";
const DEFAULT_TIME: &str = "all";
const DEFAULT_LIMIT: u32 = 25;

pub async fn search(
    query: &str,
    subreddit: Option<&str>,
    search_type: &str,
    sort: &str,
    time: &str,
    limit: u32,
    format: &str,
) -> Result<()> {
    let router = NlpRouter::new();

    // If user provided explicit --subreddit flag, use explicit params
    // Otherwise, try NLP parsing (pattern matching or AI)
    let mut params = if subreddit.is_some() {
        // User explicitly specified subreddit, use as-is
        SearchParams {
            query: query.to_string(),
            subreddit: subreddit.map(String::from),
            sort: sort.to_string(),
            time: time.to_string(),
            limit,
            search_type: search_type.to_string(),
        }
    } else {
        // Try NLP parsing (pattern matching first, then AI if needed)
        router.parse_query(query).await?
    };

    // CLI flags override NLP-parsed values when explicitly set (not default)
    if sort != DEFAULT_SORT {
        params.sort = sort.to_string();
    }
    if time != DEFAULT_TIME {
        params.time = time.to_string();
    }
    if limit != DEFAULT_LIMIT {
        params.limit = limit;
    }
    if search_type != "posts" {
        params.search_type = search_type.to_string();
    }

    let client = RedditClient::new().await?;
    let results = client.search(&params).await?;

    format_output(&results, format)?;
    Ok(())
}
