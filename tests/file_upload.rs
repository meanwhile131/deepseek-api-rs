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

    // Decode the PNG using the stable base64 API
    let file_data = {
        use base64::engine::general_purpose::STANDARD;
        use base64::Engine;
        STANDARD.decode(TINY_PNG_BASE64)?
    };
    let filename = "tiny.png";

    // Upload the file
    let file_info = api.upload_file(file_data, filename, None).await?;
    println!("Uploaded file: {:?}", file_info);

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
            _ => continue,
        }
    }

    let processed = processed.expect("File processing timed out after 4 minutes");
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