use crate::api::models::*;
use crate::config::Config;
use crate::error::{RdtError, Result};
use crate::nlp::router::SearchParams;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;

const REDDIT_API_BASE: &str = "https://oauth.reddit.com";
const REDDIT_PUBLIC_BASE: &str = "https://www.reddit.com";

pub struct RedditClient {
    client: reqwest::Client,
    config: Config,
    use_oauth: bool,
}

impl RedditClient {
    pub async fn new() -> Result<Self> {
        let config = Config::load()?;
        let use_oauth = config.has_credentials() && config.reddit.access_token.is_some();

        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&config.user_agent())
                .map_err(|e| RdtError::Config(e.to_string()))?,
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self {
            client,
            config,
            use_oauth,
        })
    }

    fn base_url(&self) -> &str {
        if self.use_oauth {
            REDDIT_API_BASE
        } else {
            REDDIT_PUBLIC_BASE
        }
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, endpoint: &str) -> Result<T> {
        // For public API, we need .json before query params
        // For OAuth API, no .json suffix needed
        let url = if self.use_oauth {
            format!("{}{}", REDDIT_API_BASE, endpoint)
        } else {
            // Split endpoint into path and query string to insert .json correctly
            let (path, query) = if let Some(idx) = endpoint.find('?') {
                (&endpoint[..idx], &endpoint[idx..])
            } else {
                (endpoint, "")
            };
            format!("{}{}.json{}", REDDIT_PUBLIC_BASE, path, query)
        };

        let mut request = self.client.get(&url);

        if self.use_oauth {
            if let Some(token) = &self.config.reddit.access_token {
                request = request.bearer_auth(token);
            }
        }

        let response = request.send().await?;

        if response.status() == 429 {
            return Err(RdtError::RateLimited);
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(RdtError::RedditApi(format!(
                "HTTP {}: {}",
                status, body
            )));
        }

        // Get the raw text first to debug deserialization issues
        let text = response.text().await?;

        let data: T = serde_json::from_str(&text).map_err(|e| {
            RdtError::RedditApi(format!(
                "JSON parse error: {} (first 500 chars: {})",
                e,
                &text[..text.len().min(500)]
            ))
        })?;

        Ok(data)
    }

    pub async fn search(&self, params: &SearchParams) -> Result<SearchResults> {
        let mut endpoint = if let Some(ref sub) = params.subreddit {
            format!("/r/{}/search", sub)
        } else {
            "/search".to_string()
        };

        let query_params = format!(
            "?q={}&sort={}&t={}&limit={}&restrict_sr={}",
            urlencoding::encode(&params.query),
            params.sort,
            params.time,
            params.limit,
            params.subreddit.is_some()
        );

        endpoint.push_str(&query_params);

        let listing: Listing<Post> = self.get(&endpoint).await?;

        let posts: Vec<PostSummary> = listing
            .data
            .children
            .into_iter()
            .map(|t| t.data.into())
            .collect();

        let count = posts.len();

        Ok(SearchResults {
            query: params.query.clone(),
            subreddit: params.subreddit.clone(),
            sort: params.sort.clone(),
            posts,
            count,
        })
    }

    pub async fn get_post(&self, id: &str) -> Result<PostSummary> {
        // Extract post ID from URL if needed
        let post_id = extract_post_id(id);

        let endpoint = format!("/by_id/t3_{}", post_id);
        let listing: Listing<Post> = self.get(&endpoint).await?;

        listing
            .data
            .children
            .into_iter()
            .next()
            .map(|t| t.data.into())
            .ok_or_else(|| RdtError::RedditApi("Post not found".to_string()))
    }

    pub async fn get_comments(
        &self,
        id: &str,
        sort: &str,
        limit: u32,
    ) -> Result<Vec<CommentSummary>> {
        let post_id = extract_post_id(id);

        let endpoint = format!("/comments/{}?sort={}&limit={}", post_id, sort, limit);

        // Reddit returns [post, comments] array
        let response: Vec<Listing<serde_json::Value>> = self.get(&endpoint).await?;

        let mut comments = Vec::new();

        if response.len() > 1 {
            for thing in response[1].data.children.iter() {
                if thing.kind == "t1" {
                    if let Ok(comment) = serde_json::from_value::<Comment>(thing.data.clone()) {
                        comments.push(comment.into());
                    }
                }
            }
        }

        Ok(comments)
    }

    pub async fn get_subreddit_info(&self, name: &str) -> Result<SubredditSummary> {
        let name = name.trim_start_matches("r/");
        let endpoint = format!("/r/{}/about", name);

        #[derive(Deserialize)]
        struct SubredditResponse {
            data: Subreddit,
        }

        let response: SubredditResponse = self.get(&endpoint).await?;
        Ok(response.data.into())
    }

    pub async fn get_subreddit_posts(
        &self,
        name: &str,
        sort: &str,
        time: &str,
        limit: u32,
    ) -> Result<Vec<PostSummary>> {
        let name = name.trim_start_matches("r/");
        let endpoint = format!("/r/{}/{}?t={}&limit={}", name, sort, time, limit);

        let listing: Listing<Post> = self.get(&endpoint).await?;

        let posts = listing
            .data
            .children
            .into_iter()
            .map(|t| t.data.into())
            .collect();

        Ok(posts)
    }

    pub async fn get_user_info(&self, username: &str) -> Result<UserSummary> {
        let username = username.trim_start_matches("u/");
        let endpoint = format!("/user/{}/about", username);

        #[derive(Deserialize)]
        struct UserResponse {
            data: User,
        }

        let response: UserResponse = self.get(&endpoint).await?;
        Ok(response.data.into())
    }

    pub async fn get_user_posts(
        &self,
        username: &str,
        sort: &str,
        limit: u32,
    ) -> Result<Vec<PostSummary>> {
        let username = username.trim_start_matches("u/");
        let endpoint = format!("/user/{}/submitted?sort={}&limit={}", username, sort, limit);

        let listing: Listing<Post> = self.get(&endpoint).await?;

        let posts = listing
            .data
            .children
            .into_iter()
            .map(|t| t.data.into())
            .collect();

        Ok(posts)
    }
}

/// Extract post ID from various formats
fn extract_post_id(input: &str) -> &str {
    // Handle full URLs like https://reddit.com/r/rust/comments/abc123/title
    if input.contains("/comments/") {
        if let Some(idx) = input.find("/comments/") {
            let rest = &input[idx + 10..];
            return rest.split('/').next().unwrap_or(input);
        }
    }

    // Handle t3_abc123 format
    if input.starts_with("t3_") {
        return &input[3..];
    }

    // Assume it's already just the ID
    input
}
