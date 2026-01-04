use crate::api::client::RedditClient;
use crate::error::Result;
use crate::output::format_output;

pub async fn info(name: &str, format: &str) -> Result<()> {
    let client = RedditClient::new().await?;
    let info = client.get_subreddit_info(name).await?;

    format_output(&info, format)?;
    Ok(())
}

pub async fn posts(name: &str, sort: &str, time: &str, limit: u32, format: &str) -> Result<()> {
    let client = RedditClient::new().await?;
    let posts = client.get_subreddit_posts(name, sort, time, limit).await?;

    format_output(&posts, format)?;
    Ok(())
}
