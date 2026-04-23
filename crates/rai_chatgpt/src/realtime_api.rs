//! OpenAI Realtime API — WebSocket-based voice and audio streaming.
//!
//! The Realtime API maintains a persistent WebSocket connection to exchange
//! low-latency audio, text, and function-call events bidirectionally.
//!
//! Reference: <https://platform.openai.com/docs/api-reference/realtime>

use std::sync::Arc;

use base64::Engine;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Mutex;
use tokio_tungstenite::{
    connect_async_tls_with_config,
    tungstenite::{client::IntoClientRequest, http::HeaderValue, Message},
};

const REALTIME_URL: &str = "wss://api.openai.com/v1/realtime";
const BETA_HEADER: &str = "realtime=v1";

// ── Error ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum RealtimeError {
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("URL parse error: {0}")]
    Url(#[from] url::ParseError),
    #[error("Channel closed")]
    ChannelClosed,
    #[error("API error: {code} — {message}")]
    ApiError { code: String, message: String },
}

pub type RealtimeResult<T> = Result<T, RealtimeError>;

// ── Events sent TO the API ────────────────────────────────────────────────────

/// Modalities supported by the Realtime API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Modality {
    Text,
    Audio,
}

/// Voice options for audio output.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Voice {
    #[default]
    Alloy,
    Ash,
    Ballad,
    Coral,
    Echo,
    Sage,
    Shimmer,
    Verse,
}

/// Audio format for input or output.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AudioFormat {
    #[default]
    Pcm16,
    G711Ulaw,
    G711Alaw,
}

/// Turn detection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnDetection {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix_padding_ms: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub silence_duration_ms: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_response: Option<bool>,
}

impl TurnDetection {
    pub fn server_vad() -> Self {
        Self {
            kind: "server_vad".into(),
            threshold: Some(0.5),
            prefix_padding_ms: Some(300),
            silence_duration_ms: Some(500),
            create_response: Some(true),
        }
    }
}

/// A function/tool definition for the Realtime session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTool {
    #[serde(rename = "type")]
    pub kind: String,
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Session configuration update.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modalities: Option<Vec<Modality>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice: Option<Voice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_audio_format: Option<AudioFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_audio_format: Option<AudioFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_detection: Option<TurnDetection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<RealtimeTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_response_output_tokens: Option<serde_json::Value>,
}

