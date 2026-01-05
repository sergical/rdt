use crate::config::Config;
use crate::error::{RdtError, Result};
use rand::Rng;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use url::Url;

const REDDIT_AUTH_URL: &str = "https://www.reddit.com/api/v1/authorize";
const REDDIT_TOKEN_URL: &str = "https://www.reddit.com/api/v1/access_token";
const REDIRECT_URI: &str = "http://127.0.0.1:8484";

// Reddit OAuth scopes needed for read/write operations
const SCOPES: &str = "read submit vote identity";

pub async fn login() -> Result<()> {
    let mut config = Config::load()?;

    // Check if client_id is configured
    let client_id = config
        .reddit
        .client_id
        .as_ref()
        .ok_or_else(|| RdtError::Auth(format!(
            "No client_id configured. To use Reddit's API:\n\
            1. Review Reddit's policies: https://support.reddithelp.com/hc/en-us/articles/42728983564564\n\
            2. Create app at https://www.reddit.com/prefs/apps (select 'installed app')\n\
            3. Set redirect URI to: {}\n\
            4. Add client_id to ~/.config/rdt/config.toml",
            REDIRECT_URI
        )))?
        .clone();

    // Start local server to receive OAuth callback (fixed port for Reddit app registration)
    let listener = TcpListener::bind("127.0.0.1:8484")
        .map_err(|e| RdtError::Auth(format!("Failed to start local server on port 8484: {}. Is another process using it?", e)))?;

    // Generate random state for CSRF protection
    let state: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    // Build authorization URL
    let auth_url = format!(
        "{}?client_id={}&response_type=code&state={}&redirect_uri={}&duration=permanent&scope={}",
        REDDIT_AUTH_URL,
        urlencoding::encode(&client_id),
        urlencoding::encode(&state),
        urlencoding::encode(REDIRECT_URI),
        urlencoding::encode(SCOPES)
    );

    println!("{}", serde_json::json!({
        "status": "waiting",
        "message": "Opening browser for Reddit authorization..."
    }));

    // Open browser
    if let Err(e) = open::that(&auth_url) {
        println!("{}", serde_json::json!({
            "status": "manual",
            "message": "Could not open browser automatically. Please open this URL:",
            "url": auth_url
        }));
        eprintln!("Browser open error: {}", e);
    }

    // Wait for callback
    let (mut stream, _) = listener.accept()
        .map_err(|e| RdtError::Auth(format!("Failed to accept connection: {}", e)))?;

    let mut reader = BufReader::new(&stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)
        .map_err(|e| RdtError::Auth(format!("Failed to read request: {}", e)))?;

    // Parse the callback URL to get the authorization code
    let request_path = request_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| RdtError::Auth("Invalid callback request".to_string()))?;

    let callback_url = Url::parse(&format!("http://localhost{}", request_path))
        .map_err(|e| RdtError::Auth(format!("Failed to parse callback URL: {}", e)))?;

    // Check for error in callback
    if let Some(error) = callback_url.query_pairs().find(|(k, _)| k == "error") {
        let error_msg = error.1.to_string();
        send_response(&mut stream, "Authorization denied. You can close this window.");
        return Err(RdtError::Auth(format!("Authorization denied: {}", error_msg)));
    }

    // Extract authorization code and state
    let code = callback_url
        .query_pairs()
        .find(|(k, _)| k == "code")
        .map(|(_, v)| v.to_string())
        .ok_or_else(|| RdtError::Auth("No authorization code in callback".to_string()))?;

    let returned_state = callback_url
        .query_pairs()
        .find(|(k, _)| k == "state")
        .map(|(_, v)| v.to_string())
        .ok_or_else(|| RdtError::Auth("No state in callback".to_string()))?;

    // Verify state matches
    if returned_state != state {
        send_response(&mut stream, "Security error: state mismatch. You can close this window.");
        return Err(RdtError::Auth("State mismatch - possible CSRF attack".to_string()));
    }

    send_response(&mut stream, "Authorization successful! You can close this window and return to the terminal.");

    // Exchange code for access token
    let client = reqwest::Client::new();
    let token_response = client
        .post(REDDIT_TOKEN_URL)
        .basic_auth(&client_id, Some("")) // For installed apps, password is empty string
        .header("User-Agent", config.user_agent())
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("redirect_uri", REDIRECT_URI),
        ])
        .send()
        .await
        .map_err(|e| RdtError::Auth(format!("Token request failed: {}", e)))?;

    if !token_response.status().is_success() {
        let error_text = token_response.text().await.unwrap_or_default();
        return Err(RdtError::Auth(format!("Token exchange failed: {}", error_text)));
    }

    let token_data: serde_json::Value = token_response
        .json()
        .await
        .map_err(|e| RdtError::Auth(format!("Failed to parse token response: {}", e)))?;

    // Extract tokens
    let access_token = token_data["access_token"]
        .as_str()
        .ok_or_else(|| RdtError::Auth("No access_token in response".to_string()))?;

    let refresh_token = token_data["refresh_token"].as_str();

    // Save tokens to config
    config.reddit.access_token = Some(access_token.to_string());
    config.reddit.refresh_token = refresh_token.map(String::from);
    config.save()?;

    println!("{}", serde_json::json!({
        "status": "success",
        "message": "Successfully logged in to Reddit"
    }));

    Ok(())
}

fn send_response(stream: &mut std::net::TcpStream, message: &str) {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nConnection: close\r\n\r\n\
        <!DOCTYPE html><html><head><title>rdt</title></head>\
        <body style=\"font-family: system-ui; text-align: center; padding: 50px;\">\
        <h1>rdt</h1><p>{}</p></body></html>",
        message
    );
    let _ = stream.write_all(response.as_bytes());
}

pub async fn status() -> Result<()> {
    let config = Config::load()?;

    let has_client_id = config.reddit.client_id.is_some();
    let has_access_token = config.reddit.access_token.is_some();
    let has_refresh_token = config.reddit.refresh_token.is_some();

    println!("{}", serde_json::json!({
        "authenticated": has_access_token,
        "has_client_id": has_client_id,
        "has_refresh_token": has_refresh_token,
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
