//! Simple CLI example for the DeepSeek API client.

use deepseek_api::DeepSeekAPI;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let token = env::var("DEEPSEEK_TOKEN")
        .expect("DEEPSEEK_TOKEN environment variable not set");
    let prompt = env::args().nth(1)
        .expect("Usage: deepseek-api <prompt>");

    let api = DeepSeekAPI::new(token).await?;
    let chat = api.create_chat().await?;
    let chat_id = chat.id.as_str();

    println!("Chat ID: {}", chat_id);
    println!("Sending prompt: {}", prompt);

    let response = api.complete(chat_id, &prompt, None, false, false).await?;
    println!("Response: {:#?}", response);

    Ok(())
}