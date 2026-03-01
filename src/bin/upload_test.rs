//! Example of uploading a file and then attaching it to a chat.

use deepseek_api::DeepSeekAPI;
use std::env;
use tokio::fs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let token = env::var("DEEPSEEK_TOKEN").expect("DEEPSEEK_TOKEN environment variable not set");
    let file_path = env::args().nth(1).expect("Usage: upload_test <file_path>");

    let api = DeepSeekAPI::new(token).await?;

    // Read the file
    let file_data = fs::read(&file_path).await?;
    let filename = std::path::Path::new(&file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("uploaded_file");

    // Upload the file
    println!("Uploading {} ({} bytes)...", file_path, file_data.len());
    let file_info = api.upload_file(file_data, filename, None).await?;
    println!("Upload successful!");
    println!("File info: {file_info:#?}");

    Ok(())
}