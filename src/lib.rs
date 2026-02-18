//! DeepSeek API client for Rust
//!
//! This crate provides an asynchronous client for the DeepSeek chat API,
//! including Proof of Work (PoW) solving using a WebAssembly module.

mod pow_solver;
mod wasm_download;
pub mod models;

use reqwest::{Client, header};
use serde_json::json;
use std::sync::Arc;
use anyhow::{Result, Context};
use futures_util::StreamExt;
use tokio::sync::Mutex;
use bytes::{Buf, BytesMut};

use crate::pow_solver::Challenge;
use crate::models::StreamingMessageBuilder;

const COMPLETION_PATH: &str = "/api/v0/chat/completion";
const POW_REQUEST: &str = r#"{"target_path":"/api/v0/chat/completion"}"#;

/// Client for interacting with the DeepSeek API.
pub struct DeepSeekAPI {
    client: Client,
    pow_solver: Arc<Mutex<pow_solver::POWSolver>>,
    token: String,
}

impl DeepSeekAPI {
    /// Creates a new DeepSeek API client.
    pub async fn new(token: impl Into<String>) -> Result<Self> {
        let token = token.into();
        let client = Client::builder()
            .default_headers({
                let mut headers = header::HeaderMap::new();
                headers.insert(
                    header::AUTHORIZATION,
                    header::HeaderValue::from_str(&format!("Bearer {}", token))
                        .context("Invalid authorization header")?,
                );
                headers.insert(
                    header::CONTENT_TYPE,
                    header::HeaderValue::from_static("application/json"),
                );
                headers
            })
            .build()?;

        let pow_solver = Arc::new(Mutex::new(pow_solver::POWSolver::new().await?));
        Ok(Self { client, pow_solver, token })
    }

    /// Creates a new chat session.
    pub async fn create_chat(&self) -> Result<crate::models::ChatSession> {
        #[derive(serde::Deserialize)]
        struct CreateChatResponse {
            data: CreateChatData,
        }
        #[derive(serde::Deserialize)]
        struct CreateChatData {
            biz_data: crate::models::ChatSession,
        }
        let response = self.client
            .post("https://chat.deepseek.com/api/v0/chat_session/create")
            .body("{}")
            .send()
            .await?
            .error_for_status()?;
        let response_text = response.text().await?;
        let response: CreateChatResponse = serde_json::from_str(&response_text)?;
        Ok(response.data.biz_data)
    }

    /// Gets information about a chat session.
    pub async fn get_chat_info(&self, chat_id: &str) -> Result<crate::models::ChatSession> {
        #[derive(serde::Deserialize)]
        struct GetChatInfoResponse {
            code: i64,
            msg: String,
            data: GetChatInfoData,
        }
        #[derive(serde::Deserialize)]
        struct GetChatInfoData {
            biz_data: GetChatInfoBizData,
        }
        #[derive(serde::Deserialize)]
        struct GetChatInfoBizData {
            chat_session: crate::models::ChatSession,
        }
        let url = format!(
            "https://chat.deepseek.com/api/v0/chat/history_messages?chat_session_id={}",
            chat_id
        );
        let response: GetChatInfoResponse = self.client
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        if response.code != 0 {
            anyhow::bail!("Failed to get chat info: {}", response.msg);
        }

        Ok(response.data.biz_data.chat_session)
    }

    /// Sets the PoW header by solving a challenge.
    async fn set_pow_header(&self) -> Result<String> {
        #[derive(serde::Deserialize)]
        struct PowChallengeResponse {
            data: PowChallengeData,
        }
        #[derive(serde::Deserialize)]
        struct PowChallengeData {
            biz_data: PowChallengeBizData,
        }
        #[derive(serde::Deserialize)]
        struct PowChallengeBizData {
            challenge: Challenge,
        }
        let challenge_response = self.client
            .post("https://chat.deepseek.com/api/v0/chat/create_pow_challenge")
            .body(POW_REQUEST)
            .send()
            .await?
            .error_for_status()?;
        let challenge_response_text = challenge_response.text().await?;

        let challenge_response: PowChallengeResponse = serde_json::from_str(&challenge_response_text)?;

        let challenge = challenge_response.data.biz_data.challenge;
        self.pow_solver.lock().await.solve_challenge(challenge)
    }

