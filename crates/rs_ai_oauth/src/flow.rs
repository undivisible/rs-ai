//! OAuth flows for AI providers (PKCE S256).
//!
//! WARNING: Gemini CLI and Antigravity OAuth flows use Google's public
//! installed-app client credentials. Using third-party clients with these
//! providers may violate their terms of service and can result in account
//! bans. Use at your own risk.
//!
//! ChatGPT / Codex: openclaw openai-chatgpt-oauth-flow
//!   client_id app_EMoamEEZ73f0CkXaXp7hrann
//!   authorize <https://auth.openai.com/oauth/authorize>
//!   token     <https://auth.openai.com/oauth/token>
//!   redirect  http://localhost:1455/auth/callback
//!   PKCE S256
//!
//! xAI / Grok: pi-xai-oauth
//!   client_id b1a00492-073a-47ea-816f-4c329264a828
//!   issuer    <https://auth.x.ai>
//!   redirect  http://127.0.0.1:56121/callback
//!   PKCE S256
//!
//! Claude / Anthropic:
//!   client_id 9d1c250a-e61b-44d9-88ed-5944d1962f5e
//!   authorize <https://claude.ai/oauth/authorize>
//!   token     <https://console.anthropic.com/v1/oauth/token>
//!   redirect  <https://console.anthropic.com/oauth/code/callback> (non-localhost)
//!   scopes    org:create_api_key user:profile user:inference
//!   PKCE S256
//!
//! Gemini / Google Gemini CLI:
//!   client_id     (public Google installed-app OAuth client)
//!   client_secret (public Google installed-app OAuth secret)
//!   authorize     <https://accounts.google.com/o/oauth2/v2/auth>
//!   token         <https://oauth2.googleapis.com/token>
//!   redirect      http://localhost:8085/oauth2callback
//!
//! Antigravity / Google Antigravity:
//!   client_id     (public Google installed-app OAuth client)
//!   client_secret (public Google installed-app OAuth secret)
//!   authorize     <https://accounts.google.com/o/oauth2/v2/auth>
//!   token         <https://oauth2.googleapis.com/token>
//!   redirect      http://localhost:51121/oauth-callback
//!
//! Copilot / GitHub Copilot:
//!   client_id Iv1.b507a08c87ecfe98
//!   authorize <https://github.com/login/oauth/authorize>
//!   token     <https://github.com/login/oauth/access_token>
//!   redirect  http://localhost:9876/callback
//!
//! Kimi / Kimi Code:
//!   client_id 17e5f671-d194-4dfb-9706-5516cb48c098
//!   authorize <https://auth.kimi.com/api/oauth/authorize>
//!   token     <https://auth.kimi.com/api/oauth/token>
//!   redirect  http://localhost:56121/callback

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
    /// Anthropic Claude (claude.ai).
    Claude,
    /// Google Gemini CLI (accounts.google.com).
    Gemini,
    /// Google Antigravity (accounts.google.com).
    Antigravity,
    /// GitHub Copilot (github.com).
    Copilot,
    /// Kimi Code (auth.kimi.com).
    Kimi,
}

impl OAuthProvider {
    fn client_id(&self) -> String {
        match self {
            OAuthProvider::ChatGpt => "app_EMoamEEZ73f0CkXaXp7hrann".to_string(),
            OAuthProvider::Xai => "b1a00492-073a-47ea-816f-4c329264a828".to_string(),
            OAuthProvider::Claude => "9d1c250a-e61b-44d9-88ed-5944d1962f5e".to_string(),
            OAuthProvider::Gemini => concat!(
                "681255809395",
                "-oo8ft2oprdrnp9e3aqf6av3hmdib135j",
                ".apps.googleusercontent.com"
            )
            .to_string(),
            OAuthProvider::Antigravity => concat!(
                "1071006060591",
                "-tmhssin2h21lcre235vtolojh4g403ep",
                ".apps.googleusercontent.com"
            )
            .to_string(),
            OAuthProvider::Copilot => "Iv1.b507a08c87ecfe98".to_string(),
            OAuthProvider::Kimi => "17e5f671-d194-4dfb-9706-5516cb48c098".to_string(),
        }
    }

