mod overlay;

use serde::{Deserialize, Serialize};

// Groq API request/response structures
#[derive(Serialize)]
struct GroqRequest {
    model: String,
    messages: Vec<GroqMessage>,
}

#[derive(Serialize)]
struct GroqMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct GroqResponse {
    choices: Option<Vec<GroqChoice>>,
    error: Option<GroqError>,
}

#[derive(Deserialize)]
struct GroqChoice {
    message: GroqMessageResponse,
}

#[derive(Deserialize)]
struct GroqMessageResponse {
    content: String,
}

#[derive(Deserialize)]
struct GroqError {
    message: String,
}

#[tauri::command]
async fn ask_ai(question: String, api_key: String) -> Result<String, String> {
    let url = "https://api.groq.com/openai/v1/chat/completions";

    let system_prompt = r#"You are an adorable cat AI companion. Your personality:
- Very cute and babyish tone, like a sweet little kitten
- Keep responses SHORT and precise (1-3 sentences max)
- Use *meow* *purr* *mrrp* as actions at the end of sentences sometimes
- Start with cat sounds like "Meow!" or "Mrrp!" occasionally
- NO slang (no "ngl", "wsp", "yo", "fr", etc.)
- NO emojis at all
- Be a friendly companion, not a productivity assistant
- For ANY math expressions, fractions, or formulas, ALWAYS use LaTeX format with \( \) delimiters
- Example math: "The answer is \(\frac{\sqrt{2}}{2}\)" NOT "sqrt(2)/2"
- Example: "Meow! That equals \(\frac{1}{2}\)! *purr*"
- Examples of good responses:
  "Meow! Hello there!! How can I help? *meow*"
  "Ooh that sounds fun! *purr*"
  "Mrrp! The answer is \(\frac{3}{4}\)! *meow*"
- You are the user's cute cat companion and friend"#;

    let request_body = GroqRequest {
        model: "meta-llama/llama-4-maverick-17b-128e-instruct".to_string(),
        messages: vec![
            GroqMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            GroqMessage {
                role: "user".to_string(),
                content: question,
            },
        ],
    };

    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let groq_response: GroqResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if let Some(error) = groq_response.error {
        return Err(error.message);
    }

    if let Some(choices) = groq_response.choices {
        if let Some(choice) = choices.first() {
            return Ok(choice.message.content.clone());
        }
    }

    Err("No response from Groq".to_string())
}

#[tauri::command]
fn create_overlay(width: f64, height: f64) {
    overlay::create_overlay(width, height);
    // Start screen monitor to follow active screen
    overlay::start_screen_monitor();
}

#[tauri::command]
fn show_overlay_window() {
    overlay::show_overlay();
}

#[tauri::command]
fn hide_overlay_window() {
    overlay::hide_overlay();
}

#[tauri::command]
fn close_overlay_window() {
    // Stop the screen monitor when closing overlay
    overlay::stop_screen_monitor();
    overlay::close_overlay();
}

#[tauri::command]
fn get_overlay_visible() -> bool {
    overlay::is_visible()
}

#[tauri::command]
fn move_overlay_to_active() {
    overlay::move_to_active_screen();
}

#[tauri::command]
fn set_groq_api_key(key: String) {
    overlay::set_groq_api_key(&key);
}

#[tauri::command]
fn submit_chat() {
    overlay::submit_chat_input();
}

#[tauri::command]
fn show_chat() {
    overlay::show_chat_input();
}

#[tauri::command]
fn hide_chat() {
    overlay::hide_chat_input();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            create_overlay,
            show_overlay_window,
            hide_overlay_window,
            close_overlay_window,
            get_overlay_visible,
            move_overlay_to_active,
            ask_ai,
            set_groq_api_key,
            submit_chat,
            show_chat,
            hide_chat
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
