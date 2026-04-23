//! Gemini Live API — real-time bidirectional voice/text conversation.
//!
//! Run: `GOOGLE_API_KEY=... cargo run --example gemini_live`

use rai_gemini::{
    live_api::{LiveEvent, LiveVoice, SpeechConfig, VoiceConfig, PrebuiltVoiceConfig,
               LiveGenerationConfig, BidiSetup},
    GEMINI_FLASH_LIVE_LATEST,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("GOOGLE_API_KEY")?;

    println!("🎙️  Connecting to Gemini Live API ({GEMINI_FLASH_LIVE_LATEST})…");

    // Build custom setup with a specific voice.
    let setup = BidiSetup {
        model: format!("models/{GEMINI_FLASH_LIVE_LATEST}"),
        system_instruction: Some(rai_gemini::live_api::SystemInstruction {
            parts: vec![rai_gemini::live_api::TextPart {
                text: "You are a friendly, concise voice assistant.".into(),
            }],
        }),
        generation_config: Some(LiveGenerationConfig {
            response_modalities: Some(vec!["AUDIO".into(), "TEXT".into()]),
            speech_config: Some(SpeechConfig {
                voice_config: Some(VoiceConfig {
                    prebuilt_voice_config: PrebuiltVoiceConfig {
                        voice_name: LiveVoice::Aoede.as_str().to_string(),
                    },
                }),
                language_code: Some("en-US".into()),
            }),
            ..Default::default()
        }),
        tools: None,
    };

    // Connect and wait for setup confirmation.
    let mut session = rai_gemini::live_api::LiveSession::connect(
        &api_key,
        GEMINI_FLASH_LIVE_LATEST,
        setup,
    )
    .await?;
    println!("✅  Connected. Waiting for setup confirmation…");

    // Wait for SetupComplete before sending.
    while let Some(ev) = session.recv().await {
        if matches!(ev?, LiveEvent::SetupComplete) {
            println!("✅  Setup confirmed.");
            break;
        }
    }

    // Send a text turn.
    let question = "In one sentence, what is the coolest thing about Rust?";
    println!("\n💬  Sending: \"{question}\"");
    session.send_text(question).await?;

    // Collect response.
    println!("\n📥  Response:\n");
    let mut audio_bytes = 0usize;
    let mut text_buf = String::new();

    loop {
        match session.recv().await {
            None => {
                println!("\n[session closed]");
                break;
            }
            Some(Err(e)) => {
                eprintln!("Error: {e}");
                break;
            }
            Some(Ok(event)) => match event {
                LiveEvent::TextDelta(t) => {
                    print!("{t}");
                    text_buf.push_str(&t);
                }
                LiveEvent::AudioDelta(pcm) => {
                    // In production: write pcm to a speaker via cpal/rodio.
                    audio_bytes += pcm.len();
                }
                LiveEvent::TurnComplete => {
                    println!("\n\n✅  Turn complete.");
                    println!("🔊  Received {audio_bytes} PCM bytes of audio.");
                    break;
                }
                LiveEvent::Interrupted => {
                    println!("\n[barge-in: model interrupted]");
                    break;
                }
                LiveEvent::ToolCall(calls) => {
                    for call in &calls {
                        println!("\n🔧  Tool call: {} {:?}", call.name, call.args);
                        // Execute tool, then:
                        // session.send_tool_result(&call.id, &call.name, result).await?;
                    }
                }
                _ => {}
            },
        }
    }

    println!("\nTranscript: {text_buf}");
    println!("\n💡  In production: pipe audio PCM into cpal or write to a WAV file.");
    Ok(())
}
