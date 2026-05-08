#[cfg(not(target_arch = "wasm32"))]
use rs_ai_core::generate_text;
#[cfg(not(target_arch = "wasm32"))]
use rs_ai_local::browser::BrowserAiProvider;
#[cfg(not(target_arch = "wasm32"))]
use rs_ai_local::browser::NoOpBrowserBridge as BrowserBridge;
#[cfg(target_arch = "wasm32")]
use rs_ai_local::browser::{wasm_bridge::WasmBrowserBridge, BrowserAiOptions};

#[cfg(target_arch = "wasm32")]
async fn run() -> Result<(), String> {
    let bridge = WasmBrowserBridge;
    let capabilities = bridge.detect().await;
    println!("Browser AI available: {}", capabilities.available);
    println!("Browser: {:?}", capabilities.browser);
    println!("Backing model: {:?}", capabilities.backing_model);

    if capabilities.available {
        let result = bridge
            .generate(
                "Explain Rust ownership in one paragraph",
                &BrowserAiOptions::default(),
            )
            .await?;
        println!("Response: {result}");
    }

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let provider = BrowserAiProvider::new(BrowserBridge);
    let capabilities = provider.detect().await;
    println!("Browser AI available: {}", capabilities.available);
    println!("Browser: {:?}", capabilities.browser);
    println!("Backing model: {:?}", capabilities.backing_model);

    if capabilities.available {
        let model = provider.model();
        let result = generate_text(&model, "Explain Rust ownership in one paragraph").await?;
        println!("Response: {result}");
    }

    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    wasm_bindgen_futures::spawn_local(async {
        if let Err(error) = run().await {
            web_sys::console::error_1(&error.into());
        }
    });
}

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run().await
}
