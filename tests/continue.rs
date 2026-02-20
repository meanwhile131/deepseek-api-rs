use anyhow::Result;
use deepseek_api::{DeepSeekAPI, StreamChunk};
use futures_util::StreamExt;
use std::env;
use tokio::pin;

#[tokio::test]
async fn test_continue_incomplete_message() -> Result<()> {
    // This test requires a valid DEEPSEEK_TOKEN environment variable.
    // Run with: DEEPSEEK_TOKEN=your_token_here cargo test --test continue -- --test-threads=1
    let token = env::var("DEEPSEEK_TOKEN").expect("DEEPSEEK_TOKEN not set");
    let api = DeepSeekAPI::new(token).await?;

    // Create a new chat session
    let chat = api.create_chat().await?;
    let chat_id = chat.id.as_str();

    // Send a prompt designed to cause an incomplete response by asking the model to think continuously.
    let prompt = "think for as long as possible, do NOT stop thinking";

    // Collect the streaming response until finish, with thinking enabled.
    let mut stream = api.complete_stream(chat_id.to_string(), prompt.to_string(), None, false, true);
    pin!(stream);

    let mut final_message = None;
    let mut content_chunks = Vec::new();
    let mut thinking_chunks = Vec::new();
    while let Some(chunk) = stream.next().await {
        match chunk? {
            StreamChunk::Content(text) => {
                println!("Content chunk received ({} chars)", text.len());
                content_chunks.push(text);
            }
            StreamChunk::Thinking(text) => {
                println!("Thinking chunk received ({} chars)", text.len());
                thinking_chunks.push(text);
            }
            StreamChunk::Message(msg) => {
                println!("Final message received with status: {:?}", msg.status);
                final_message = Some(msg);
                break;
            }
        }
    }

    let final_msg = final_message.expect("No final message received");

    // With auto-continuation, the message should be complete.
    assert_ne!(final_msg.status.as_deref(), Some("INCOMPLETE"), "Message should be complete after auto-continuation");

    // Also check that we received some thinking and content chunks.
    assert!(!thinking_chunks.is_empty(), "Expected at least one thinking chunk");
    assert!(!content_chunks.is_empty(), "Expected at least one content chunk");

    // If we got here, the test passed.
    Ok(())
}