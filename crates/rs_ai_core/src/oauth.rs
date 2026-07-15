//! OAuth flows for ChatGPT and xAI/Grok (PKCE S256).
//! Based on openclaw openai-chatgpt-oauth-flow and pi-xai-oauth.

use serde::{Deserialize, Serialize};
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
    pub fn name(&self) -> &str {
        match self {
            OAuthProvider::ChatGpt => "chatgpt",
            OAuthProvider::Xai => "grok",
        }
    }

    pub fn scopes(&self) -> &str {
        match self {
            OAuthProvider::ChatGpt => "openid profile email offline_access",
            OAuthProvider::Xai => {
                "openid profile email offline_access grok-cli:access api:access"
            }
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
#[derive(Debug)]
pub enum OAuthError {
    Network(String),
    Auth(String),
    Timeout,
    PortInUse(u16),
}

/// PKCE helper: generate code verifier + challenge (S256).
fn pkce_pair() -> (String, String) {
    use sha2::{Digest, Sha256};
    let mut raw = [0u8; 32];
    getrandom::getrandom(&mut raw).expect("entropy");
    let verifier = base64url(&raw);
    let challenge = {
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        base64url(&hasher.finalize())
    };
    (verifier, challenge)
}

fn base64url(bytes: &[u8]) -> String {
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut s = String::new();
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        s.push(T[((triple >> 18) & 63) as usize] as char);
        s.push(T[((triple >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            s.push(T[((triple >> 6) & 63) as usize] as char);
        }
        if chunk.len() > 2 {
            s.push(T[(triple & 63) as usize] as char);
        }
    }
    s
}

fn urlenc(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn html_ok(body: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

/// Start the OAuth flow for the given provider.
/// Opens the browser and waits for the callback on a localhost server.
/// Returns OAuth tokens on success.
pub fn start_oauth_flow(provider: OAuthProvider) -> Result<OAuthTokens, OAuthError> {
    match provider {
        OAuthProvider::ChatGpt => oauth_flow_chatgpt(),
        OAuthProvider::Xai => oauth_flow_xai(),
    }
}

fn oauth_flow_chatgpt() -> Result<OAuthTokens, OAuthError> {
    const CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
    const AUTHORIZE: &str = "https://auth.openai.com/oauth/authorize";
    const TOKEN: &str = "https://auth.openai.com/oauth/token";
    const SCOPES: &str = "openid profile email offline_access";
    const REDIRECT_PORT: u16 = 1455;
    const REDIRECT_PATH: &str = "/auth/callback";

    let (verifier, challenge) = pkce_pair();
    let redirect = format!("http://localhost:{REDIRECT_PORT}{REDIRECT_PATH}");

    let listener = TcpListener::bind(("0.0.0.0", REDIRECT_PORT))
        .map_err(|_| OAuthError::PortInUse(REDIRECT_PORT))?;
    listener.set_nonblocking(true).ok();

    let url = format!(
        "{AUTHORIZE}?response_type=code&client_id={cid}&redirect_uri={redir}&scope={s}&code_challenge={ch}&code_challenge_method=S256&state={st}&originator=codex_cli_rs",
        cid = urlenc(CLIENT_ID),
        redir = urlenc(&redirect),
        s = urlenc(SCOPES),
        ch = urlenc(&challenge),
        st = urlenc("rs_ai_oauth"),
    );

    open_browser(&url).map_err(OAuthError::Network)?;

    let code = wait_for_callback(&listener, REDIRECT_PATH)?;
    exchange_code(TOKEN, CLIENT_ID, &redirect, &verifier, &code)
}

fn oauth_flow_xai() -> Result<OAuthTokens, OAuthError> {
    const CLIENT_ID: &str = "b1a00492-073a-47ea-816f-4c329264a828";
    const DISCOVERY: &str = "https://auth.x.ai/.well-known/openid-configuration";
    const SCOPES: &str = "openid profile email offline_access grok-cli:access api:access";
    const REDIRECT_HOST: &str = "127.0.0.1";
    const REDIRECT_PORT: u16 = 56121;
    const REDIRECT_PATH: &str = "/callback";

    let (auth_ep, token_ep) = discover_xai(DISCOVERY)?;

    let (verifier, challenge) = pkce_pair();

    let listener = TcpListener::bind((REDIRECT_HOST, REDIRECT_PORT))
        .or_else(|_| TcpListener::bind((REDIRECT_HOST, 0u16)))
        .map_err(|_| OAuthError::PortInUse(REDIRECT_PORT))?;
    let bound_port = listener
        .local_addr()
        .map(|a| a.port())
        .unwrap_or(REDIRECT_PORT);
    let redirect = format!("http://{REDIRECT_HOST}:{bound_port}{REDIRECT_PATH}");
    listener.set_nonblocking(true).ok();

    let url = format!(
        "{auth}?response_type=code&client_id={cid}&redirect_uri={redir}&scope={s}&code_challenge={ch}&code_challenge_method=S256&state={st}",
        auth = auth_ep,
        cid = urlenc(CLIENT_ID),
        redir = urlenc(&redirect),
        s = urlenc(SCOPES),
        ch = urlenc(&challenge),
        st = urlenc("rs_ai_oauth"),
    );

    open_browser(&url).map_err(OAuthError::Network)?;

    let code = wait_for_callback(&listener, REDIRECT_PATH)?;
    exchange_code(&token_ep, CLIENT_ID, &redirect, &verifier, &code)
}

fn discover_xai(discovery_url: &str) -> Result<(String, String), OAuthError> {
    let out = std::process::Command::new("curl")
        .args(["-fsSL", discovery_url])
        .output()
        .map_err(|e| OAuthError::Network(format!("xAI discovery failed: {e}")))?;
    if !out.status.success() {
        return Err(OAuthError::Network("xAI discovery failed".into()));
    }
    let v: serde_json::Value = serde_json::from_slice(&out.stdout)
        .map_err(|e| OAuthError::Network(format!("discovery json: {e}")))?;
    let auth = v
        .get("authorization_endpoint")
        .and_then(|x| x.as_str())
        .ok_or_else(|| OAuthError::Network("missing authorization_endpoint".into()))?
        .to_string();
    let token = v
        .get("token_endpoint")
        .and_then(|x| x.as_str())
        .ok_or_else(|| OAuthError::Network("missing token_endpoint".into()))?
        .to_string();
    Ok((auth, token))
}

fn wait_for_callback(listener: &TcpListener, _path: &str) -> Result<String, OAuthError> {
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(180);

    while start.elapsed() < timeout {
        match listener.accept() {
            Ok((mut stream, _)) => {
                let mut buf = [0u8; 8192];
                let n = stream.read(&mut buf).unwrap_or(0);
                let req = String::from_utf8_lossy(&buf[..n]);
                let line = req.lines().next().unwrap_or("");
                let req_path = line.split_whitespace().nth(1).unwrap_or("");

                if !req_path.contains('?') {
                    let body = "Not found";
                    let _ = stream.write_all(
                        format!(
                            "HTTP/1.1 404 Not Found\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                            body.len()
                        )
                        .as_bytes(),
                    );
                    continue;
                }

                let q = req_path.split('?').nth(1).unwrap_or("");
                let mut code = None;
                for pair in q.split('&') {
                    let mut it = pair.splitn(2, '=');
                    let k = it.next().unwrap_or("");
                    let v = it.next().unwrap_or("");
                    if k == "code" {
                        code = Some(url_decode(v));
                    }
                }

                if let Some(code) = code {
                    let body = "<html><body style='font-family:sans-serif;background:#111;color:#eee;padding:40px'><h1>Connected to rs_ai</h1><p>You can close this tab.</p></body></html>";
                    let _ = stream.write_all(html_ok(body).as_bytes());
                    return Ok(code);
                }

                let body = "<html><body><h1>Missing code parameter</h1></body></html>";
                let _ = stream.write_all(html_ok(body).as_bytes());
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
    token_url: &str,
    client_id: &str,
    redirect_uri: &str,
    code_verifier: &str,
    code: &str,
) -> Result<OAuthTokens, OAuthError> {
    let body = format!(
        "grant_type=authorization_code&client_id={}&code={}&redirect_uri={}&code_verifier={}",
        urlenc(client_id),
        urlenc(code),
        urlenc(redirect_uri),
        urlenc(code_verifier),
    );

    let out = std::process::Command::new("curl")
        .args([
            "-fsSL",
            "-X",
            "POST",
            token_url,
            "-H",
            "Content-Type: application/x-www-form-urlencoded",
            "-H",
            "Accept: application/json",
            "--data",
            &body,
        ])
        .output()
        .map_err(|e| OAuthError::Network(format!("token exchange failed: {e}")))?;

    if !out.status.success() {
        return Err(OAuthError::Auth(
            String::from_utf8_lossy(&out.stderr).to_string(),
        ));
    }

    let v: serde_json::Value = serde_json::from_slice(&out.stdout)
        .map_err(|e| OAuthError::Network(format!("token json: {e}")))?;

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

/// Refresh an expired token using the refresh token.
pub async fn refresh_oauth_token(
    provider: OAuthProvider,
    tokens: &OAuthTokens,
) -> Result<OAuthTokens, OAuthError> {
    let refresh = tokens.refresh_token.as_deref().unwrap_or("");
    if refresh.is_empty() {
        return Err(OAuthError::Auth("No refresh token available".into()));
    }

    let token_endpoint = match provider {
        OAuthProvider::ChatGpt => "https://auth.openai.com/oauth/token",
        OAuthProvider::Xai => "https://auth.x.ai/oauth2/token",
    };

    let client_id = match provider {
        OAuthProvider::ChatGpt => "app_EMoamEEZ73f0CkXaXp7hrann",
        OAuthProvider::Xai => "b1a00492-073a-47ea-816f-4c329264a828",
    };

    let body = format!(
        "grant_type=refresh_token&refresh_token={}&client_id={}",
        urlenc(refresh),
        urlenc(client_id),
    );

    let client = reqwest::Client::new();
    let response = client
        .post(token_endpoint)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Accept", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|e| OAuthError::Network(format!("token refresh failed: {e}")))?;

    if !response.status().is_success() {
        return Err(OAuthError::Auth("Token refresh failed".into()));
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_pair_generates_valid_verifier_and_challenge() {
        let (verifier, challenge) = pkce_pair();
        assert!(!verifier.is_empty());
        assert!(!challenge.is_empty());
        assert_ne!(verifier, challenge);
        // Both should be base64url (no + / or =)
        assert!(!verifier.contains('+'));
        assert!(!verifier.contains('/'));
        assert!(!verifier.contains('='));
    }

    #[test]
    fn base64url_encodes_correctly() {
        let input = b"hello";
        let result = base64url(input);
        assert!(!result.is_empty());
        assert!(!result.contains('+'));
        assert!(!result.contains('/'));
        assert!(!result.contains('='));
    }

    #[test]
    fn url_decode_handles_encoded_string() {
        assert_eq!(url_decode("hello%20world"), "hello world");
        assert_eq!(url_decode("a+b"), "a b");
        assert_eq!(url_decode("simple"), "simple");
    }

    #[test]
    fn provider_names_are_correct() {
        assert_eq!(OAuthProvider::ChatGpt.name(), "chatgpt");
        assert_eq!(OAuthProvider::Xai.name(), "grok");
    }
}
