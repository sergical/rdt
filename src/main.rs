mod api;
mod cli;
mod config;
mod error;
mod nlp;
mod output;

use clap::{Parser, Subcommand};
use cli::{auth, post, search, subreddit, user};

#[derive(Parser)]
#[command(name = "rdt")]
#[command(author, version, about = "Reddit CLI for AI agents", long_about = None)]
struct Cli {
    /// Output format (json, table)
    #[arg(short, long, default_value = "json", global = true)]
    format: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Authentication commands
    Auth {
        #[command(subcommand)]
        action: AuthAction,
    },

    /// Search Reddit
    Search {
        /// Search query (supports natural language)
        query: String,

        /// Limit to specific subreddit
        #[arg(short, long)]
        subreddit: Option<String>,

        /// Search type: posts, comments
        #[arg(short = 't', long, default_value = "posts")]
        r#type: String,

        /// Sort order: hot, new, top, relevance
        #[arg(long, default_value = "relevance")]
        sort: String,

        /// Time filter: hour, day, week, month, year, all
        #[arg(long, default_value = "all")]
        time: String,

        /// Maximum number of results
        #[arg(short, long, default_value = "25")]
        limit: u32,
    },

    /// Post operations
    Post {
        #[command(subcommand)]
        action: PostAction,
    },

    /// Subreddit operations
    Subreddit {
        #[command(subcommand)]
        action: SubredditAction,
    },

    /// User operations
    User {
        #[command(subcommand)]
        action: UserAction,
    },
}

#[derive(Subcommand)]
enum AuthAction {
    /// Login to Reddit via OAuth
    Login,
    /// Check authentication status
    Status,
    /// Logout and clear credentials
    Logout,
}

#[derive(Subcommand)]
enum PostAction {
    /// Get a post by ID
    Get {
        /// Post ID (e.g., "abc123" or full URL)
        id: String,
    },
    /// Get comments for a post
    Comments {
        /// Post ID
        id: String,
        /// Sort order: best, top, new, controversial, old
        #[arg(long, default_value = "best")]
        sort: String,
        /// Maximum number of comments
        #[arg(short, long, default_value = "100")]
        limit: u32,
    },
}

#[derive(Subcommand)]
enum SubredditAction {
    /// Get subreddit info
    Info {
        /// Subreddit name
        name: String,
    },
    /// Get posts from a subreddit
    Posts {
        /// Subreddit name
        name: String,
        /// Sort order: hot, new, top, rising
        #[arg(long, default_value = "hot")]
        sort: String,
        /// Time filter for top posts
        #[arg(long, default_value = "day")]
        time: String,
        /// Maximum number of posts
        #[arg(short, long, default_value = "25")]
        limit: u32,
    },
}

#[derive(Subcommand)]
enum UserAction {
    /// Get user info
    Info {
        /// Username
        username: String,
    },
    /// Get user's posts
    Posts {
        /// Username
        username: String,
        /// Sort order: hot, new, top, controversial
        #[arg(long, default_value = "new")]
        sort: String,
        /// Maximum number of posts
        #[arg(short, long, default_value = "25")]
        limit: u32,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Auth { action } => match action {
            AuthAction::Login => auth::login().await,
            AuthAction::Status => auth::status().await,
            AuthAction::Logout => auth::logout().await,
        },
        Commands::Search {
            query,
            subreddit,
            r#type,
            sort,
            time,
            limit,
        } => {
            search::search(&query, subreddit.as_deref(), &r#type, &sort, &time, limit, &cli.format)
                .await
        }
        Commands::Post { action } => match action {
            PostAction::Get { id } => post::get(&id, &cli.format).await,
            PostAction::Comments { id, sort, limit } => {
                post::comments(&id, &sort, limit, &cli.format).await
            }
        },
        Commands::Subreddit { action } => match action {
            SubredditAction::Info { name } => subreddit::info(&name, &cli.format).await,
            SubredditAction::Posts {
                name,
                sort,
                time,
                limit,
            } => subreddit::posts(&name, &sort, &time, limit, &cli.format).await,
        },
        Commands::User { action } => match action {
            UserAction::Info { username } => user::info(&username, &cli.format).await,
            UserAction::Posts {
                username,
                sort,
                limit,
            } => user::posts(&username, &sort, limit, &cli.format).await,
        },
    };

    if let Err(e) = result {
        eprintln!("{}", serde_json::json!({
            "error": e.to_string(),
            "type": format!("{:?}", e).split('(').next().unwrap_or("Unknown")
        }));
        std::process::exit(1);
    }
}
