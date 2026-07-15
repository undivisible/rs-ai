//! OAuth flows for ChatGPT and xAI/Grok (PKCE S256).
//! Uses the `oauth2` crate for PKCE challenge generation.
//! Based on openclaw openai-chatgpt-oauth-flow and pi-xai-oauth.

use oauth2::PkceCodeChallenge;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::{SystemTime, UNIX_EPOCH};

/// Supported OAuth providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OAuthProvider {
    /// OpenAI ChatGPT (oauth.openai.com).
    ChatGpt,
    /// xAI Grok (auth.x.ai).
    Xai,
}

impl OAuthProvider {
    fn client_id(&self) -> &str {
        match self {
            OAuthProvider::ChatGpt => "app_EMoamEEZ73f0CkXaXp7hrann",
            OAuthProvider::Xai => "b1a00492-073a-47ea-816f-4c329264a828",
        }
    }

    fn auth_url(&self) -> &str {
        match self {
            OAuthProvider::ChatGpt => "https://auth.openai.com/oauth/authorize",
            OAuthProvider::Xai => "https://auth.x.ai/oauth2/authorize",
        }
    }

    fn token_url(&self) -> &str {
        match self {
            OAuthProvider::ChatGpt => "https://auth.openai.com/oauth/token",
            OAuthProvider::Xai => "https://auth.x.ai/oauth2/token",
        }
    }

    fn scopes(&self) -> Vec<&str> {
        match self {
            OAuthProvider::ChatGpt => vec!["openid", "profile", "email", "offline_access"],
            OAuthProvider::Xai => {
                vec![
                    "openid",
                    "profile",
                    "email",
                    "offline_access",
                    "grok-cli:access",
                    "api:access",
                ]
            }
        }
    }

    fn redirect_port(&self) -> u16 {
        match self {
            OAuthProvider::ChatGpt => 1455,
            OAuthProvider::Xai => 56121,
        }
    }

    /// Provider name string.
    pub fn name(&self) -> &str {
        match self {
            OAuthProvider::ChatGpt => "chatgpt",
            OAuthProvider::Xai => "grok",
        }
    }
}

/// OAuth tokens returned by the token exchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: u64,
}

/// Errors during OAuth flow.
#[derive(Debug, thiserror::Error)]
pub enum OAuthError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Authentication failed: {0}")]
    Auth(String),
    #[error("OAuth flow timed out")]
    Timeout,
    #[error("Port {0} is in use")]
    PortInUse(u16),
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Start the OAuth flow for the given provider.
/// Opens the browser and waits for the callback on a localhost server.
/// Returns OAuth tokens on success.
pub fn start_oauth_flow(provider: OAuthProvider) -> Result<OAuthTokens, OAuthError> {
    let redirect_port = provider.redirect_port();
    let redirect_host = if matches!(provider, OAuthProvider::ChatGpt) {
        "0.0.0.0"
    } else {
        "127.0.0.1"
    };
    let redirect_path = if matches!(provider, OAuthProvider::ChatGpt) {
        "/auth/callback"
    } else {
        "/callback"
    };
    let redirect_uri_str = format!("http://{redirect_host}:{redirect_port}{redirect_path}");

    // Set up local callback server
    let listener = TcpListener::bind((redirect_host, redirect_port))
        .map_err(|_| OAuthError::PortInUse(redirect_port))?;
    listener.set_nonblocking(true).ok();

    // Generate PKCE challenge
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Build authorize URL manually
    let scopes = provider.scopes();
    let authorize_url = {
        let mut url = format!(
            "{}?response_type=code&client_id={}&redirect_uri={}&code_challenge={}&code_challenge_method=S256",
            provider.auth_url(),
            url_encode(provider.client_id()),
            url_encode(&redirect_uri_str),
            url_encode(pkce_challenge.as_str()),
        );
        for scope in scopes {
            url.push_str("&scope=");
            url.push_str(&url_encode(scope));
        }
        url
    };

    // Open browser
    open_browser(&authorize_url).map_err(OAuthError::Network)?;

    // Wait for callback
    let code = wait_for_callback(&listener)?;

    // Exchange code for tokens
    exchange_code(&provider, &code, pkce_verifier.secret(), &redirect_uri_str)
}

