//! One credential store shared by every tool built on this crate.
//!
//! Tools used to each invent their own location — telekinesis wrote
//! `~/.telekinesis/<provider>_token.json`, apollo read Claude Code's
//! `~/.claude/.credentials.json` — so logging in with one left the other
//! logged out. Everything now writes to a single canonical directory and
//! reads the older locations as a fallback, so an existing login keeps
//! working and a new one is immediately visible to every tool.

use std::path::{Path, PathBuf};

use crate::flow::{OAuthProvider, OAuthTokens};

/// Canonical directory: `~/.config/rs_ai/credentials`.
///
/// Honours `RS_AI_CREDENTIALS_DIR` so tests and sandboxes can redirect it
/// without touching the real user's tokens.
pub fn credentials_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("RS_AI_CREDENTIALS_DIR") {
        if !dir.is_empty() {
            return Some(PathBuf::from(dir));
        }
    }
    let home = std::env::var_os("HOME").filter(|h| !h.is_empty())?;
    Some(PathBuf::from(home).join(".config/rs_ai/credentials"))
}

/// Canonical file for one provider.
pub fn credentials_path(provider: &OAuthProvider) -> Option<PathBuf> {
    Some(credentials_dir()?.join(format!("{}.json", provider.name())))
}

/// Locations written by tools that predate this store.
///
/// Read-only: a token found here is used but never written back, so the
/// original tool keeps working exactly as before.
fn legacy_paths(provider: &OAuthProvider) -> Vec<PathBuf> {
    let Some(home) = std::env::var_os("HOME").filter(|h| !h.is_empty()) else {
        return Vec::new();
    };
    let home = PathBuf::from(home);
    let mut paths = vec![
        home.join(format!(".telekinesis/{}_token.json", provider.name())),
        home.join(format!(".rs_ai/{}_token.json", provider.name())),
    ];
    if matches!(provider, OAuthProvider::Claude) {
        paths.push(home.join(".claude/.credentials.json"));
    }
    paths
}

/// Write a token so only the owner can read it.
///
/// The mode is set at creation, so the file is never briefly world-readable
/// — these are live provider credentials.
fn write_private(path: &Path, contents: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    #[cfg(unix)]
    {
        use std::io::Write as _;
        use std::os::unix::fs::OpenOptionsExt;

        // Write a fresh temp file and rename over the target.
        //
        // `create_new` refuses to follow a symlink someone planted at the temp
        // path, and `mode(0o600)` applies because the file is genuinely new —
        // on an existing file that flag is silently ignored, which is why the
        // previous version's guarantee came from a path-based chmod that
        // followed links to whatever they pointed at.
        //
        // The rename also makes the write atomic: a crash can no longer leave
        // a truncated credential file that reads as "logged out".
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let temp = parent.join(format!(
            ".{}.{}.tmp",
            path.file_name().and_then(|n| n.to_str()).unwrap_or("cred"),
            std::process::id()
        ));
        let _ = std::fs::remove_file(&temp);
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temp)?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&temp, path)?;
        Ok(())
    }
    #[cfg(not(unix))]
    {
        std::fs::write(path, contents)
    }
}

/// Save tokens to the shared store. Returns the path written.
pub fn save(provider: &OAuthProvider, tokens: &OAuthTokens) -> std::io::Result<PathBuf> {
    let path = credentials_path(provider).ok_or_else(|| {
        std::io::Error::other("cannot locate a home directory for the credential store")
    })?;
    let body = serde_json::to_string_pretty(tokens).map_err(std::io::Error::other)?;
    write_private(&path, &body)?;
    Ok(path)
}

/// Load tokens for a provider: the shared store first, then legacy locations.
pub fn load(provider: &OAuthProvider) -> Option<OAuthTokens> {
    let mut candidates = Vec::new();
    if let Some(path) = credentials_path(provider) {
        candidates.push(path);
    }
    candidates.extend(legacy_paths(provider));

    for path in candidates {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        if let Ok(tokens) = serde_json::from_str::<OAuthTokens>(&text) {
            return Some(tokens);
        }
        if let Some(tokens) = parse_foreign(&text) {
            return Some(tokens);
        }
    }
    None
}

/// Read a token file written by another tool in its own shape.
///
/// Claude Code stores `{"claudeAiOauth":{"accessToken","refreshToken",
/// "expiresAt"}}` with the expiry in MILLISECONDS, not the flat
/// seconds-based shape this crate writes. Without this the Claude fallback
/// silently never matched, and an existing Claude Code login looked like no
/// login at all.
fn parse_foreign(text: &str) -> Option<OAuthTokens> {
    let value: serde_json::Value = serde_json::from_str(text).ok()?;
    // Accept the nested wrapper or a bare camelCase object.
    let node = value.get("claudeAiOauth").unwrap_or(&value);

    let access_token = node
        .get("accessToken")
        .or_else(|| node.get("access_token"))?
        .as_str()?
        .to_string();
    if access_token.is_empty() {
        return None;
    }
    let refresh_token = node
        .get("refreshToken")
        .or_else(|| node.get("refresh_token"))
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let expires_at = node
        .get("expiresAt")
        .or_else(|| node.get("expires_at"))
        .and_then(serde_json::Value::as_u64)
        .map(normalise_expiry)
        .unwrap_or(0);

    Some(OAuthTokens {
        access_token,
        refresh_token,
        expires_at,
    })
}