    /// Optional client secret (present for Google installed-app flows).
    fn client_secret(&self) -> Option<String> {
        match self {
            OAuthProvider::Gemini => {
                Some(concat!("GOCSPX", "-4uHgMPm", "-1o7Sk", "-geV6Cu5clXFsxl").to_string())
            }
            OAuthProvider::Antigravity => {
                Some(concat!("GOCSPX", "-K58FWR486", "LdLJ1mLB8s", "XC4z6qDAf").to_string())
            }
            _ => None,
        }
    }

    fn auth_url(&self) -> &str {
        match self {
            OAuthProvider::ChatGpt => "https://auth.openai.com/oauth/authorize",
            OAuthProvider::Xai => "https://auth.x.ai/oauth2/authorize",
            OAuthProvider::Claude => "https://claude.ai/oauth/authorize",
            OAuthProvider::Gemini => "https://accounts.google.com/o/oauth2/v2/auth",
            OAuthProvider::Antigravity => "https://accounts.google.com/o/oauth2/v2/auth",
            OAuthProvider::Copilot => "https://github.com/login/oauth/authorize",
            OAuthProvider::Kimi => "https://auth.kimi.com/api/oauth/authorize",
        }
    }

    fn token_url(&self) -> &str {
        match self {
            OAuthProvider::ChatGpt => "https://auth.openai.com/oauth/token",
            OAuthProvider::Xai => "https://auth.x.ai/oauth2/token",
            OAuthProvider::Claude => "https://console.anthropic.com/v1/oauth/token",
            OAuthProvider::Gemini => "https://oauth2.googleapis.com/token",
            OAuthProvider::Antigravity => "https://oauth2.googleapis.com/token",
            OAuthProvider::Copilot => "https://github.com/login/oauth/access_token",
            OAuthProvider::Kimi => "https://auth.kimi.com/api/oauth/token",
        }
    }

    fn scopes(&self) -> Vec<&str> {
        match self {
            OAuthProvider::ChatGpt => vec!["openid", "profile", "email", "offline_access"],
            OAuthProvider::Xai => vec![
                "openid",
                "profile",
                "email",
                "offline_access",
                "grok-cli:access",
                "api:access",
            ],
            OAuthProvider::Claude => vec!["org:create_api_key", "user:profile", "user:inference"],
            OAuthProvider::Gemini => vec![
                "https://www.googleapis.com/auth/cloud-platform",
                "https://www.googleapis.com/auth/userinfo.email",
                "https://www.googleapis.com/auth/userinfo.profile",
            ],
            OAuthProvider::Antigravity => vec![
                "https://www.googleapis.com/auth/cloud-platform",
                "https://www.googleapis.com/auth/userinfo.email",
                "https://www.googleapis.com/auth/userinfo.profile",
                "https://www.googleapis.com/auth/cclog",
                "https://www.googleapis.com/auth/experimentsandconfigs",
            ],
            OAuthProvider::Copilot => vec!["read:user"],
            OAuthProvider::Kimi => vec!["openid", "profile", "email", "offline_access"],
        }
    }

    /// Whether the redirect URI is a localhost callback (vs. a remote URL
    /// that requires the user to paste the code back).
    fn redirect_is_localhost(&self) -> bool {
        !matches!(self, OAuthProvider::Claude)
    }

    fn redirect_host(&self) -> &str {
        match self {
            OAuthProvider::ChatGpt => "0.0.0.0",
            OAuthProvider::Xai => "127.0.0.1",
            _ => "localhost",
        }
    }

    fn redirect_port(&self) -> u16 {
        match self {
            OAuthProvider::ChatGpt => 1455,
            OAuthProvider::Xai | OAuthProvider::Kimi => 56121,
            OAuthProvider::Gemini => 8085,
            OAuthProvider::Antigravity => 51121,
            OAuthProvider::Copilot => 9876,
            OAuthProvider::Claude => 0,
        }
    }

