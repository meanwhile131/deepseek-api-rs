//! End-to-end tests for the DeepSeek API client.
//!
//! These tests require the `DEEPSEEK_TOKEN` environment variable to be set.
//! They are ignored by default to avoid network requests and rate limits.
//! Run with: `cargo test -- --ignored` to execute them.

use deepseek_api::{DeepSeekAPI, StreamChunk};
use futures_util::{StreamExt, pin_mut};

#[tokio::test]
async fn test_e2e_completion() {
    let token = match std::env::var("DEEPSEEK_TOKEN") {
        Ok(t) => t,
        Err(_) => {
            println!("Skipping e2e test because DEEPSEEK_TOKEN is not set");
            return;
        }
    };

    let api = DeepSeekAPI::new(token).await.unwrap();
    let chat = api.create_chat().await.unwrap();
    let chat_id = chat["id"].as_str().expect("Chat ID not found");

    let response = api
        .complete(chat_id, "Say 'test'", None, false, false)
        .await
        .unwrap();

    assert!(!response.is_empty(), "Response should not be empty");
    println!("Completion response: {}", response);
}

#[tokio::test]
async fn test_e2e_streaming() {
    let token = match std::env::var("DEEPSEEK_TOKEN") {
        Ok(t) => t,
        Err(_) => {
            println!("Skipping e2e test because DEEPSEEK_TOKEN is not set");
            return;
        }
    };

    let api = DeepSeekAPI::new(token).await.unwrap();
    let chat = api.create_chat().await.unwrap();
    let chat_id = chat["id"].as_str().expect("Chat ID not found").to_string();

    let stream = api.complete_stream(
        chat_id,
        "Say 'test'".to_string(),
        None,
        false,
        false,
    );
    pin_mut!(stream); // pin the stream so we can call .next()

    let mut got_content = false;
    while let Some(chunk) = stream.next().await {
        match chunk.unwrap() {
            StreamChunk::Content(content) => {
                got_content = true;
                println!("Content: {}", content);
            }
            StreamChunk::Thinking(thought) => {
                println!("Thinking: {}", thought);
            }
            StreamChunk::Message(msg) => {
                println!("Final message: {}", msg);
            }
        }
    }

    assert!(got_content, "Should have received at least one content chunk");
}