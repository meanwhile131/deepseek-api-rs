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
        .complete(chat_id, "Hello", None, false, false)
        .await
        .unwrap();

    assert!(
        !response.content.is_empty(),
        "Response content should not be empty"
    );
    println!("Completion response: {:#?}", response);
    // Check that some expected fields are present
    assert!(
        response.message_id.is_some(),
        "message_id should be present"
    );
    assert!(response.parent_id.is_some(), "parent_id should be present");
    assert!(response.role.is_some(), "role should be present");
    assert!(
        response.inserted_at.is_some(),
        "inserted_at should be present"
    );
}

#[tokio::test]
async fn test_e2e_get_chat_info() {
    let token = std::env::var("DEEPSEEK_TOKEN")
        .expect("DEEPSEEK_TOKEN environment variable must be set to run this test");

    let api = DeepSeekAPI::new(token).await.unwrap();
    let chat = api.create_chat().await.unwrap();

    // Fetch chat info using the chat ID
    let chat_info = api.get_chat_info(&chat.id).await.unwrap();

    // Verify that the returned chat info matches the created chat
    assert_eq!(chat_info.id, chat.id);
    assert_eq!(chat_info.seq_id, chat.seq_id);
    assert_eq!(chat_info.agent, chat.agent);
    assert_eq!(chat_info.title, chat.title); // both should be None initially
    assert_eq!(chat_info.title_type, chat.title_type);
    assert_eq!(chat_info.version, chat.version);
    assert_eq!(chat_info.current_message_id, chat.current_message_id); // both None
    assert_eq!(chat_info.pinned, chat.pinned);
    // inserted_at and updated_at might be slightly different due to timing, so just check they exist
    assert!(chat_info.inserted_at > 0.0);
    assert!(chat_info.updated_at > 0.0);
}

#[tokio::test]
async fn test_e2e_chat_info_after_completion() {
    let token = std::env::var("DEEPSEEK_TOKEN")
        .expect("DEEPSEEK_TOKEN environment variable must be set to run this test");

    let api = DeepSeekAPI::new(token).await.unwrap();
    let chat = api.create_chat().await.unwrap();
    let chat_id = chat.id.clone();

    // Send a completion
    let response = api
        .complete(
            &chat_id,
            "Hello, this is a test message",
            None,
            false,
            false,
        )
        .await
        .unwrap();

    // Fetch chat info again
    let chat_info = api.get_chat_info(&chat_id).await.unwrap();

    // The chat should now have a current_message_id (the ID of the last message)
    assert!(
        chat_info.current_message_id.is_some(),
        "current_message_id should be set after completion"
    );
    assert_eq!(chat_info.current_message_id, response.message_id);
    assert!(
        chat_info.updated_at > chat.inserted_at,
        "updated_at should be later than inserted_at"
    );
    // Version might have incremented
    assert!(chat_info.version >= chat.version);
}

#[tokio::test]
async fn test_e2e_thinking() {
    let token = std::env::var("DEEPSEEK_TOKEN")
        .expect("DEEPSEEK_TOKEN environment variable must be set to run this test");

    let api = DeepSeekAPI::new(token).await.unwrap();
    let chat = api.create_chat().await.unwrap();
    let chat_id = &chat.id;

    let response = api
        .complete(
            chat_id,
            "Explain quantum computing in one sentence",
            None,
            false,
            true,
        )
        .await
        .unwrap();

    assert!(
        !response.content.is_empty(),
        "Response content should not be empty"
    );
    // thinking_content may or may not be present depending on the model
    if let Some(thinking) = &response.thinking_content {
        println!("Thinking content: {}", thinking);
    }
}

#[tokio::test]
async fn test_e2e_search() {
    let token = std::env::var("DEEPSEEK_TOKEN")
        .expect("DEEPSEEK_TOKEN environment variable must be set to run this test");

    let api = DeepSeekAPI::new(token).await.unwrap();
    let chat = api.create_chat().await.unwrap();
    let chat_id = &chat.id;

    let response = api
        .complete(
            chat_id,
            "What is the capital of France? Use web search.",
            None,
            true,
            false,
        )
        .await
        .unwrap();

    assert!(
        !response.content.is_empty(),
        "Response content should not be empty"
    );
    println!("Search response: {}", response.content);
}

#[tokio::test]
async fn test_e2e_conversation() {
    let token = std::env::var("DEEPSEEK_TOKEN")
        .expect("DEEPSEEK_TOKEN environment variable must be set to run this test");

    let api = DeepSeekAPI::new(token).await.unwrap();
    let chat = api.create_chat().await.unwrap();
    let chat_id = chat.id.clone();

    // First message
    let first_response = api
        .complete(&chat_id, "My name is Alice.", None, false, false)
        .await
        .unwrap();
    assert!(
        first_response.message_id.is_some(),
        "First response should have message_id"
    );
    assert!(
        !first_response.content.is_empty(),
        "First response content should not be empty"
    );
    let first_message_id = first_response.message_id.unwrap();

    // Second message, referencing the first
    let second_response = api
        .complete(
            &chat_id,
            "What's my name?",
            Some(first_message_id),
            false,
            false,
        )
        .await
        .unwrap();

    assert!(
        second_response.parent_id.is_some(),
        "Second response should have parent_id"
    );
    assert!(
        !second_response.content.is_empty(),
        "Second response content should not be empty"
    );
    // The response should contain "Alice" somewhere (or at least acknowledge the name)
    println!("Second response: {}", second_response.content);
    // We can't guarantee exact phrasing, but we can assert that content length is reasonable
}

#[tokio::test]
async fn test_e2e_streaming() {
    let token = std::env::var("DEEPSEEK_TOKEN")
        .expect("DEEPSEEK_TOKEN environment variable must be set to run this test");

    let api = DeepSeekAPI::new(token).await.unwrap();
    let chat = api.create_chat().await.unwrap();
    let chat_id = chat.id.clone();

    let stream = api.complete_stream(chat_id, "Hello".to_string(), None, false, false);
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
                assert!(
                    !msg.content.is_empty(),
                    "Final message content should not be empty"
                );
                assert!(msg.message_id.is_some(), "message_id should be present");
                assert!(msg.parent_id.is_some(), "parent_id should be present");
                assert!(msg.role.is_some(), "role should be present");
                assert!(msg.inserted_at.is_some(), "inserted_at should be present");
            }
        }
    }

    assert!(
        got_content,
        "Should have received at least one content chunk"
    );
}
