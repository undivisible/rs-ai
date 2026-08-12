//! Claude Code OAuth request compatibility.
//!
//! Anthropic subscription OAuth tokens are not interchangeable with API keys:
//! Claude Code currently expects a versioned billing system block, request
//! headers, and a body checksum. This module keeps that wire contract isolated
//! and fails closed when the request shape is not the one it knows how to
//! transform. The constants must be revalidated when Claude Code changes.

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

/// Claude Code version whose request conventions this implementation speaks.
pub const CLAUDE_CODE_VERSION: &str = "2.1.224";
/// Claude Code billing entrypoint.
pub const CLAUDE_CODE_ENTRYPOINT: &str = "sdk-cli";
const BILLING_PREFIX: &str = "x-anthropic-billing-header: ";
const CCH_PLACEHOLDER: &str = "cch=00000";
const AGENT_SDK_SYSTEM_PROMPT: &str =
    "You are a Claude agent, built on Anthropic's Claude Agent SDK.";
const LEGACY_PI_OAUTH_SYSTEM_PROMPT: &str =
    "You are Claude Code, Anthropic's official CLI for Claude.";

const CCH_SEED: u64 = 0x4d659218e32a3268;
const MASK_64: u64 = u64::MAX;
const PRIME64_1: u64 = 0x9e3779b185ebca87;
const PRIME64_2: u64 = 0xc2b2ae3d27d4eb4f;
const PRIME64_3: u64 = 0x165667b19e3779f9;
const PRIME64_4: u64 = 0x85ebca77c2b2ae63;
const PRIME64_5: u64 = 0x27d4eb2f165667c5;

/// Identity metadata optionally read from Claude Code's local state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaudeCodeIdentity {
    /// Claude Code's installation/device ID.
    pub device_id: String,
    /// Subscription account UUID.
    pub account_uuid: String,
}

/// Return whether a token is an Anthropic subscription OAuth token.
pub fn is_anthropic_oauth_token(token: &str) -> bool {
    token.contains("sk-ant-oat")
}

/// Parse and validate the identity shape used by Claude Code.
pub fn parse_identity(value: &Value) -> Option<ClaudeCodeIdentity> {
    let device_id = value.get("userID")?.as_str()?;
    let account_uuid = value.get("oauthAccount")?.get("accountUuid")?.as_str()?;
    if device_id.len() != 64 || !device_id.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    if !looks_like_uuid(account_uuid) {
        return None;
    }
    Some(ClaudeCodeIdentity {
        device_id: device_id.to_string(),
        account_uuid: account_uuid.to_string(),
    })
}

/// Discover Claude Code identity in memory from environment or `~/.claude.json`.
pub fn discover_identity() -> Option<ClaudeCodeIdentity> {
    let device_id = std::env::var("CLAUDE_CODE_DEVICE_ID").ok();
    let account_uuid = std::env::var("CLAUDE_CODE_ACCOUNT_UUID").ok();
    if let (Some(device_id), Some(account_uuid)) = (device_id, account_uuid) {
        if let Some(identity) = parse_identity(&serde_json::json!({
            "userID": device_id,
            "oauthAccount": {"accountUuid": account_uuid}
        })) {
            return Some(identity);
        }
    }

    let path = if let Some(config_dir) = std::env::var_os("CLAUDE_CONFIG_DIR") {
        std::path::PathBuf::from(config_dir).join(".claude.json")
    } else {
        std::env::var_os("HOME")
            .map(std::path::PathBuf::from)
            .map(|home| home.join(".claude.json"))?
    };
    let text = std::fs::read_to_string(path).ok()?;
    let value = serde_json::from_str(&text).ok()?;
    parse_identity(&value)
}

/// Build the versioned Claude Code billing system block.
pub fn build_billing_header(first_user_prompt: &str) -> String {
    let mut hasher = Sha256::new();
    let selected: String = [4usize, 7, 20]
        .into_iter()
        .map(|index| first_user_prompt.chars().nth(index).unwrap_or('0'))
        .collect();
    hasher.update(format!("59cf53e54c78{selected}{CLAUDE_CODE_VERSION}"));
    let digest = hasher.finalize();
    let fingerprint = digest[..]
        .iter()
        .take(2)
        .fold(String::new(), |mut out, byte| {
            use std::fmt::Write as _;
            write!(&mut out, "{byte:02x}").expect("writing to String cannot fail");
            out
        });
    // The upstream protocol uses the first three hexadecimal characters.
    let fingerprint = fingerprint[..3].to_string();
    format!(
        "{BILLING_PREFIX}cc_version={CLAUDE_CODE_VERSION}.{fingerprint}; cc_entrypoint={CLAUDE_CODE_ENTRYPOINT}; {CCH_PLACEHOLDER};"
    )
}

