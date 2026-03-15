use chrono::Utc;
use serde::Deserialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::core::plugin::{ConfigField, ConfigFieldType};
use crate::error::{Result, WorkOsError};
use crate::models::config::WorkOsConfig;

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

pub const GOOGLE_CLIENT_ID: Option<&str> = option_env!("GOOGLE_CLIENT_ID");
pub const GOOGLE_CLIENT_SECRET: Option<&str> = option_env!("GOOGLE_CLIENT_SECRET");

pub const OAUTH_SCOPES: &str =
    "https://www.googleapis.com/auth/calendar.readonly https://www.googleapis.com/auth/tasks.readonly";

/// Returns true if both app credentials were embedded at build time.
pub fn check_if_credentials_present() -> bool {
    GOOGLE_CLIENT_ID.map_or(false, |s| !s.is_empty())
        && GOOGLE_CLIENT_SECRET.map_or(false, |s| !s.is_empty())
}

/// Shared config schema for all Google plugins.
pub fn oauth_token_schema() -> Vec<ConfigField> {
    vec![
        ConfigField {
            name: "access_token",
            label: "Access Token",
            help: "Set automatically by `work-os auth google`",
            field_type: ConfigFieldType::Secret,
            required: false,
            default: None,
        },
        ConfigField {
            name: "refresh_token",
            label: "Refresh Token",
            help: "Set automatically by `work-os auth google`",
            field_type: ConfigFieldType::Secret,
            required: false,
            default: None,
        },
    ]
}

/// Per-user OAuth token stored in config.toml under [plugins.google].
/// Shared across all Google plugins since they use the same OAuth app.
#[derive(Debug, Clone)]
pub struct GoogleOAuthConfig {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<i64>,
}

// ============================================================================
// OAuth flow
// ============================================================================

/// Run the full OAuth2 browser flow and save the token to config.
pub async fn run_oauth_flow() -> Result<()> {
    let client_id = GOOGLE_CLIENT_ID.ok_or_else(|| {
        WorkOsError::Google(
            "GOOGLE_CLIENT_ID not set. Add it to .cargo/config.toml and rebuild.".into(),
        )
    })?;
    let client_secret = GOOGLE_CLIENT_SECRET.ok_or_else(|| {
        WorkOsError::Google(
            "GOOGLE_CLIENT_SECRET not set. Add it to .cargo/config.toml and rebuild.".into(),
        )
    })?;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| WorkOsError::Google(format!("Failed to bind local port: {}", e)))?;

    let port = listener
        .local_addr()
        .map_err(|e| WorkOsError::Google(format!("Failed to read local port: {}", e)))?
        .port();

    let redirect_uri = format!("http://127.0.0.1:{}/callback", port);

    let auth_url = format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&access_type=offline&prompt=consent",
        GOOGLE_AUTH_URL,
        urlencoding::encode(client_id),
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(OAUTH_SCOPES),
    );

    println!("Opening browser for Google authentication...");
    println!("If the browser does not open, visit:\n{}", auth_url);

    if let Err(e) = open::that(&auth_url) {
        eprintln!("Warning: Could not open browser: {}", e);
    }

    let code = tokio::time::timeout(
        std::time::Duration::from_secs(120),
        wait_for_callback(listener),
    )
    .await
    .map_err(|_| WorkOsError::Google("Authentication timed out (2 minutes)".into()))?
    .map_err(|e| WorkOsError::Google(format!("Callback error: {}", e)))?;

    let token = exchange_code(&code, &redirect_uri, client_id, client_secret).await?;
    save_token_to_config(&token)?;

    println!("Token saved to ~/.work-os/config.toml");
    Ok(())
}

/// Refresh an expired access token and persist the updated token.
pub async fn refresh_access_token(refresh_token: &str) -> Result<String> {
    let client_id = GOOGLE_CLIENT_ID
        .ok_or_else(|| WorkOsError::Google("GOOGLE_CLIENT_ID not embedded at build time".into()))?;
    let client_secret = GOOGLE_CLIENT_SECRET.ok_or_else(|| {
        WorkOsError::Google("GOOGLE_CLIENT_SECRET not embedded at build time".into())
    })?;

    let http = reqwest::Client::new();
    let params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
    ];

    let resp = http
        .post(GOOGLE_TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| WorkOsError::Google(format!("Token refresh request failed: {}", e)))?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(WorkOsError::Google(format!(
            "Token refresh failed: {}",
            body
        )));
    }

    #[derive(Deserialize)]
    struct RefreshResponse {
        access_token: String,
        expires_in: Option<i64>,
    }

    let data: RefreshResponse = resp
        .json()
        .await
        .map_err(|e| WorkOsError::Google(format!("Failed to parse refresh response: {}", e)))?;

    let expires_at = data.expires_in.map(|secs| Utc::now().timestamp() + secs);

    let token = GoogleOAuthConfig {
        access_token: data.access_token.clone(),
        refresh_token: Some(refresh_token.to_string()),
        expires_at,
    };
    save_token_to_config(&token)?;

    Ok(data.access_token)
}

