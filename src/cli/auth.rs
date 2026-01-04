use crate::config::Config;
use crate::error::{RdtError, Result};

pub async fn login() -> Result<()> {
    let config = Config::load()?;

    // TODO: Implement OAuth2 flow
    // 1. Open browser to Reddit authorization URL
    // 2. Start local server to receive callback
    // 3. Exchange code for access token
    // 4. Store token in config

    println!("{}", serde_json::json!({
        "status": "not_implemented",
        "message": "OAuth login not yet implemented. Please create a Reddit app at https://www.reddit.com/prefs/apps and add credentials to config."
    }));

    Ok(())
}

pub async fn status() -> Result<()> {
    let config = Config::load()?;

    let authenticated = config.has_credentials();

    println!("{}", serde_json::json!({
        "authenticated": authenticated,
        "config_path": config.config_path().display().to_string(),
    }));

    Ok(())
}

pub async fn logout() -> Result<()> {
    let mut config = Config::load()?;
    config.clear_credentials()?;

    println!("{}", serde_json::json!({
        "status": "success",
        "message": "Logged out successfully"
    }));

    Ok(())
}