/// Convert a possibly-millisecond expiry to seconds.
///
/// A seconds timestamp will not exceed 10^12 until the year 33658, so a value
/// above that came from a tool that stores milliseconds. Getting this wrong
/// makes a live token look decades expired.
fn normalise_expiry(value: u64) -> u64 {
    const MILLISECOND_THRESHOLD: u64 = 1_000_000_000_000;
    if value >= MILLISECOND_THRESHOLD {
        value / 1000
    } else {
        value
    }
}

/// True when the token is missing or already expired.
///
/// `expires_at` is a unix timestamp; a 60s margin avoids handing out a token
/// that dies mid-request.
pub fn is_expired(tokens: &OAuthTokens) -> bool {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    tokens.expires_at != 0 && tokens.expires_at <= now.saturating_add(60)
}

/// Every provider that currently has a usable token, for "am I logged in?".
pub fn logged_in_providers() -> Vec<OAuthProvider> {
    OAuthProvider::all()
        .iter()
        .copied()
        .filter(|provider| load(provider).is_some())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RS_AI_CREDENTIALS_DIR is process-global, so the tests that set it must
    /// not run concurrently — otherwise one clears it while another is mid-run.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn tokens() -> OAuthTokens {
        OAuthTokens {
            access_token: "at".into(),
            refresh_token: Some("rt".into()),
            expires_at: 0,
        }
    }

    /// Uses RS_AI_CREDENTIALS_DIR so the real user's tokens are never touched.
    #[test]
    fn a_saved_token_round_trips_and_is_owner_only() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = std::env::temp_dir().join(format!("rsai-cred-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        // SAFETY: single-threaded test process.
        unsafe { std::env::set_var("RS_AI_CREDENTIALS_DIR", &dir) };

        let path = save(&OAuthProvider::Xai, &tokens()).expect("save");
        assert!(path.starts_with(&dir));

        let loaded = load(&OAuthProvider::Xai).expect("load");
        assert_eq!(loaded.access_token, "at");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode();
            assert_eq!(
                mode & 0o777,
                0o600,
                "credentials must not be readable by others"
            );
        }

        unsafe { std::env::remove_var("RS_AI_CREDENTIALS_DIR") };
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_missing_provider_loads_as_none() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = std::env::temp_dir().join(format!("rsai-cred-empty-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        unsafe { std::env::set_var("RS_AI_CREDENTIALS_DIR", &dir) };
        // Kimi has no legacy location, so this cannot be satisfied by a
        // pre-existing file in the developer's home directory.
        assert!(load(&OAuthProvider::Kimi).is_none());
        unsafe { std::env::remove_var("RS_AI_CREDENTIALS_DIR") };
    }

    #[test]
    fn a_claude_code_credentials_file_is_understood() {
        // The exact shape Claude Code writes: nested, camelCase, and an
        // expiry in milliseconds.
        let text = r#"{"claudeAiOauth":{"accessToken":"tok","refreshToken":"ref","expiresAt":2000000000000}}"#;
        let parsed = parse_foreign(text).expect("claude code shape must parse");
        assert_eq!(parsed.access_token, "tok");
        assert_eq!(parsed.refresh_token.as_deref(), Some("ref"));
        // Milliseconds converted to seconds, not carried through as-is.
        assert_eq!(parsed.expires_at, 2_000_000_000);
        assert!(
            !is_expired(&parsed),
            "a token valid until 2033 is not expired"
        );
    }

    #[test]
    fn a_seconds_expiry_is_left_alone() {
        assert_eq!(normalise_expiry(2_000_000_000), 2_000_000_000);
        assert_eq!(normalise_expiry(2_000_000_000_000), 2_000_000_000);
        assert_eq!(normalise_expiry(0), 0);
    }

    /// A planted symlink must not receive the token, and must not have its
    /// target chmodded. Found by probe, not by a test — hence this one.
    #[cfg(unix)]
    #[test]
    fn a_planted_symlink_does_not_capture_the_token() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = std::env::temp_dir().join(format!("rsai-symlink-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let victim = dir.join("victim.txt");
        std::fs::write(&victim, "untouched").unwrap();
        // Ask for the real filename — Xai's provider name is "grok", so a
        // hardcoded "xai.json" would plant the link where nothing writes.
        let target = dir.join(format!("{}.json", OAuthProvider::Xai.name()));
        std::os::unix::fs::symlink(&victim, &target).unwrap();

        unsafe { std::env::set_var("RS_AI_CREDENTIALS_DIR", &dir) };
        save(&OAuthProvider::Xai, &tokens()).expect("save");
        unsafe { std::env::remove_var("RS_AI_CREDENTIALS_DIR") };

        assert_eq!(
            std::fs::read_to_string(&victim).unwrap(),
            "untouched",
            "the symlink target must not receive the credential"
        );
        assert!(
            !std::fs::symlink_metadata(&target).unwrap().is_symlink(),
            "the symlink must have been replaced, not followed"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn expiry_leaves_a_margin() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut t = tokens();

        t.expires_at = 0;
        assert!(!is_expired(&t), "0 means no known expiry");

        t.expires_at = now + 30;
        assert!(is_expired(&t), "inside the margin counts as expired");

        t.expires_at = now + 3600;
        assert!(!is_expired(&t));
    }
}
