//! Simple CLI example for the `DeepSeek` API client.

use deepseek_api::DeepSeekAPI;
use futures_util::StreamExt;
use std::env;
use tokio::pin;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let token = env::var("DEEPSEEK_TOKEN").expect("DEEPSEEK_TOKEN environment variable not set");
    let prompt = env::args().nth(1).expect("Usage: deepseek-api <prompt>");

    let api = DeepSeekAPI::new(token).await?;
    let chat = api.create_chat().await?;
    let chat_id = chat.id.as_str();

    println!("Chat ID: {chat_id}");
    println!("Sending prompt: {prompt}");

    let stream = api.complete_stream(chat_id.to_string(), prompt, None, true, true, vec![]);
    pin!(stream);
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(deepseek_api::StreamChunk::Content(text)) => println!("Content: {text}"),
            Ok(deepseek_api::StreamChunk::Thinking(text)) => println!("Thinking: {text}"),
            Ok(deepseek_api::StreamChunk::Message(msg)) => println!("Final message: {msg:#?}"),
            Err(e) => eprintln!("Error: {e}"),
        }
    }

    // If the final message's status is "INCOMPLETE", you can continue it by calling:
    // let mut continue_stream = api.continue_stream(chat_id.to_string(), final_msg.message_id.unwrap(), true);
    // while let Some(chunk) = continue_stream.next().await { ... }

    Ok(())
}
