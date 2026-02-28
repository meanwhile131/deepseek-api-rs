use anyhow::Result;
use deepseek_api::{DeepSeekAPI, StreamChunk};
use futures_util::StreamExt;
use std::env;
use std::time::Duration;
use tokio::pin;

// A 1x1 pixel PNG (base64 encoded)
const TINY_PNG_BASE64: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAACklEQVR4nGMAAQAABQABDQottAAAAABJRU5ErkJggg==";

#[tokio::test]
async fn test_file_upload_and_use() -> Result<()> {
    let token = env::var("DEEPSEEK_TOKEN")
        .expect("DEEPSEEK_TOKEN environment variable must be set to run this test");

    let api = DeepSeekAPI::new(token).await?;
    let chat = api.create_chat().await?;
    let chat_id = chat.id.as_str();

    // Decode the PNG
    let file_data = base64::decode(TINY_PNG_BASE64)?;
    let filename = "tiny.png";

    // Upload the file
    let file_info = api.upload_file(file_data, filename, None).await?;
    println!("Uploaded file: {:?}", file_info);

    // Wait for processing
    let processed = api
        .wait_for_file_processing(&file_info.id, 10, Duration::from_millis(500))
        .await?;
    println!("Processed file: {:?}", processed);

    assert_eq!(processed.status, "SUCCESS");
    assert_eq!(processed.file_name, filename);
    assert!(processed.token_usage.is_some());

    // Now use the file in a completion
    let prompt = "What is shown in this image?";
    let response = api
        .complete(chat_id, prompt, None, false, true, vec![processed.id.clone()])
        .await?;

    println!("Response: {}", response.content);
    assert!(!response.content.is_empty());

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
    while let Some(chunk) = stream.next().await {
        match chunk? {
            StreamChunk::Content(c) => {
                println!("Content chunk: {}", c);
                got_content = true;
            }
            StreamChunk::Thinking(t) => println!("Thinking: {}", t),
            StreamChunk::Message(msg) => {
                println!("Final message: {:?}", msg);
                assert!(!msg.content.is_empty());
            }
        }
    }
    assert!(got_content, "Should have received content");

    Ok(())
}