// ============================================================================
// Token persistence — targeted raw TOML edit, never re-serializes the full config
// ============================================================================

/// Write the token fields into [plugins.google] by editing the raw TOML value.
/// This avoids re-serializing WorkOsConfig through serde, which would drop
/// nested tables (e.g. [plugins.google_calendar.colors]) due to a toml crate
/// limitation with #[serde(flatten)] on HashMaps containing table values.
fn save_token_to_config(token: &GoogleOAuthConfig) -> Result<()> {
    let config_path = WorkOsConfig::config_path()?;
    let contents = std::fs::read_to_string(&config_path)
        .map_err(|e| WorkOsError::Google(format!("Failed to read config: {}", e)))?;

    // Parse as raw toml::Value — no serde structs involved, no flatten issues
    let mut doc: toml::Value = toml::from_str(&contents)
        .map_err(|e| WorkOsError::Google(format!("Failed to parse config: {}", e)))?;

    // Navigate to plugins.google, creating the table if it doesn't exist
    let plugins = doc
        .as_table_mut()
        .and_then(|t| t.get_mut("plugins"))
        .and_then(|v| v.as_table_mut())
        .ok_or_else(|| WorkOsError::Google("Config is missing a [plugins] section".into()))?;

    let google = plugins
        .entry("google".to_string())
        .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));

    let google_table = google
        .as_table_mut()
        .ok_or_else(|| WorkOsError::Google("plugins.google is not a table".into()))?;

    google_table.insert(
        "access_token".to_string(),
        toml::Value::String(token.access_token.clone()),
    );
    if let Some(ref rt) = token.refresh_token {
        google_table.insert("refresh_token".to_string(), toml::Value::String(rt.clone()));
    }
    if let Some(exp) = token.expires_at {
        google_table.insert("expires_at".to_string(), toml::Value::Integer(exp));
    }

    let new_contents = toml::to_string_pretty(&doc)
        .map_err(|e| WorkOsError::Google(format!("Failed to serialize config: {}", e)))?;

    std::fs::write(&config_path, new_contents)
        .map_err(|e| WorkOsError::Google(format!("Failed to write config: {}", e)))
}

// ============================================================================
// Internal helpers
// ============================================================================

async fn wait_for_callback(listener: tokio::net::TcpListener) -> std::io::Result<String> {
    loop {
        let (mut socket, _) = listener.accept().await?;

        let mut buf = [0u8; 4096];
        let n = socket.read(&mut buf).await?;
        let request = std::str::from_utf8(&buf[..n]).unwrap_or("");

        if let Some(code) = parse_code_from_request(request) {
            let html = concat!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nConnection: close\r\n\r\n",
                "<!DOCTYPE html>",
                "<html>",
                "<head><title>Work OS</title></head>",
                "<body style='font-family:system-ui;max-width:480px;margin:80px auto;text-align:center;color:#1a1a1a'>",
                "<h1 style='font-size:24px;margin-bottom:8px'>🗂 Work OS</h1>",
                "<p style='color:#666;margin-bottom:32px'>Your personal work command center</p>",
                "<div style='background:#f0fdf4;border:1px solid #86efac;border-radius:8px;padding:24px'>",
                "<p style='font-size:20px;margin:0 0 8px'>✅ Google account connected</p>",
                "<p style='color:#555;margin:0'>Calendar and Tasks access granted.<br>You can close this tab and return to the terminal.</p>",
                "</div>",
                "</body></html>",
            );
            let _ = socket.write_all(html.as_bytes()).await;
            return Ok(code);
        }

        let _ = socket
            .write_all(b"HTTP/1.1 404 Not Found\r\nConnection: close\r\n\r\n")
            .await;
    }
}

fn parse_code_from_request(request: &str) -> Option<String> {
    let path = request.lines().next()?.split_whitespace().nth(1)?;
    let query = path.split('?').nth(1)?;

    for param in query.split('&') {
        if let Some(encoded) = param.strip_prefix("code=") {
            return urlencoding::decode(encoded).ok().map(|s| s.into_owned());
        }
    }
    None
}

async fn exchange_code(
    code: &str,
    redirect_uri: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<GoogleOAuthConfig> {
    let http = reqwest::Client::new();
    let params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("code", code),
        ("grant_type", "authorization_code"),
        ("redirect_uri", redirect_uri),
    ];

    let response = http
        .post(GOOGLE_TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| WorkOsError::Google(format!("Token exchange request failed: {}", e)))?;

    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(WorkOsError::Google(format!(
            "Token exchange failed: {}",
            body
        )));
    }

    #[derive(Deserialize)]
    struct TokenResponse {
        access_token: String,
        refresh_token: Option<String>,
        expires_in: Option<i64>,
    }

    let resp: TokenResponse = response
        .json()
        .await
        .map_err(|e| WorkOsError::Google(format!("Failed to parse token response: {}", e)))?;

    Ok(GoogleOAuthConfig {
        access_token: resp.access_token,
        refresh_token: resp.refresh_token,
        expires_at: resp.expires_in.map(|secs| Utc::now().timestamp() + secs),
    })
}
