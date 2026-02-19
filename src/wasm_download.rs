//! Download and cache the DeepSeek WASM module.

use anyhow::{Context, Result};
use dirs::cache_dir;
use std::path::PathBuf;

const WASM_FILENAME: &str = "sha3_wasm_bg.7b9ca65ddd.wasm";
const WASM_URL: &str = "https://fe-static.deepseek.com/chat/static/sha3_wasm_bg.7b9ca65ddd.wasm";

/// Returns the local filesystem path to the DeepSeek WASM module.
/// Downloads the WASM file if it is not already present in the user's cache directory.
pub async fn get_wasm_path() -> Result<PathBuf> {
    let cache_dir = cache_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine cache directory"))?
        .join("deepseek");
    tokio::fs::create_dir_all(&cache_dir).await?;

    let local_path = cache_dir.join(WASM_FILENAME);

    if local_path.exists() {
        return Ok(local_path);
    }

    // Download the file
    let response = reqwest::get(WASM_URL)
        .await
        .with_context(|| format!("Failed to download WASM from {}", WASM_URL))?;

    let bytes = response
        .bytes()
        .await
        .context("Failed to read response body")?;

    tokio::fs::write(&local_path, &bytes)
        .await
        .with_context(|| format!("Failed to write WASM to {:?}", local_path))?;

    Ok(local_path)
}
