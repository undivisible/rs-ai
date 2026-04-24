//! Gemini Live API — bidirectional WebSocket streaming for voice, video, and text.
//!
//! The Live API enables low-latency, real-time interactions by streaming audio,
//! video frames, and text in both directions over a persistent WebSocket.
//!
//! Reference: <https://ai.google.dev/gemini-api/docs/live-api>

use base64::Engine;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio_tungstenite::{
    connect_async_tls_with_config,
    tungstenite::{client::IntoClientRequest, Message},
};

const LIVE_URL: &str =
    "wss://generativelanguage.googleapis.com/ws/google.ai.generativelanguage.v1beta.GenerativeService.BidiGenerateContent";

// ── Error ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum LiveError {
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("URL parse error: {0}")]
    Url(#[from] url::ParseError),
    #[error("API error: {message}")]
    ApiError { message: String },
    #[error("Session closed")]
    SessionClosed,
}

pub type LiveResult<T> = Result<T, LiveError>;

// ── Voice / audio options ─────────────────────────────────────────────────────

/// HD voice IDs available in Gemini Live (30 voices across 24 languages).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LiveVoice {
    #[default]
    Aoede,
    Charon,
    Fenrir,
    Kore,
    Puck,
    Custom(String),
}

impl LiveVoice {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Aoede => "Aoede",
            Self::Charon => "Charon",
            Self::Fenrir => "Fenrir",
            Self::Kore => "Kore",
            Self::Puck => "Puck",
            Self::Custom(v) => v.as_str(),
        }
    }
}

/// Audio encoding format.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AudioEncoding {
    Linear16,
    Mulaw,
    Alaw,
}

/// Speech generation config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_config: Option<VoiceConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    pub prebuilt_voice_config: PrebuiltVoiceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrebuiltVoiceConfig {
    pub voice_name: String,
}

// ── Setup message ─────────────────────────────────────────────────────────────

/// Tool / function definition for the live session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveTool {
    pub function_declarations: Vec<FunctionDeclaration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDeclaration {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Session configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LiveGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_modalities: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speech_config: Option<SpeechConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
}

// ── Client → Server messages ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct SetupMessage {
    pub setup: BidiSetup,
}

