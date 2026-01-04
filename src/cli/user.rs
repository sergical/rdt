use crate::api::client::RedditClient;
use crate::error::Result;
use crate::output::format_output;

pub async fn info(username: &str, format: &str) -> Result<()> {
    let client = RedditClient::new().await?;
    let info = client.get_user_info(username).await?;

    format_output(&info, format)?;
    Ok(())
}

pub async fn posts(username: &str, sort: &str, limit: u32, format: &str) -> Result<()> {
    let client = RedditClient::new().await?;
    let posts = client.get_user_posts(username, sort, limit).await?;

    format_output(&posts, format)?;
    Ok(())
}
