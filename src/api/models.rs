use serde::{Deserialize, Serialize};

/// Reddit API listing response wrapper
#[derive(Debug, Deserialize)]
pub struct Listing<T> {
    pub kind: String,
    pub data: ListingData<T>,
}

#[derive(Debug, Deserialize)]
pub struct ListingData<T> {
    pub after: Option<String>,
    pub before: Option<String>,
    pub children: Vec<Thing<T>>,
}

#[derive(Debug, Deserialize)]
pub struct Thing<T> {
    pub kind: String,
    pub data: T,
}

/// Reddit post data
#[derive(Debug, Serialize, Deserialize)]
pub struct Post {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub subreddit: String,
    #[serde(default)]
    pub subreddit_name_prefixed: String,
    #[serde(default)]
    pub selftext: Option<String>,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub permalink: String,
    #[serde(default)]
    pub score: i64,
    #[serde(default)]
    pub upvote_ratio: f64,
    #[serde(default)]
    pub num_comments: u64,
    #[serde(default)]
    pub created_utc: f64,
    #[serde(default)]
    pub is_self: bool,
    #[serde(default)]
    pub over_18: bool,
    #[serde(default)]
    pub spoiler: bool,
    #[serde(default)]
    pub stickied: bool,
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub link_flair_text: Option<String>,
}

/// Simplified post for output
#[derive(Debug, Serialize)]
pub struct PostSummary {
    pub id: String,
    pub title: String,
    pub author: String,
    pub subreddit: String,
    pub url: String,
    pub score: i64,
    pub num_comments: u64,
    pub created_utc: f64,
}

impl From<Post> for PostSummary {
    fn from(p: Post) -> Self {
        Self {
            id: p.id,
            title: p.title,
            author: p.author,
            subreddit: p.subreddit,
            url: format!("https://reddit.com{}", p.permalink),
            score: p.score,
            num_comments: p.num_comments,
            created_utc: p.created_utc,
        }
    }
}

/// Reddit comment data
#[derive(Debug, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub name: String,
    pub author: String,
    pub body: String,
    pub score: i64,
    pub created_utc: f64,
    pub depth: Option<u32>,
    pub parent_id: String,
    pub link_id: String,
    #[serde(default)]
    pub replies: serde_json::Value, // Can be Listing or empty string
}

/// Simplified comment for output
#[derive(Debug, Serialize)]
pub struct CommentSummary {
    pub id: String,
    pub author: String,
    pub body: String,
    pub score: i64,
    pub created_utc: f64,
    pub depth: u32,
}

impl From<Comment> for CommentSummary {
    fn from(c: Comment) -> Self {
        Self {
            id: c.id,
            author: c.author,
            body: c.body,
            score: c.score,
            created_utc: c.created_utc,
            depth: c.depth.unwrap_or(0),
        }
    }
}

/// Subreddit info
#[derive(Debug, Serialize, Deserialize)]
pub struct Subreddit {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub display_name_prefixed: String,
    pub title: String,
    pub public_description: String,
    pub subscribers: u64,
    pub active_user_count: Option<u64>,
    pub created_utc: f64,
    pub over18: bool,
    pub url: String,
}

/// Simplified subreddit for output
#[derive(Debug, Serialize)]
pub struct SubredditSummary {
    pub name: String,
    pub title: String,
    pub description: String,
    pub subscribers: u64,
    pub active_users: Option<u64>,
    pub nsfw: bool,
    pub url: String,
}

impl From<Subreddit> for SubredditSummary {
    fn from(s: Subreddit) -> Self {
        Self {
            name: s.display_name,
            title: s.title,
            description: s.public_description,
            subscribers: s.subscribers,
            active_users: s.active_user_count,
            nsfw: s.over18,
            url: format!("https://reddit.com{}", s.url),
        }
    }
}

/// User info
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub link_karma: i64,
    pub comment_karma: i64,
    pub created_utc: f64,
    #[serde(default)]
    pub is_gold: bool,
    #[serde(default)]
    pub is_mod: bool,
    #[serde(default)]
    pub verified: bool,
}

/// Simplified user for output
#[derive(Debug, Serialize)]
pub struct UserSummary {
    pub name: String,
    pub link_karma: i64,
    pub comment_karma: i64,
    pub total_karma: i64,
    pub created_utc: f64,
    pub is_gold: bool,
}

impl From<User> for UserSummary {
    fn from(u: User) -> Self {
        Self {
            name: u.name,
            link_karma: u.link_karma,
            comment_karma: u.comment_karma,
            total_karma: u.link_karma + u.comment_karma,
            created_utc: u.created_utc,
            is_gold: u.is_gold,
        }
    }
}

/// Search results wrapper
#[derive(Debug, Serialize)]
pub struct SearchResults {
    pub query: String,
    pub subreddit: Option<String>,
    pub sort: String,
    pub posts: Vec<PostSummary>,
    pub count: usize,
}