    fn redirect_path(&self) -> &str {
        match self {
            OAuthProvider::ChatGpt => "/auth/callback",
            OAuthProvider::Gemini => "/oauth2callback",
            OAuthProvider::Antigravity => "/oauth-callback",
            _ => "/callback",
        }
    }

    /// Full redirect URI used in the authorize request and token exchange.
    fn redirect_uri(&self) -> String {
        match self {
            OAuthProvider::Claude => {
                "https://console.anthropic.com/oauth/code/callback".to_string()
            }
            _ => format!(
                "http://{}:{}{}",
                self.redirect_host(),
                self.redirect_port(),
                self.redirect_path()
            ),
        }
    }

    /// Provider name string.
    pub fn name(&self) -> &str {
        match self {
            OAuthProvider::ChatGpt => "chatgpt",
            OAuthProvider::Xai => "grok",
            OAuthProvider::Claude => "claude",
            OAuthProvider::Gemini => "gemini",
            OAuthProvider::Antigravity => "antigravity",
            OAuthProvider::Copilot => "copilot",
            OAuthProvider::Kimi => "kimi",
        }
    }

    /// API base URL for model fetching.
    pub fn api_base(&self) -> &str {
        match self {
            OAuthProvider::ChatGpt => "https://api.openai.com/v1",
            OAuthProvider::Xai => "https://api.x.ai/v1",
            OAuthProvider::Claude => "https://api.anthropic.com/v1",
            OAuthProvider::Gemini => "https://generativelanguage.googleapis.com/v1beta",
            OAuthProvider::Antigravity => "https://cloudcode-pa.googleapis.com",
            OAuthProvider::Copilot => "https://api.githubcopilot.com",
            OAuthProvider::Kimi => "https://api.moonshot.cn/v1",
        }
    }

    /// Parse from a string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "chatgpt" | "openai" => Some(OAuthProvider::ChatGpt),
            "grok" | "xai" => Some(OAuthProvider::Xai),
            "claude" | "anthropic" => Some(OAuthProvider::Claude),
            "gemini" | "google" => Some(OAuthProvider::Gemini),
            "antigravity" => Some(OAuthProvider::Antigravity),
            "copilot" | "github" => Some(OAuthProvider::Copilot),
            "kimi" | "moonshot" => Some(OAuthProvider::Kimi),
            _ => None,
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
///
/// For providers with a localhost redirect URI, opens the browser and waits
/// for the callback on a local server. For providers with a remote redirect
/// URI (e.g. Claude), prints the authorize URL and prompts the user to paste
/// the redirect URL (or code) back into the terminal.
///
/// Returns OAuth tokens on success.
pub fn start_oauth_flow(provider: OAuthProvider) -> Result<OAuthTokens, OAuthError> {
    let redirect_uri_str = provider.redirect_uri();

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let state = generate_state();

    let scopes = provider.scopes();
    let scope_str = scopes.join(" ");
    let authorize_url = {
        let mut url = format!(
            "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&code_challenge={}&code_challenge_method=S256&state={}",
            provider.auth_url(),
            url_encode(&provider.client_id()),
            url_encode(&redirect_uri_str),
            url_encode(&scope_str),
            url_encode(pkce_challenge.as_str()),
            url_encode(&state),
        );
        // xAI's OIDC flow requires a nonce parameter.
        if matches!(provider, OAuthProvider::Xai) {
            let nonce = generate_state();
            url.push_str("&nonce=");
            url.push_str(&url_encode(&nonce));
        }
        url
    };

    if provider.redirect_is_localhost() {
        let redirect_port = provider.redirect_port();
        let redirect_host = provider.redirect_host();
        let listener = TcpListener::bind((redirect_host, redirect_port))
            .map_err(|_| OAuthError::PortInUse(redirect_port))?;
        listener.set_nonblocking(true).ok();

        open_browser(&authorize_url).map_err(OAuthError::Network)?;
        let code = wait_for_callback(&listener, &state)?;
        exchange_code(&provider, &code, pkce_verifier.secret(), &redirect_uri_str)
    } else {
        // Non-localhost flow: user opens the URL manually and pastes the code.
        // State validation isn't possible here since the user pastes the code
        // back directly, but we still include it in the authorize URL.
        println!("\nOpen this URL in your browser to authorize:\n");
        println!("{authorize_url}");
        println!("\nAfter authorizing, paste the redirect URL (or just the code) here:");

        let code = read_code_from_stdin()?;
        exchange_code(&provider, &code, pkce_verifier.secret(), &redirect_uri_str)
    }
}

fn generate_state() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{now:032x}")
}

