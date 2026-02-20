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

    let mut final_msg = final_message.expect("No final message received");

    // With the prompt "think for as long as possible, do NOT stop thinking", we expect the message to be incomplete.
    // Assert that the status is "INCOMPLETE".
    assert_eq!(final_msg.status.as_deref(), Some("INCOMPLETE"), "Expected incomplete message but got status {:?}", final_msg.status);

    // Also check that we received some thinking content.
    assert!(!thinking_chunks.is_empty(), "Expected at least one thinking chunk");

    let msg_id = final_msg.message_id.expect("Message ID missing for incomplete message");
    println!("Message incomplete, continuing with message_id: {}", msg_id);
    let mut continue_stream = api.continue_stream(chat_id.to_string(), msg_id, true);
    pin!(continue_stream);

    let mut continued_content = Vec::new();
    let mut continued_thinking = Vec::new();
    while let Some(chunk) = continue_stream.next().await {
        match chunk? {
            StreamChunk::Content(text) => {
                println!("Continued content chunk ({} chars)", text.len());
                continued_content.push(text);
            }
            StreamChunk::Thinking(text) => {
                println!("Continued thinking chunk ({} chars)", text.len());
                continued_thinking.push(text);
            }
            StreamChunk::Message(msg) => {
                println!("Final continued message received with status: {:?}", msg.status);
                final_msg = msg;
                break;
            }
        }
    }

    // After continuation, the status should be "DONE" or at least not "INCOMPLETE".
    // The exact status might be "DONE" or absent, but we expect it not to be incomplete.
    assert_ne!(final_msg.status.as_deref(), Some("INCOMPLETE"), "Message still incomplete after continuation");

    // Optionally, check that more content or thinking was added.
    if !continued_content.is_empty() || !continued_thinking.is_empty() {
        println!("Continuation added {} content chunks and {} thinking chunks", continued_content.len(), continued_thinking.len());
    } else {
        println!("No additional content received during continuation");
    }

    // If we got here, the test passed.
    Ok(())
}