use crate::api::client::RedditClient;
use crate::error::Result;
use crate::output::format_output;

pub async fn get(id: &str, format: &str) -> Result<()> {
    let client = RedditClient::new().await?;
    let post = client.get_post(id).await?;

    format_output(&post, format)?;
    Ok(())
}

pub async fn comments(id: &str, sort: &str, limit: u32, format: &str) -> Result<()> {
    let client = RedditClient::new().await?;
    let comments = client.get_comments(id, sort, limit).await?;

    format_output(&comments, format)?;
    Ok(())
}