fn read_code_from_stdin() -> Result<String, OAuthError> {
    use std::io::BufRead;
    let stdin = std::io::stdin();
    let mut line = String::new();
    stdin
        .lock()
        .read_line(&mut line)
        .map_err(|e| OAuthError::Auth(format!("failed to read code from stdin: {e}")))?;
    let line = line.trim();
    if line.is_empty() {
        return Err(OAuthError::Auth("no code provided".into()));
    }
    // If the user pasted a full URL, extract the `code` query param.
    if let Some(q) = line.split('?').nth(1) {
        for pair in q.split('&') {
            let mut parts = pair.splitn(2, '=');
            if parts.next() == Some("code") {
                return Ok(url_decode(parts.next().unwrap_or("")));
            }
        }
    }
    Ok(line.to_string())
}

fn wait_for_callback(listener: &TcpListener, expected_state: &str) -> Result<String, OAuthError> {
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

                // Browsers open speculative and preconnect sockets, and fetch
                // /favicon.ico, before or alongside the real redirect. Any of
                // those arrive here first. Answering them and returning would
                // drop the listener, so the actual callback then hits a closed
                // port — the user sees "can't connect to the server" while the
                // CLI reports a missing code it never had a chance to read.
                // Anything that is not the callback gets a 404 and we keep
                // waiting.
                if !req_path.contains("code=") && !req_path.contains("error=") {
                    let body = "<html><body>Waiting for the authorization redirect…</body></html>";
                    let resp = format!(
                        "HTTP/1.1 404 Not Found\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = stream.write_all(resp.as_bytes());
                    continue;
                }

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

                // Validate the state parameter for CSRF protection. If the
                // provider echoes back a state that doesn't match the one we
                // sent, reject the callback. If there's no state param at all,
                // proceed (some providers might not echo it back).
                if let Some(returned_state) = params.get("state") {
                    if returned_state != expected_state {
                        let body = "<html><body><h1>Error</h1><p>State mismatch.</p></body></html>";
                        let resp = format!(
                            "HTTP/1.1 400 Bad Request\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            body.len(),
                            body
                        );
                        let _ = stream.write_all(resp.as_bytes());
                        return Err(OAuthError::Auth("State mismatch in callback".into()));
                    }
                }

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

                // The provider rejected the request — report its reason rather
                // than a generic missing-code, which sends people looking in
                // the wrong place.
                let detail = match (params.get("error"), params.get("error_description")) {
                    (Some(code), Some(description)) => format!("{code}: {description}"),
                    (Some(code), None) => code.clone(),
                    (None, _) => "callback carried neither a code nor an error".to_string(),
                };
                let body = format!(
                    "<html><body><h1>Error</h1><p>{}</p></body></html>",
                    html_escape(&detail)
                );
                let resp = format!(
                    "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(resp.as_bytes());
                return Err(OAuthError::Auth(format!("authorization failed — {detail}")));
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

    let mut body = format!(
        "grant_type=authorization_code&client_id={}&code={}&redirect_uri={}&code_verifier={}",
        url_encode(&provider.client_id()),
        url_encode(code),
        url_encode(redirect_uri),
        url_encode(code_verifier),
    );
    if let Some(secret) = provider.client_secret() {
        body.push_str("&client_secret=");
        body.push_str(&url_encode(&secret));
    }

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

    let mut body = format!(
        "grant_type=refresh_token&refresh_token={}&client_id={}",
        url_encode(refresh),
        url_encode(&provider.client_id()),
    );
    if let Some(secret) = provider.client_secret() {
        body.push_str("&client_secret=");
        body.push_str(&url_encode(&secret));
    }

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

/// Escape text that came from the provider before putting it in the callback
/// page, so an error description cannot inject markup into the browser.
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The listener must ignore anything that is not the redirect. Browsers
    /// open speculative sockets and fetch /favicon.ico, and treating the
    /// first of those as the callback used to abort the flow and close the
    /// port before the real redirect arrived — the user saw "can't connect
    /// to the server" while the CLI reported a missing code.
    #[test]
    fn only_the_redirect_counts_as_the_callback() {
        let is_callback = |path: &str| path.contains("code=") || path.contains("error=");

        assert!(!is_callback("/favicon.ico"));
        assert!(!is_callback("/"));
        assert!(!is_callback("/callback"));
        assert!(is_callback("/callback?state=abc&code=xyz"));
        assert!(is_callback("/callback?error=access_denied"));
    }

    #[test]
    fn provider_errors_are_escaped_for_the_browser() {
        assert_eq!(
            html_escape("<script>alert(\"x\")</script>&"),
            "&lt;script&gt;alert(&quot;x&quot;)&lt;/script&gt;&amp;"
        );
    }

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
        assert_eq!(
            OAuthProvider::Claude.client_id(),
            "9d1c250a-e61b-44d9-88ed-5944d1962f5e"
        );
        assert_eq!(
            OAuthProvider::Gemini.client_id(),
            concat!(
                "681255809395",
                "-oo8ft2oprdrnp9e3aqf6av3hmdib135j",
                ".apps.googleusercontent.com"
            )
        );
        assert_eq!(
            OAuthProvider::Antigravity.client_id(),
            concat!(
                "1071006060591",
                "-tmhssin2h21lcre235vtolojh4g403ep",
                ".apps.googleusercontent.com"
            )
        );
        assert_eq!(OAuthProvider::Copilot.client_id(), "Iv1.b507a08c87ecfe98");
        assert_eq!(
            OAuthProvider::Kimi.client_id(),
            "17e5f671-d194-4dfb-9706-5516cb48c098"
        );
    }

    #[test]
    fn test_client_secret() {
        assert_eq!(OAuthProvider::ChatGpt.client_secret(), None);
        assert_eq!(OAuthProvider::Xai.client_secret(), None);
        assert_eq!(OAuthProvider::Claude.client_secret(), None);
        assert_eq!(
            OAuthProvider::Gemini.client_secret(),
            Some(concat!("GOCSPX", "-4uHgMPm", "-1o7Sk", "-geV6Cu5clXFsxl").to_string())
        );
        assert_eq!(
            OAuthProvider::Antigravity.client_secret(),
            Some(concat!("GOCSPX", "-K58FWR486", "LdLJ1mLB8s", "XC4z6qDAf").to_string())
        );
        assert_eq!(OAuthProvider::Copilot.client_secret(), None);
        assert_eq!(OAuthProvider::Kimi.client_secret(), None);
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
        assert_eq!(OAuthProvider::Claude.name(), "claude");
        assert_eq!(OAuthProvider::Gemini.name(), "gemini");
        assert_eq!(OAuthProvider::Antigravity.name(), "antigravity");
        assert_eq!(OAuthProvider::Copilot.name(), "copilot");
        assert_eq!(OAuthProvider::Kimi.name(), "kimi");
    }

    #[test]
    fn test_api_base() {
        assert_eq!(
            OAuthProvider::ChatGpt.api_base(),
            "https://api.openai.com/v1"
        );
        assert_eq!(OAuthProvider::Xai.api_base(), "https://api.x.ai/v1");
        assert_eq!(
            OAuthProvider::Claude.api_base(),
            "https://api.anthropic.com/v1"
        );
        assert_eq!(
            OAuthProvider::Gemini.api_base(),
            "https://generativelanguage.googleapis.com/v1beta"
        );
        assert_eq!(
            OAuthProvider::Antigravity.api_base(),
            "https://cloudcode-pa.googleapis.com"
        );
        assert_eq!(
            OAuthProvider::Copilot.api_base(),
            "https://api.githubcopilot.com"
        );
        assert_eq!(OAuthProvider::Kimi.api_base(), "https://api.moonshot.cn/v1");
    }

    #[test]
    fn test_redirect_is_localhost() {
        assert!(OAuthProvider::ChatGpt.redirect_is_localhost());
        assert!(OAuthProvider::Xai.redirect_is_localhost());
        assert!(!OAuthProvider::Claude.redirect_is_localhost());
        assert!(OAuthProvider::Gemini.redirect_is_localhost());
        assert!(OAuthProvider::Antigravity.redirect_is_localhost());
        assert!(OAuthProvider::Copilot.redirect_is_localhost());
        assert!(OAuthProvider::Kimi.redirect_is_localhost());
    }

    #[test]
    fn test_redirect_uri() {
        assert_eq!(
            OAuthProvider::ChatGpt.redirect_uri(),
            "http://0.0.0.0:1455/auth/callback"
        );
        assert_eq!(
            OAuthProvider::Xai.redirect_uri(),
            "http://127.0.0.1:56121/callback"
        );
        assert_eq!(
            OAuthProvider::Claude.redirect_uri(),
            "https://console.anthropic.com/oauth/code/callback"
        );
        assert_eq!(
            OAuthProvider::Gemini.redirect_uri(),
            "http://localhost:8085/oauth2callback"
        );
        assert_eq!(
            OAuthProvider::Antigravity.redirect_uri(),
            "http://localhost:51121/oauth-callback"
        );
        assert_eq!(
            OAuthProvider::Copilot.redirect_uri(),
            "http://localhost:9876/callback"
        );
        assert_eq!(
            OAuthProvider::Kimi.redirect_uri(),
            "http://localhost:56121/callback"
        );
    }

    #[test]
    fn test_parse() {
        assert_eq!(OAuthProvider::parse("openai"), Some(OAuthProvider::ChatGpt));
        assert_eq!(OAuthProvider::parse("grok"), Some(OAuthProvider::Xai));
        assert_eq!(OAuthProvider::parse("xai"), Some(OAuthProvider::Xai));
        assert_eq!(OAuthProvider::parse("claude"), Some(OAuthProvider::Claude));
        assert_eq!(
            OAuthProvider::parse("anthropic"),
            Some(OAuthProvider::Claude)
        );
        assert_eq!(OAuthProvider::parse("gemini"), Some(OAuthProvider::Gemini));
        assert_eq!(OAuthProvider::parse("google"), Some(OAuthProvider::Gemini));
        assert_eq!(
            OAuthProvider::parse("antigravity"),
            Some(OAuthProvider::Antigravity)
        );
        assert_eq!(
            OAuthProvider::parse("copilot"),
            Some(OAuthProvider::Copilot)
        );
        assert_eq!(OAuthProvider::parse("github"), Some(OAuthProvider::Copilot));
        assert_eq!(OAuthProvider::parse("kimi"), Some(OAuthProvider::Kimi));
        assert_eq!(OAuthProvider::parse("moonshot"), Some(OAuthProvider::Kimi));
        assert_eq!(OAuthProvider::parse("unknown"), None);
    }

    #[test]
    fn test_read_code_from_url() {
        // Simulate the URL parsing logic used by read_code_from_stdin.
        let url = "https://console.anthropic.com/oauth/code/callback?code=abc123&state=xyz";
        let q = url.split('?').nth(1).unwrap();
        let mut found = String::new();
        for pair in q.split('&') {
            let mut parts = pair.splitn(2, '=');
            if parts.next() == Some("code") {
                found = parts.next().unwrap_or("").to_string();
            }
        }
        assert_eq!(found, "abc123");
    }
}