    /// Completes a chat message (non-streaming).
    pub async fn complete(
        &self,
        chat_id: &str,
        prompt: &str,
        parent_message_id: Option<i64>,
        search: bool,
        thinking: bool,
    ) -> Result<models::Message> {
        let pow_response = self.set_pow_header().await?;
        let client = self.client.clone();
        let request = json!({
            "chat_session_id": chat_id,
            "prompt": prompt,
            "parent_message_id": parent_message_id,
            "ref_file_ids": [],
            "search_enabled": search,
            "thinking_enabled": thinking,
        });

        let response = client
            .post(format!("https://chat.deepseek.com{}", COMPLETION_PATH))
            .header("x-ds-pow-response", &pow_response)
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        // Get the full response body as text
        let response_text = response.text().await?;


        // If the response contains "data: " lines, treat as SSE; otherwise treat as single JSON
        if response_text.contains("\ndata: ") || response_text.starts_with("data: ") {
            // SSE response
            let mut builder = crate::models::StreamingMessageBuilder::default();
            let mut current_property: Option<String> = None;
            let mut finished = false;
            let mut toast_error: Option<String> = None;

            let lines: Vec<&str> = response_text.lines().collect();
            let mut i = 0;
            while i < lines.len() {
                let line = lines[i];
                i += 1;
                if line.is_empty() {
                    continue;
                }
                if line == "event: finish" {
                    finished = true;
                    break;
                }
                if line == "event: toast" {
                    // Next line should be data: ...
                    if i < lines.len() {
                        let data_line = lines[i];
                        i += 1;
                        if data_line.starts_with("data: ") {
                            let toast_data = &data_line[6..];
                            // Try to parse error message
                            if let Ok(toast) = serde_json::from_str::<serde_json::Value>(toast_data) {
                                let msg = toast["content"].as_str().unwrap_or("Unknown error");
                                toast_error = Some(msg.to_string());
                            } else {
                                toast_error = Some("Unknown toast error".to_string());
                            }
                        }
                    }
                    continue;
                }
                if !line.starts_with("data: ") {
                    continue;
                }
                let data_str = &line[6..];

                let data: crate::models::StreamingUpdate = serde_json::from_str(data_str)?;
                // Handle case where the entire data is a plain JSON object (not a patch)
                if data.v.is_none() && data.p.is_none() {
                    // Try to interpret the whole data as a message object
                    let full_value: serde_json::Value = serde_json::from_str(data_str)?;
                    if full_value.get("response").is_some() {
                        builder = crate::models::StreamingMessageBuilder::from_value(full_value)?;
                    }
                    // Otherwise ignore (likely metadata)
                    continue;
                }
                // Determine if this is a new object and get path before borrowing data
                let is_new_object = data.v.as_ref().map_or(false, |v| v.is_object() && data.p.as_deref().unwrap_or("").is_empty());
                let path = data.p.clone().unwrap_or_default();
                if is_new_object {
                    // New object (initial state) - only use if it contains a "response" field
                    if data.v.as_ref().and_then(|v| v.get("response")).is_some() {
                        builder = crate::models::StreamingMessageBuilder::from_value(data.v.unwrap().clone())?;
                    }
                    continue;
                }
                if path.is_empty() {
                    // continuation of previous path
                    if let Some(ref cur) = current_property {
                        let mut update = data;
                        update.p = Some(cur.clone());
                        update.o = Some("APPEND".to_string());
                        builder.apply_update(&update)?;
                    }
                } else {
                    current_property = Some(path.clone());
                    builder.apply_update(&data)?;
                }
            }

            if let Some(err) = toast_error {
                anyhow::bail!("API error: {}", err);
            }


            builder.build().context("Failed to build final message")
        } else {
            // Single JSON response

            let value: serde_json::Value = serde_json::from_str(&response_text)?;
            // Check for API error response (has code field non-zero)
            if let Some(code) = value.get("code").and_then(|c| c.as_i64()) {
                if code != 0 {
                    let msg = value["msg"].as_str().unwrap_or("Unknown error");
                    anyhow::bail!("API error (code {}): {}", code, msg);
                }
            }
            // Try to extract message from "response" field if present, otherwise assume the whole object is the message
            if let Some(response_obj) = value.get("response") {
                serde_json::from_value(response_obj.clone()).map_err(Into::into)
            } else {
                serde_json::from_value(value).map_err(Into::into)
            }
        }
    }

    /// Completes a chat message (streaming), yielding chunks of content or thinking.
    pub fn complete_stream(
        &self,
        chat_id: String,
        prompt: String,
        parent_message_id: Option<i64>,
        search: bool,
        thinking: bool,
    ) -> impl futures_util::Stream<Item = Result<StreamChunk>> + '_ {
        use async_stream::stream;

