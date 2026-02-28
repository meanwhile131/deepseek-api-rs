use anyhow::Result;
use deepseek_api::{DeepSeekAPI, StreamChunk};
use futures_util::StreamExt;
use std::env;
use std::time::Duration;
use tokio::pin;

#[tokio::test]
async fn test_file_upload_and_use() -> Result<()> {
    let token = env::var("DEEPSEEK_TOKEN")
        .expect("DEEPSEEK_TOKEN environment variable must be set to run this test");

    let api = DeepSeekAPI::new(token).await?;
    let chat = api.create_chat().await?;
    let chat_id = chat.id.as_str();

    // Create a simple text file content
    let file_content = "Hello, this is a test file.\nIt contains two lines.";
    let file_data = file_content.as_bytes().to_vec();
    let filename = "test.txt";

    // Upload the file
    let file_info = api.upload_file(file_data, filename, Some("text/plain")).await?;
    println!("Uploaded file: {file_info:?}");

    // Manually poll for file processing status with debug output (allow up to 4 minutes)
    let max_attempts = 120;
    let delay = Duration::from_secs(2);
    let mut processed = None;

    for attempt in 0..max_attempts {
        tokio::time::sleep(delay).await;
        let info = api.fetch_file_info(&file_info.id).await?;
        println!("Attempt {}: file status = {:?}, error_code = {:?}", attempt, info.status, info.error_code);
        match info.status.as_str() {
            "SUCCESS" => {
                processed = Some(info);
                break;
            }
            "ERROR" => anyhow::bail!("File processing error: {:?}", info.error_code),
            _ => (),
        }
    }

    let processed = processed.expect("File processing timed out after 4 minutes");
    println!("Processed file: {processed:?}");

    assert_eq!(processed.status, "SUCCESS");
    assert_eq!(processed.file_name, filename);
    assert!(processed.token_usage.is_some());

    // Now use the file in a completion, asking the model to read the file content
    let prompt = "What is the content of the uploaded file?";
    let response = api
        .complete(chat_id, prompt, None, false, true, vec![processed.id.clone()])
        .await?;

    println!("Response: {}", response.content);
    assert!(!response.content.is_empty());
    // Check that the response contains the expected text (or at least part of it)
    assert!(
        response.content.contains("Hello, this is a test file") || 
        response.content.contains("two lines"),
        "Response should mention the file content"
    );

    // Optionally, test streaming with the file
    let stream = api.complete_stream(
        chat_id.to_string(),
        prompt.to_string(),
        None,
        false,
        true,
        vec![processed.id],
    );
    pin!(stream);
    let mut got_content = false;
    let mut full_response = String::new();
    while let Some(chunk) = stream.next().await {
        match chunk? {
            StreamChunk::Content(c) => {
                println!("Content chunk: {c}");
                full_response.push_str(&c);
                got_content = true;
            }
            StreamChunk::Thinking(t) => println!("Thinking: {t}"),
            StreamChunk::Message(msg) => {
                println!("Final message: {msg:?}");
                assert!(!msg.content.is_empty());
            }
        }
    }
    assert!(got_content, "Should have received content");
    assert!(
        full_response.contains("Hello, this is a test file") ||
        full_response.contains("two lines"),
        "Streamed response should mention the file content"
    );

    Ok(())
}