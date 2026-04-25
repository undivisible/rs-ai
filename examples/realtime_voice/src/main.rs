//! OpenAI Realtime API — voice agent with function calling.
//!
//! Run: `OPENAI_API_KEY=sk-... cargo run --example realtime_voice`

use rs_ai_providers::chatgpt::{
    realtime_api::{ServerEvent, SessionConfig, TurnDetection, Voice},
    ChatGptProvider, GPT_4O_REALTIME,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = ChatGptProvider::new(std::env::var("OPENAI_API_KEY")?);

    println!("🎙️  Connecting to OpenAI Realtime API ({GPT_4O_REALTIME})…");
    let mut session = provider.realtime_session(GPT_4O_REALTIME).await?;

    // Configure: text + audio output, server-side VAD, alloy voice.
    session
        .configure(SessionConfig {
            modalities: Some(vec![
                rs_ai_providers::chatgpt::realtime_api::Modality::Text,
                rs_ai_providers::chatgpt::realtime_api::Modality::Audio,
            ]),
            voice: Some(Voice::Alloy),
            turn_detection: Some(TurnDetection::server_vad()),
            instructions: Some(
                "You are a helpful voice assistant. Be concise and friendly.".into(),
            ),
            ..Default::default()
        })
        .await?;
    println!("✅  Session configured.");

    // Send a text message (in a real app, you'd stream PCM audio instead).
    println!("\n💬  Sending: \"Tell me a one-sentence fun fact about Rust.\"");
    session
        .send_text("Tell me a one-sentence fun fact about Rust.")
        .await?;

    // Collect response events.
    println!("\n📥  Response:\n");
    let mut text_buf = String::new();
    let mut audio_bytes = 0usize;

    loop {
        match session.recv().await {
            None => {
                println!("\n[connection closed]");
                break;
            }
            Some(Err(e)) => {
                eprintln!("Error: {e}");
                break;
            }
            Some(Ok(event)) => match event {
                ServerEvent::TextDelta { delta, .. } => {
                    print!("{delta}");
                    text_buf.push_str(&delta);
                }
                ServerEvent::AudioDelta { delta, .. } => {
                    // decode base64 → PCM bytes (play via a real audio sink in production)
                    if let Ok(pcm) =
                        rs_ai_providers::chatgpt::realtime_api::RealtimeSession::decode_audio(
                            &delta,
                        )
                    {
                        audio_bytes += pcm.len();
                    }
                }
                ServerEvent::AudioDone { .. } => {
                    println!("\n🔊  Audio done ({audio_bytes} PCM bytes received)");
                }
                ServerEvent::ResponseDone { .. } => {
                    println!("\n✅  Response complete.");
                    break;
                }
                ServerEvent::Error { error } => {
                    eprintln!("\n❌  API error [{}]: {}", error.code, error.message);
                    break;
                }
                _ => {}
            },
        }
    }

    println!("\nFull transcript: {text_buf}");
    println!("\n💡  In production: pipe audio_bytes into your speaker via cpal or rodio.");
    Ok(())
}