        let this = self.clone();
        stream! {
            let pow_response = match this.set_pow_header().await {
                Ok(r) => r,
                Err(e) => {
                    yield Err(e);
                    return;
                }
            };
            let request = json!({
                "chat_session_id": chat_id,
                "prompt": prompt,
                "parent_message_id": parent_message_id,
                "ref_file_ids": [],
                "search_enabled": search,
                "thinking_enabled": thinking,
            });
            let response = match this.client
                .post(format!("https://chat.deepseek.com{}", COMPLETION_PATH))
                .header("x-ds-pow-response", &pow_response)
                .json(&request)
                .send()
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    yield Err(e.into());
                    return;
                }
            };
            let response = match response.error_for_status() {
                Ok(r) => r,
                Err(e) => {
                    yield Err(e.into());
                    return;
                }
            };

            let mut builder = crate::models::StreamingMessageBuilder::default();
            let mut current_property: Option<String> = None;
            let mut buffer = BytesMut::new();
            let mut toast_error: Option<String> = None;

            let mut bytes = response.bytes_stream();
            while let Some(chunk) = bytes.next().await {
                let chunk = match chunk {
                    Ok(c) => c,
                    Err(e) => {
                        yield Err(e.into());
                        return;
                    }
                };
                buffer.extend_from_slice(&chunk);
                while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
                    let line = buffer.split_to(pos);
                    buffer.advance(1); // consume newline
                    if line.is_empty() {
                        continue;
                    }
                    if line == &b"event: finish"[..] {
                        // Build final message and yield it, then exit the stream
                        if let Some(err) = toast_error {
                            yield Err(anyhow::anyhow!("API error: {}", err));
                            return;
                        }

                        match builder.build() {
                            Ok(final_msg) => {
                                yield Ok(StreamChunk::Message(final_msg));
                                return;
                            }
                            Err(e) => {
                                yield Err(e);
                                return;
                            }
                        }
                    }
                    if line == &b"event: toast"[..] {
                        // The next line should be a data line containing the error details.
                        // Continue to the next iteration; we'll handle the data line in the next loop.
                        continue;
                    }
                    if !line.starts_with(b"data: ") {
                        continue;
                    }
                    let data_json = &line[6..];

                    
                    // If we previously saw a toast event, this data line should contain the error.
                    // But we don't have a flag. Instead, we'll check if the data looks like an error.
                    // We'll parse it generically.
                    if let Ok(val) = serde_json::from_slice::<serde_json::Value>(data_json) {
                        if val.get("type").and_then(|t| t.as_str()) == Some("error") {
                            if let Some(content) = val.get("content").and_then(|c| c.as_str()) {
                                yield Err(anyhow::anyhow!("API error: {}", content));
                                return;
                            }
                        }
                    }

                    let data: crate::models::StreamingUpdate = match serde_json::from_slice(data_json) {
                        Ok(d) => d,
                        Err(e) => {
                            yield Err(e.into());
                            return;
                        }
                    };
                    // Handle case where the entire data is a plain JSON object (not a patch)
                    if data.v.is_none() && data.p.is_none() {
                        // Try to interpret the whole data as a message object
                        let full_value: serde_json::Value = match serde_json::from_slice(data_json) {
                            Ok(v) => v,
                            Err(e) => {
                                yield Err(e.into());
                                return;
                            }
                        };
                        if full_value.get("response").is_some() {
                            builder = match crate::models::StreamingMessageBuilder::from_value(full_value) {
                                Ok(b) => b,
                                Err(e) => {
                                    yield Err(e);
                                    return;
                                }
                            };
                        }
                        // Otherwise ignore (likely metadata)
                        continue;
                    }
                    // Extract necessary information without holding a reference across moves
                    let is_new_object = data.v.as_ref().map_or(false, |v| v.is_object() && data.p.as_deref().unwrap_or("").is_empty());
                    let path = data.p.clone().unwrap_or_default();
                    let content_to_yield = if !is_new_object && !path.is_empty() {
                        if path == "response/content" {
                            data.v.as_ref().and_then(|v| v.as_str().map(|s| StreamChunk::Content(s.to_string())))
                        } else if path == "response/thinking_content" {
                            data.v.as_ref().and_then(|v| v.as_str().map(|s| StreamChunk::Thinking(s.to_string())))
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    if is_new_object {
                        // New object (initial state) - only use if it contains a "response" field
                        if data.v.as_ref().and_then(|v| v.get("response")).is_some() {
                            builder = match crate::models::StreamingMessageBuilder::from_value(data.v.unwrap().clone()) {
                                Ok(b) => b,
                                Err(e) => {
                                    yield Err(e);
                                    return;
                                }
                            };
                        }
                        continue;
                    }

                    if path.is_empty() {
                        // continuation of previous path
                        if let Some(ref cur) = current_property {
                            // Determine content to yield before moving data
                            let continuation_content = if cur == "response/content" {
                                data.v.as_ref().and_then(|v| v.as_str().map(|s| StreamChunk::Content(s.to_string())))
                            } else if cur == "response/thinking_content" {
                                data.v.as_ref().and_then(|v| v.as_str().map(|s| StreamChunk::Thinking(s.to_string())))
                            } else {
                                None
                            };
                            let mut update = data.clone();
                            update.p = Some(cur.clone());
                            update.o = Some("APPEND".to_string());
                            if let Err(e) = builder.apply_update(&update) {
                                yield Err(e);
                                return;
                            }
                            if let Some(chunk) = continuation_content {
                                yield Ok(chunk);
                            }
                        }
                    } else {
                        current_property = Some(path.clone());
                        if let Err(e) = builder.apply_update(&data) {
                            yield Err(e);
                            return;
                        }
                        if let Some(chunk) = content_to_yield {
                            yield Ok(chunk);
                        }
                    }
                }
            }
        }
    }

    // Removed handle_property_update; logic moved to StreamingMessageBuilder
}

/// Represents a chunk from the streaming response.
#[derive(Debug)]
pub enum StreamChunk {
    Content(String),
    Thinking(String),
    Message(models::Message),
}

impl Clone for DeepSeekAPI {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            pow_solver: Arc::clone(&self.pow_solver),
            token: self.token.clone(),
        }
    }
}