/// Transform an Anthropic request into the current Claude Code OAuth shape.
pub fn transform_payload(
    mut payload: Value,
    first_user_prompt: &str,
    session_id: Option<&str>,
    identity: Option<&ClaudeCodeIdentity>,
) -> Result<Value, crate::OAuthError> {
    let object = payload.as_object_mut().ok_or_else(|| {
        crate::OAuthError::Auth("Claude Code OAuth request must be a JSON object".into())
    })?;

    let existing_system = object.remove("system").unwrap_or(Value::Null);
    let mut remaining = match existing_system {
        Value::Array(blocks) => blocks,
        Value::String(text) => vec![serde_json::json!({"type": "text", "text": text})],
        Value::Null => Vec::new(),
        _ => {
            return Err(crate::OAuthError::Auth(
                "Claude Code OAuth request has an unsupported system field".into(),
            ));
        }
    };

    let first_text = remaining
        .first()
        .and_then(|block| block.get("text"))
        .and_then(Value::as_str);
    let second_text = remaining
        .get(1)
        .and_then(|block| block.get("text"))
        .and_then(Value::as_str);
    if first_text.is_some_and(|text| text.starts_with(BILLING_PREFIX))
        && second_text == Some(AGENT_SDK_SYSTEM_PROMPT)
    {
        remaining.drain(..2);
    } else if first_text == Some(LEGACY_PI_OAUTH_SYSTEM_PROMPT) {
        remaining.drain(..1);
    }

    let mut system = vec![
        serde_json::json!({"type": "text", "text": build_billing_header(first_user_prompt)}),
        serde_json::json!({"type": "text", "text": AGENT_SDK_SYSTEM_PROMPT}),
    ];
    system.extend(remaining);
    object.insert("system".into(), Value::Array(system));

    if let (Some(identity), Some(session_id)) = (identity, session_id) {
        let metadata = object
            .entry("metadata")
            .or_insert_with(|| Value::Object(Map::new()));
        let metadata = metadata.as_object_mut().ok_or_else(|| {
            crate::OAuthError::Auth("Claude Code OAuth metadata must be an object".into())
        })?;
        metadata.insert(
            "user_id".into(),
            Value::String(
                serde_json::json!({
                    "device_id": identity.device_id,
                    "account_uuid": identity.account_uuid,
                    "session_id": session_id,
                })
                .to_string(),
            ),
        );
    }
    Ok(payload)
}