/// Client → server events.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientEvent {
    #[serde(rename = "session.update")]
    SessionUpdate { session: SessionConfig },
    #[serde(rename = "input_audio_buffer.append")]
    InputAudioBufferAppend { audio: String },
    #[serde(rename = "input_audio_buffer.commit")]
    InputAudioBufferCommit,
    #[serde(rename = "input_audio_buffer.clear")]
    InputAudioBufferClear,
    #[serde(rename = "conversation.item.create")]
    ConversationItemCreate { item: ConversationItem },
    #[serde(rename = "conversation.item.truncate")]
    ConversationItemTruncate {
        item_id: String,
        content_index: u32,
        audio_end_ms: u32,
    },
    #[serde(rename = "conversation.item.delete")]
    ConversationItemDelete { item_id: String },
    #[serde(rename = "response.create")]
    ResponseCreate {
        #[serde(skip_serializing_if = "Option::is_none")]
        response: Option<ResponseConfig>,
    },
    #[serde(rename = "response.cancel")]
    ResponseCancel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationItem {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<ItemContent>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemContent {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResponseConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modalities: Option<Vec<Modality>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

// ── Events received FROM the API ──────────────────────────────────────────────

/// Server → client events.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerEvent {
    #[serde(rename = "session.created")]
    SessionCreated { session: serde_json::Value },
    #[serde(rename = "session.updated")]
    SessionUpdated { session: serde_json::Value },
    #[serde(rename = "input_audio_buffer.speech_started")]
    SpeechStarted {
        audio_start_ms: u64,
        item_id: String,
    },
    #[serde(rename = "input_audio_buffer.speech_stopped")]
    SpeechStopped {
        audio_end_ms: u64,
        item_id: String,
    },
    #[serde(rename = "input_audio_buffer.committed")]
    AudioBufferCommitted { item_id: String },
    #[serde(rename = "conversation.item.created")]
    ConversationItemCreated { item: serde_json::Value },
    #[serde(rename = "conversation.item.input_audio_transcription.completed")]
    TranscriptionCompleted {
        item_id: String,
        content_index: u32,
        transcript: String,
    },
    #[serde(rename = "response.created")]
    ResponseCreated { response: serde_json::Value },
    #[serde(rename = "response.output_item.added")]
    ResponseOutputItemAdded { item: serde_json::Value },
    #[serde(rename = "response.audio_transcript.delta")]
    AudioTranscriptDelta {
        item_id: String,
        delta: String,
    },
    #[serde(rename = "response.audio.delta")]
    AudioDelta {
        item_id: String,
        delta: String, // base64-encoded PCM16
    },
    #[serde(rename = "response.audio.done")]
    AudioDone { item_id: String },
    #[serde(rename = "response.text.delta")]
    TextDelta {
        item_id: String,
        delta: String,
    },
    #[serde(rename = "response.text.done")]
    TextDone {
        item_id: String,
        text: String,
    },
    #[serde(rename = "response.function_call_arguments.delta")]
    FunctionCallArgumentsDelta {
        item_id: String,
        call_id: String,
        delta: String,
    },
    #[serde(rename = "response.function_call_arguments.done")]
    FunctionCallArgumentsDone {
        item_id: String,
        call_id: String,
        name: String,
        arguments: String,
    },
    #[serde(rename = "response.done")]
    ResponseDone { response: serde_json::Value },
    #[serde(rename = "error")]
    Error {
        error: RealtimeApiError,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RealtimeApiError {
    pub r#type: String,
    pub code: String,
    pub message: String,
}

// ── RealtimeSession ───────────────────────────────────────────────────────────

type WsSink = futures::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    Message,
>;
type WsStream = futures::stream::SplitStream<
    tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
>;

/// An active Realtime API session.
///
/// # Example
///
/// ```no_run
/// use rai_chatgpt::{ChatGptProvider, GPT_4O_REALTIME};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let provider = ChatGptProvider::new(std::env::var("OPENAI_API_KEY")?);
/// let mut session = provider.realtime_session(GPT_4O_REALTIME).await?;
///
/// // Configure the session
/// session.configure(Default::default()).await?;
///
/// // Send a text message and get a response
/// session.send_text("Hello, how are you?").await?;
/// while let Some(event) = session.recv().await {
///     match event? {
///         rai_chatgpt::realtime_api::ServerEvent::TextDelta { delta, .. } => print!("{delta}"),
///         rai_chatgpt::realtime_api::ServerEvent::ResponseDone { .. } => break,
///         _ => {}
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub struct RealtimeSession {
    sink: Arc<Mutex<WsSink>>,
    stream: Arc<Mutex<WsStream>>,
    model: String,
}

impl RealtimeSession {
    pub(crate) async fn connect(api_key: &str, model: &str) -> RealtimeResult<Self> {
        let url_str = format!("{REALTIME_URL}?model={model}");
        let mut request = url_str.as_str().into_client_request()?;
        request.headers_mut().insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {api_key}"))
                .map_err(|e| RealtimeError::ApiError { code: "header".into(), message: e.to_string() })?,
        );
        request.headers_mut().insert(
            "OpenAI-Beta",
            HeaderValue::from_static(BETA_HEADER),
        );

        let (ws, _response) = connect_async_tls_with_config(request, None, false, None).await?;
        let (sink, stream) = ws.split();
        Ok(Self {
            sink: Arc::new(Mutex::new(sink)),
            stream: Arc::new(Mutex::new(stream)),
            model: model.to_string(),
        })
    }

    /// Update the session configuration.
    pub async fn configure(&mut self, config: SessionConfig) -> RealtimeResult<()> {
        self.send_event(ClientEvent::SessionUpdate { session: config })
            .await
    }

    /// Send raw PCM16 audio bytes (will be base64-encoded).
    pub async fn send_audio(&mut self, pcm16: &[u8]) -> RealtimeResult<()> {
        let encoded = base64::engine::general_purpose::STANDARD.encode(pcm16);
        self.send_event(ClientEvent::InputAudioBufferAppend { audio: encoded })
            .await
    }

    /// Commit the audio buffer and request a response.
    pub async fn commit_audio(&mut self) -> RealtimeResult<()> {
        self.send_event(ClientEvent::InputAudioBufferCommit).await?;
        self.send_event(ClientEvent::ResponseCreate { response: None })
            .await
    }

    /// Send a text message as a user turn.
    pub async fn send_text(&mut self, text: &str) -> RealtimeResult<()> {
        self.send_event(ClientEvent::ConversationItemCreate {
            item: ConversationItem {
                kind: "message".into(),
                id: None,
                role: Some("user".into()),
                content: Some(vec![ItemContent {
                    kind: "input_text".into(),
                    text: Some(text.to_string()),
                    audio: None,
                    transcript: None,
                }]),
                call_id: None,
                output: None,
            },
        })
        .await?;
        self.send_event(ClientEvent::ResponseCreate { response: None })
            .await
    }

    /// Return a tool call result back to the model.
    pub async fn send_tool_result(
        &mut self,
        call_id: &str,
        output: &str,
    ) -> RealtimeResult<()> {
        self.send_event(ClientEvent::ConversationItemCreate {
            item: ConversationItem {
                kind: "function_call_output".into(),
                id: None,
                role: None,
                content: None,
                call_id: Some(call_id.to_string()),
                output: Some(output.to_string()),
            },
        })
        .await?;
        self.send_event(ClientEvent::ResponseCreate { response: None })
            .await
    }

    /// Receive the next server event.
    pub async fn recv(&mut self) -> Option<RealtimeResult<ServerEvent>> {
        let mut stream = self.stream.lock().await;
        loop {
            let msg = stream.next().await?;
            match msg {
                Ok(Message::Text(text)) => {
                    return Some(serde_json::from_str(&text).map_err(RealtimeError::Json));
                }
                Ok(Message::Close(_)) => return None,
                Ok(_) => continue, // ping/pong/binary — ignore
                Err(e) => return Some(Err(RealtimeError::WebSocket(e))),
            }
        }
    }

    /// Decode a base64 audio delta from the server into raw PCM16 bytes.
    pub fn decode_audio(base64_delta: &str) -> RealtimeResult<Vec<u8>> {
        base64::engine::general_purpose::STANDARD
            .decode(base64_delta)
            .map_err(|e| {
                RealtimeError::ApiError {
                    code: "decode_error".into(),
                    message: e.to_string(),
                }
            })
    }

    async fn send_event(&mut self, event: ClientEvent) -> RealtimeResult<()> {
        let json = serde_json::to_string(&event)?;
        let mut sink = self.sink.lock().await;
        sink.send(Message::Text(json.into())).await?;
        Ok(())
    }
}
