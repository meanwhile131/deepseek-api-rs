//! End-to-end tests for the DeepSeek API client.
//!
//! These tests require the `DEEPSEEK_TOKEN` environment variable to be set.

use deepseek_api::{DeepSeekAPI, StreamChunk};
use futures_util::{StreamExt, pin_mut};

#[tokio::test]
async fn test_e2e_completion() {
    let token = std::env::var("DEEPSEEK_TOKEN")
        .expect("DEEPSEEK_TOKEN environment variable must be set to run this test");

    let api = DeepSeekAPI::new(token).await.unwrap();
    let chat = api.create_chat().await.unwrap();
    let chat_id = &chat.id;

    let response = api
        .complete(chat_id, "Say 'test'", None, false, false)
        .await
        .unwrap();

    assert!(!response.content.is_empty(), "Response content should not be empty");
    println!("Completion response: {:#?}", response);
    // Check that some expected fields are present
    assert!(response.message_id.is_some(), "message_id should be present");
    assert!(response.parent_id.is_some(), "parent_id should be present");
    assert!(response.role.is_some(), "role should be present");
    assert!(response.inserted_at.is_some(), "inserted_at should be present");
}

#[tokio::test]
async fn test_e2e_streaming() {
    let token = std::env::var("DEEPSEEK_TOKEN")
        .expect("DEEPSEEK_TOKEN environment variable must be set to run this test");

    let api = DeepSeekAPI::new(token).await.unwrap();
    let chat = api.create_chat().await.unwrap();
    let chat_id = chat.id.clone();

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
                println!("Final message: {:#?}", msg);
                // Optionally check content and fields
                assert!(!msg.content.is_empty(), "Final message content should not be empty");
                assert!(msg.message_id.is_some(), "message_id should be present");
                assert!(msg.parent_id.is_some(), "parent_id should be present");
                assert!(msg.role.is_some(), "role should be present");
                assert!(msg.inserted_at.is_some(), "inserted_at should be present");
            }
        }
    }

    assert!(got_content, "Should have received at least one content chunk");
}