fn wait_for_callback(listener: &TcpListener) -> Result<String, OAuthError> {
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(180);

    while start.elapsed() < timeout {
        match listener.accept() {
            Ok((mut stream, _)) => {
                let mut buf = [0u8; 8192];
                let n = stream.read(&mut buf).unwrap_or(0);
                let req = String::from_utf8_lossy(&buf[..n]);
                let req_path = req
                    .lines()
                    .next()
                    .unwrap_or("")
                    .split(' ')
                    .nth(1)
                    .unwrap_or("");

                let query = req_path.split('?').nth(1).unwrap_or("");
                let params: HashMap<&str, String> = query
                    .split('&')
                    .filter_map(|pair| {
                        let mut parts = pair.splitn(2, '=');
                        let k = parts.next()?;
                        let v = url_decode(parts.next().unwrap_or(""));
                        Some((k, v))
                    })
                    .collect();

                if let Some(code) = params.get("code") {
                    let body = "<html><body style='font-family:sans-serif;background:#111;color:#eee;padding:40px'><h1>Connected</h1><p>You can close this tab.</p></body></html>";
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = stream.write_all(resp.as_bytes());
                    return Ok(code.clone());
                }

                let body =
                    "<html><body><h1>Error</h1><p>Missing authorization code.</p></body></html>";
                let resp = format!(
                    "HTTP/1.1 400 Bad Request\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(resp.as_bytes());
                return Err(OAuthError::Auth("Missing code in callback".into()));
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(_) => break,
        }
    }
    Err(OAuthError::Timeout)
}

fn exchange_code(
    provider: &OAuthProvider,
    code: &str,
    code_verifier: &str,
    redirect_uri: &str,
) -> Result<OAuthTokens, OAuthError> {
    use reqwest::blocking::Client as BlockingClient;

    let http_client = BlockingClient::new();

    let body = format!(
        "grant_type=authorization_code&client_id={}&code={}&redirect_uri={}&code_verifier={}",
        url_encode(provider.client_id()),
        url_encode(code),
        url_encode(redirect_uri),
        url_encode(code_verifier),
    );

    let response = http_client
        .post(provider.token_url())
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Accept", "application/json")
        .body(body)
        .send()
        .map_err(|e| OAuthError::Network(format!("token exchange request failed: {e}")))?;

    let v: serde_json::Value = response
        .json()
        .map_err(|e| OAuthError::Network(format!("token exchange parse failed: {e}")))?;

    let access_token = v
        .get("access_token")
        .and_then(|x| x.as_str())
        .ok_or_else(|| OAuthError::Auth("no access_token in response".into()))?
        .to_string();
    let refresh_token = v
        .get("refresh_token")
        .and_then(|x| x.as_str())
        .filter(|s| !s.is_empty())
        .map(String::from);
    let expires_in = v.get("expires_in").and_then(|x| x.as_u64()).unwrap_or(3600);
    let expires_at = now_secs() + expires_in;

    Ok(OAuthTokens {
        access_token,
        refresh_token,
        expires_at,
    })
}

/// Refresh an expired token using the refresh token.
pub async fn refresh_oauth_token(
    provider: OAuthProvider,
    tokens: &OAuthTokens,
) -> Result<OAuthTokens, OAuthError> {
    let refresh = tokens.refresh_token.as_deref().unwrap_or("");
    if refresh.is_empty() {
        return Err(OAuthError::Auth("No refresh token available".into()));
    }

    let body = format!(
        "grant_type=refresh_token&refresh_token={}&client_id={}",
        url_encode(refresh),
        url_encode(provider.client_id()),
    );

    let http_client = reqwest::Client::new();
    let response = http_client
        .post(provider.token_url())
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Accept", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|e| OAuthError::Network(format!("token refresh failed: {e}")))?;

    let v: serde_json::Value = response
        .json()
        .await
        .map_err(|e| OAuthError::Network(format!("token refresh json: {e}")))?;

    let access_token = v
        .get("access_token")
        .and_then(|x| x.as_str())
        .ok_or_else(|| OAuthError::Auth("no access_token in refresh response".into()))?
        .to_string();
    let new_refresh = v
        .get("refresh_token")
        .and_then(|x| x.as_str())
        .filter(|s| !s.is_empty())
        .map(String::from)
        .or_else(|| tokens.refresh_token.clone());
    let expires_in = v.get("expires_in").and_then(|x| x.as_u64()).unwrap_or(3600);
    let expires_at = now_secs() + expires_in;

    Ok(OAuthTokens {
        access_token,
        refresh_token: new_refresh,
        expires_at,
    })
}

fn url_decode(s: &str) -> String {
    let mut out = String::new();
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'+' => {
                out.push(' ');
                i += 1;
            }
            b'%' if i + 2 < b.len() => {
                if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                    out.push(v as char);
                    i += 3;
                } else {
                    out.push('%');
                    i += 1;
                }
            }
            c => {
                out.push(c as char);
                i += 1;
            }
        }
    }
    out
}

fn url_encode(s: &str) -> String {
    use url::form_urlencoded::byte_serialize;
    byte_serialize(s.as_bytes()).collect()
}

fn open_browser(url: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).status().ok();
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .status()
            .ok();
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", url])
            .status()
            .ok();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_client_ids() {
        assert_eq!(
            OAuthProvider::ChatGpt.client_id(),
            "app_EMoamEEZ73f0CkXaXp7hrann"
        );
        assert_eq!(
            OAuthProvider::Xai.client_id(),
            "b1a00492-073a-47ea-816f-4c329264a828"
        );
    }

    #[test]
    fn test_url_decode() {
        assert_eq!(url_decode("hello%20world"), "hello world");
        assert_eq!(url_decode("a+b"), "a b");
        assert_eq!(url_decode("simple"), "simple");
    }

    #[test]
    fn test_url_encode() {
        assert_eq!(url_encode("hello world"), "hello+world");
    }

    #[test]
    fn test_provider_names() {
        assert_eq!(OAuthProvider::ChatGpt.name(), "chatgpt");
        assert_eq!(OAuthProvider::Xai.name(), "grok");
    }
}
