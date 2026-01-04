use crate::api::client::RedditClient;
use crate::error::Result;
use crate::nlp::router::NlpRouter;
use crate::output::format_output;

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
    let params = if subreddit.is_some() {
        // User explicitly specified subreddit, use as-is
        crate::nlp::router::SearchParams {
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

    let client = RedditClient::new().await?;
    let results = client.search(&params).await?;

    format_output(&results, format)?;
    Ok(())
}