#[derive(Debug, Clone, Serialize)]
pub struct BidiSetup {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<SystemInstruction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<LiveGenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<LiveTool>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemInstruction {
    pub parts: Vec<TextPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPart {
    pub text: String,
}

/// Real-time audio/video input chunk.
#[derive(Debug, Clone, Serialize)]
pub struct RealtimeInputMessage {
    #[serde(rename = "realtimeInput")]
    pub realtime_input: RealtimeInput,
}

#[derive(Debug, Clone, Serialize)]
pub struct RealtimeInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_chunks: Option<Vec<MediaChunk>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_stream_end: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MediaChunk {
    pub mime_type: String,
    pub data: String, // base64-encoded
}

/// Turn-based text/multi-turn content.
#[derive(Debug, Clone, Serialize)]
pub struct ClientContentMessage {
    #[serde(rename = "clientContent")]
    pub client_content: ClientContent,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClientContent {
    pub turns: Vec<ContentTurn>,
    pub turn_complete: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContentTurn {
    pub role: String,
    pub parts: Vec<serde_json::Value>,
}

/// Tool call result back to the model.
#[derive(Debug, Clone, Serialize)]
pub struct ToolResponseMessage {
    pub tool_response: ToolResponse,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolResponse {
    pub function_responses: Vec<FunctionResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FunctionResponse {
    pub id: String,
    pub name: String,
    pub response: serde_json::Value,
}

// ── Server → Client messages ──────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerMessage {
    pub setup_complete: Option<serde_json::Value>,
    pub server_content: Option<ServerContent>,
    pub tool_call: Option<ToolCallEvent>,
    pub tool_call_cancellation: Option<ToolCallCancellation>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerContent {
    pub model_turn: Option<ModelTurn>,
    pub turn_complete: Option<bool>,
    pub interrupted: Option<bool>,
    pub generation_complete: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelTurn {
    pub parts: Vec<ModelPart>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelPart {
    pub text: Option<String>,
    pub inline_data: Option<InlineData>,
    pub function_call: Option<FunctionCallPart>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InlineData {
    pub mime_type: String,
    pub data: String, // base64-encoded audio/image
}

#[derive(Debug, Clone, Deserialize)]
pub struct FunctionCallPart {
    pub id: String,
    pub name: String,
    pub args: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallEvent {
    pub function_calls: Vec<FunctionCallPart>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallCancellation {
    pub ids: Vec<String>,
}

// ── Decoded event ─────────────────────────────────────────────────────────────

/// High-level decoded event from the Live API.
#[derive(Debug, Clone)]
pub enum LiveEvent {
    SetupComplete,
    TextDelta(String),
    AudioDelta(Vec<u8>), // raw PCM bytes
    TurnComplete,
    Interrupted,
    ToolCall(Vec<FunctionCallPart>),
    ToolCallCancelled(Vec<String>),
}

// ── LiveSession ───────────────────────────────────────────────────────────────

type WsSink = futures::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    Message,
>;
type WsStream = futures::stream::SplitStream<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
>;

/// An active Gemini Live API session.
///
/// # Example
///
/// ```no_run
/// use rs_ai_gemini::{GeminiProvider, live_api::{LiveEvent, LiveGenerationConfig}};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let provider = GeminiProvider::new(std::env::var("GOOGLE_API_KEY")?);
/// let mut session = provider.live_session("gemini-2.5-flash-live-preview").await?;
///
/// // Stream audio in and receive text/audio back
/// session.send_audio_chunk(b"...pcm16...", "audio/pcm").await?;
/// session.end_audio_turn().await?;
///
/// while let Some(event) = session.recv().await {
///     match event? {
///         LiveEvent::TextDelta(t) => print!("{t}"),
///         LiveEvent::AudioDelta(pcm) => { /* play audio */ }
///         LiveEvent::TurnComplete => break,
///         _ => {}
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub struct LiveSession {
    sink: Arc<Mutex<WsSink>>,
    stream: Arc<Mutex<WsStream>>,
}

impl LiveSession {
    pub async fn connect(api_key: &str, _model: &str, config: BidiSetup) -> LiveResult<Self> {
        let url_str = format!("{LIVE_URL}?key={api_key}");
        let request = url_str
            .as_str()
            .into_client_request()
            .map_err(LiveError::WebSocket)?;

        let (ws, _) = connect_async_tls_with_config(request, None, false, None).await?;
        let (mut sink, stream) = ws.split();

        // Send the setup message immediately after connection.
        let setup = SetupMessage { setup: config };
        sink.send(Message::Text(serde_json::to_string(&setup)?))
            .await?;

        Ok(Self {
            sink: Arc::new(Mutex::new(sink)),
            stream: Arc::new(Mutex::new(stream)),
        })
    }

    /// Send a chunk of raw audio bytes.
    ///
    /// `mime_type` is typically `"audio/pcm"` (16-bit, 16 kHz, mono) or
    /// `"audio/opus"`.
    pub async fn send_audio_chunk(&mut self, pcm: &[u8], mime_type: &str) -> LiveResult<()> {
        let data = base64::engine::general_purpose::STANDARD.encode(pcm);
        let msg = RealtimeInputMessage {
            realtime_input: RealtimeInput {
                media_chunks: Some(vec![MediaChunk {
                    mime_type: mime_type.to_string(),
                    data,
                }]),
                text: None,
                audio_stream_end: None,
            },
        };
        self.send_raw(&msg).await
    }

    /// Send a video frame as JPEG bytes.
    pub async fn send_video_frame(&mut self, jpeg: &[u8]) -> LiveResult<()> {
        let data = base64::engine::general_purpose::STANDARD.encode(jpeg);
        let msg = RealtimeInputMessage {
            realtime_input: RealtimeInput {
                media_chunks: Some(vec![MediaChunk {
                    mime_type: "image/jpeg".to_string(),
                    data,
                }]),
                text: None,
                audio_stream_end: None,
            },
        };
        self.send_raw(&msg).await
    }

    /// Signal that the audio stream has ended (triggers model response).
    pub async fn end_audio_turn(&mut self) -> LiveResult<()> {
        let msg = RealtimeInputMessage {
            realtime_input: RealtimeInput {
                media_chunks: None,
                text: None,
                audio_stream_end: Some(true),
            },
        };
        self.send_raw(&msg).await
    }

    /// Send a text message as a user turn.
    pub async fn send_text(&mut self, text: &str) -> LiveResult<()> {
        let msg = ClientContentMessage {
            client_content: ClientContent {
                turns: vec![ContentTurn {
                    role: "user".into(),
                    parts: vec![serde_json::json!({"text": text})],
                }],
                turn_complete: true,
            },
        };
        self.send_raw(&msg).await
    }

    /// Return a tool/function result to the model.
    pub async fn send_tool_result(
        &mut self,
        call_id: &str,
        name: &str,
        result: serde_json::Value,
    ) -> LiveResult<()> {
        let msg = ToolResponseMessage {
            tool_response: ToolResponse {
                function_responses: vec![FunctionResponse {
                    id: call_id.to_string(),
                    name: name.to_string(),
                    response: result,
                }],
            },
        };
        self.send_raw(&msg).await
    }

    /// Receive the next high-level event.
    pub async fn recv(&mut self) -> Option<LiveResult<LiveEvent>> {
        let mut stream = self.stream.lock().await;
        loop {
            let msg = stream.next().await?;
            match msg {
                Ok(Message::Text(text)) => {
                    let server_msg: ServerMessage = match serde_json::from_str(&text) {
                        Ok(m) => m,
                        Err(e) => return Some(Err(LiveError::Json(e))),
                    };
                    if let Some(event) = Self::decode(server_msg) {
                        return Some(Ok(event));
                    }
                    // Unknown/empty message — keep looping.
                }
                Ok(Message::Close(_)) => return None,
                Ok(_) => continue,
                Err(e) => return Some(Err(LiveError::WebSocket(e))),
            }
        }
    }

    fn decode(msg: ServerMessage) -> Option<LiveEvent> {
        if msg.setup_complete.is_some() {
            return Some(LiveEvent::SetupComplete);
        }
        if let Some(tc) = msg.tool_call {
            return Some(LiveEvent::ToolCall(tc.function_calls));
        }
        if let Some(tcc) = msg.tool_call_cancellation {
            return Some(LiveEvent::ToolCallCancelled(tcc.ids));
        }
        if let Some(sc) = msg.server_content {
            if sc.interrupted == Some(true) {
                return Some(LiveEvent::Interrupted);
            }
            if let Some(turn) = sc.model_turn {
                // Collect text first
                let text: String = turn
                    .parts
                    .iter()
                    .filter_map(|p| p.text.as_deref())
                    .collect();
                if !text.is_empty() {
                    return Some(LiveEvent::TextDelta(text));
                }
                // Then audio
                for part in &turn.parts {
                    if let Some(inline) = &part.inline_data {
                        if inline.mime_type.starts_with("audio/") {
                            if let Ok(pcm) =
                                base64::engine::general_purpose::STANDARD.decode(&inline.data)
                            {
                                return Some(LiveEvent::AudioDelta(pcm));
                            }
                        }
                    }
                }
            }
            if sc.turn_complete == Some(true) {
                return Some(LiveEvent::TurnComplete);
            }
        }
        None
    }

    async fn send_raw<T: Serialize>(&mut self, msg: &T) -> LiveResult<()> {
        let json = serde_json::to_string(msg)?;
        let mut sink = self.sink.lock().await;
        sink.send(Message::Text(json)).await?;
        Ok(())
    }
}