/// Patch the structure-aware `cch` checksum in a serialized request.
pub fn patch_cch(serialized_body: &str) -> Result<String, crate::OAuthError> {
    let mut parsed: Value = serde_json::from_str(serialized_body).map_err(|e| {
        crate::OAuthError::Auth(format!("Claude Code OAuth request is not valid JSON: {e}"))
    })?;
    let object = parsed.as_object().ok_or_else(|| {
        crate::OAuthError::Auth("Claude Code OAuth request must be a JSON object".into())
    })?;
    let system = object
        .get("system")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            crate::OAuthError::Auth("Claude Code OAuth request is missing system".into())
        })?;
    let billing = system
        .first()
        .and_then(|block| block.get("text"))
        .and_then(Value::as_str)
        .ok_or_else(|| {
            crate::OAuthError::Auth("Claude Code OAuth request is missing billing text".into())
        })?
        .to_string();
    if !billing.starts_with(BILLING_PREFIX) {
        return Err(crate::OAuthError::Auth(
            "Claude Code OAuth request has an invalid billing block".into(),
        ));
    }
    if !billing.contains(CCH_PLACEHOLDER) {
        if billing
            .strip_suffix(';')
            .and_then(|value| value.rsplit_once("; cch="))
            .is_some_and(|(_, value)| {
                value.len() == 5
                    && value
                        .bytes()
                        .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
            })
        {
            return Ok(serialized_body.to_string());
        }
        return Err(crate::OAuthError::Auth(
            "Claude Code OAuth request is missing the cch placeholder".into(),
        ));
    }
    if !object.contains_key("model") || !object.contains_key("max_tokens") {
        return Err(crate::OAuthError::Auth(
            "Claude Code OAuth request is missing model or max_tokens".into(),
        ));
    }

    let mut normalized_value = parsed.clone();
    let normalized_object = normalized_value
        .as_object_mut()
        .expect("object validated above");
    normalized_object.shift_remove("max_tokens");
    normalized_object.insert("model".into(), Value::String(String::new()));
    let normalized = serde_json::to_vec(&normalized_value).map_err(|e| {
        crate::OAuthError::Auth(format!(
            "Claude Code OAuth request normalization failed: {e}"
        ))
    })?;
    let hash = xx_hash64(&normalized, CCH_SEED) & 0xfffff;
    let cch = format!("{hash:05x}");
    let object = parsed.as_object_mut().expect("object validated above");
    let system = object
        .get_mut("system")
        .and_then(Value::as_array_mut)
        .expect("system validated above");
    let billing_block = system
        .first_mut()
        .and_then(Value::as_object_mut)
        .expect("billing block validated above");
    billing_block.insert(
        "text".into(),
        Value::String(billing.replace(CCH_PLACEHOLDER, &format!("cch={cch}"))),
    );
    serde_json::to_string(&parsed).map_err(|e| {
        crate::OAuthError::Auth(format!(
            "Claude Code OAuth request serialization failed: {e}"
        ))
    })
}

/// Return the headers required by the current Claude Code OAuth profile.
pub fn headers(session_id: Option<&str>, client_request_id: &str) -> Vec<(&'static str, String)> {
    let mut headers = vec![
        (
            "user-agent",
            format!("claude-cli/{CLAUDE_CODE_VERSION} (external, {CLAUDE_CODE_ENTRYPOINT})"),
        ),
        ("x-app", "cli".into()),
        (
            "anthropic-beta",
            "claude-code-20250219,oauth-2025-04-20".into(),
        ),
        ("x-client-request-id", client_request_id.into()),
    ];
    if let Some(session_id) = session_id {
        headers.push(("x-claude-code-session-id", session_id.into()));
    }
    headers
}

fn looks_like_uuid(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 36
        && [8, 13, 18, 23].iter().all(|&index| bytes[index] == b'-')
        && matches!(bytes[14], b'1'..=b'5')
        && matches!(bytes[19], b'8'..=b'9' | b'a'..=b'f' | b'A'..=b'F')
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| [8, 13, 18, 23].contains(&index) || byte.is_ascii_hexdigit())
}

fn xx_hash64(bytes: &[u8], seed: u64) -> u64 {
    let mut offset = 0;
    let mut hash;
    if bytes.len() >= 32 {
        let mut v1 = seed.wrapping_add(PRIME64_1).wrapping_add(PRIME64_2);
        let mut v2 = seed.wrapping_add(PRIME64_2);
        let mut v3 = seed;
        let mut v4 = seed.wrapping_sub(PRIME64_1);
        while offset <= bytes.len() - 32 {
            v1 = round(v1, read_u64(bytes, offset));
            v2 = round(v2, read_u64(bytes, offset + 8));
            v3 = round(v3, read_u64(bytes, offset + 16));
            v4 = round(v4, read_u64(bytes, offset + 24));
            offset += 32;
        }
        hash = v1
            .rotate_left(1)
            .wrapping_add(v2.rotate_left(7))
            .wrapping_add(v3.rotate_left(12))
            .wrapping_add(v4.rotate_left(18));
        hash = merge_round(hash, v1);
        hash = merge_round(hash, v2);
        hash = merge_round(hash, v3);
        hash = merge_round(hash, v4);
    } else {
        hash = seed.wrapping_add(PRIME64_5);
    }
    hash = hash.wrapping_add(bytes.len() as u64);
    while offset + 8 <= bytes.len() {
        let lane = round(0, read_u64(bytes, offset));
        hash = (hash ^ lane)
            .rotate_left(27)
            .wrapping_mul(PRIME64_1)
            .wrapping_add(PRIME64_4);
        offset += 8;
    }
    if offset + 4 <= bytes.len() {
        hash ^= (read_u32(bytes, offset) as u64).wrapping_mul(PRIME64_1);
        hash = hash
            .rotate_left(23)
            .wrapping_mul(PRIME64_2)
            .wrapping_add(PRIME64_3);
        offset += 4;
    }
    while offset < bytes.len() {
        hash ^= (bytes[offset] as u64).wrapping_mul(PRIME64_5);
        hash = hash.rotate_left(11).wrapping_mul(PRIME64_1);
        offset += 1;
    }
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(PRIME64_2);
    hash ^= hash >> 29;
    hash = hash.wrapping_mul(PRIME64_3);
    (hash ^ (hash >> 32)) & MASK_64
}

