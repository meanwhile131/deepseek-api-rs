//! End-to-end tests for the `DeepSeek` API client.
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
        .complete(chat_id, "Hello", None, false, false, vec![])
        .await
        .unwrap();

    assert!(
        !response.content.is_empty(),
        "Response content should not be empty"
    );
    println!("Completion response: {response:#?}");
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
            vec![],
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
            vec![],
        )
        .await
        .unwrap();

    assert!(
        !response.content.is_empty(),
        "Response content should not be empty"
    );
    // thinking_content may or may not be present depending on the model
    if let Some(thinking) = &response.thinking_content {
        println!("Thinking content: {thinking}");
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
            vec![],
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
        .complete(&chat_id, "My name is Alice.", None, false, false, vec![])
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
            vec![],
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

    let stream = api.complete_stream(chat_id, "Hello".to_string(), None, false, false, vec![]);
    pin_mut!(stream); // pin the stream so we can call .next()

    let mut got_content = false;
    while let Some(chunk) = stream.next().await {
        match chunk.unwrap() {
            StreamChunk::Content(content) => {
                got_content = true;
                println!("Content: {content}");
            }
            StreamChunk::Thinking(thought) => {
                println!("Thinking: {thought}");
            }
            StreamChunk::Message(msg) => {
                println!("Final message: {msg:#?}");
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

#[tokio::test]
async fn test_second_response_first_thinking_token() {
    let token = std::env::var("DEEPSEEK_TOKEN")
        .expect("DEEPSEEK_TOKEN environment variable must be set to run this test");

    let api = DeepSeekAPI::new(token).await.unwrap();
    let chat = api.create_chat().await.unwrap();
    let chat_id = chat.id.clone();

    // First message without thinking to establish conversation
    let first_response = api
        .complete(&chat_id, "Hello, my name is TestUser.", None, false, false, vec![])
        .await
        .unwrap();
    let first_message_id = first_response.message_id.expect("first response should have message_id");

    // Second message with thinking enabled
    let stream = api.complete_stream(
        chat_id.clone(),
        "What is my name? Please think step by step before answering.".to_string(),
        Some(first_message_id),
        false,
        true, // thinking enabled
        vec![],
    );
    pin_mut!(stream);

    let mut thinking_chunks_received = Vec::new();
    let mut content_chunks_received = Vec::new();
    let mut final_message = None;
    let mut first_thinking_chunk = None;

    while let Some(chunk) = stream.next().await {
        match chunk.unwrap() {
            StreamChunk::Thinking(thought) => {
                if first_thinking_chunk.is_none() {
                    first_thinking_chunk = Some(thought.clone());
                }
                thinking_chunks_received.push(thought);
            }
            StreamChunk::Content(content) => {
                content_chunks_received.push(content);
            }
            StreamChunk::Message(msg) => {
                println!("Final message received");
                final_message = Some(msg);
                break;
            }
        }
    }

    // Verify that we received at least one thinking chunk and at least one content chunk
    assert!(
        !thinking_chunks_received.is_empty(),
        "Expected at least one thinking chunk for the second response"
    );
    assert!(
        !content_chunks_received.is_empty(),
        "Expected at least one content chunk for the second response"
    );

    // Build the full thinking content from all chunks to check the beginning.
    let full_thinking: String = thinking_chunks_received.concat();
    let trimmed_start = full_thinking.trim_start();
    let first_non_whitespace_char = trimmed_start.chars().next();

    assert!(
        first_non_whitespace_char.is_some(),
        "Full thinking content is empty or only whitespace"
    );

    // For the reported issue, the first token was missing, causing the output to start with a space.
    // So we ensure the full thinking does not start with whitespace.
    assert!(
        !full_thinking.starts_with(char::is_whitespace),
        "Full thinking content should not start with whitespace, but it starts with: {:?}",
        &full_thinking.chars().take(10).collect::<String>()
    );

    // Also check the first chunk specifically – if it starts with whitespace, that's a problem.
    if let Some(first_chunk) = first_thinking_chunk {
        let first_char = first_chunk.chars().next();
        assert!(
            first_char.is_some() && !first_char.unwrap().is_whitespace(),
            "First thinking chunk should start with a non-whitespace character, but got: {:?}",
            first_chunk
        );
    } else {
        panic!("No thinking chunk received");
    }

    // The final message should be present
    let final_msg = final_message.expect("No final message received");
    assert!(
        !final_msg.content.is_empty(),
        "Final message content should not be empty"
    );
    // The parent_id may be either the first message ID or an intermediate assistant message ID;
    // we only require that it exists and that the conversation is coherent.
    assert!(
        final_msg.parent_id.is_some(),
        "Final message should have a parent_id"
    );
    assert!(final_msg.message_id.is_some());
}