fn round(accumulator: u64, input: u64) -> u64 {
    accumulator
        .wrapping_add(input.wrapping_mul(PRIME64_2))
        .rotate_left(31)
        .wrapping_mul(PRIME64_1)
}

fn merge_round(accumulator: u64, value: u64) -> u64 {
    (accumulator ^ round(0, value))
        .wrapping_mul(PRIME64_1)
        .wrapping_add(PRIME64_4)
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("checked length"),
    )
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(
        bytes[offset..offset + 8]
            .try_into()
            .expect("checked length"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transforms_and_patches_a_request() {
        let payload = serde_json::json!({
            "model": "claude-sonnet-4-6",
            "max_tokens": 128,
            "system": "Be concise.",
            "messages": [{"role": "user", "content": "Hello"}]
        });
        let transformed = transform_payload(payload, "Hello", None, None).unwrap();
        let body = serde_json::to_string(&transformed).unwrap();
        let patched = patch_cch(&body).unwrap();
        assert!(patched.contains("cch="));
        assert!(!patched.contains(CCH_PLACEHOLDER));
        assert!(patched.contains(AGENT_SDK_SYSTEM_PROMPT));
    }

    #[test]
    fn emits_claude_code_beta_and_request_headers() {
        let headers = headers(Some("session"), "request");
        assert!(headers.iter().any(|(name, value)| {
            *name == "anthropic-beta" && value == "claude-code-20250219,oauth-2025-04-20"
        }));
        assert!(headers
            .iter()
            .any(|(name, value)| *name == "x-claude-code-session-id" && value == "session"));
    }

    #[test]
    fn matches_pi_black_checksum_vector_and_only_patches_billing_block() {
        let body = r#"{"model":"claude-opus-5","messages":[{"role":"user","content":"A"}],"max_tokens":64000,"stream":true,"system":[{"type":"text","text":"x-anthropic-billing-header: cc_version=2.1.224.000; cc_entrypoint=sdk-cli; cch=00000;"}]}"#;
        let patched = patch_cch(body).expect("checksum");
        assert!(patched.contains("cch=7ba34"));

        let collision = serde_json::json!({
            "model": "claude-opus-5",
            "messages": [{
                "role": "user",
                "content": "cch=00000",
                "model": "nested-model",
                "max_tokens": 7
            }],
            "max_tokens": 64000,
            "stream": true,
            "system": [
                {"type": "text", "text": "x-anthropic-billing-header: cc_version=2.1.224.000; cc_entrypoint=sdk-cli; cch=00000;"},
                {"type": "text", "text": "fake cch=00000"}
            ],
            "tools": [{"name": "probe", "description": "model max_tokens cch=00000", "input_schema": {"type": "object"}}]
        });
        let patched: Value =
            serde_json::from_str(&patch_cch(&collision.to_string()).unwrap()).unwrap();
        assert!(patched["system"][0]["text"]
            .as_str()
            .unwrap()
            .ends_with(';'));
        assert!(!patched["system"][0]["text"]
            .as_str()
            .unwrap()
            .contains("cch=00000"));
        assert_eq!(patched["system"][1], collision["system"][1]);
        assert_eq!(patched["messages"][0], collision["messages"][0]);
        assert_eq!(patched["tools"], collision["tools"]);
    }

    #[test]
    fn rejects_invalid_identity() {
        assert!(parse_identity(&serde_json::json!({})).is_none());
        assert!(parse_identity(&serde_json::json!({
            "userID": "short",
            "oauthAccount": {"accountUuid": "not-a-uuid"}
        }))
        .is_none());
    }
}
