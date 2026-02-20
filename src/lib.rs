//! `DeepSeek` API client for Rust
//!
//! This crate provides an asynchronous client for the `DeepSeek` chat API,
//! including Proof of Work (`PoW`) solving using a WebAssembly module.

pub mod models;
mod pow_solver;
mod wasm_download;

use anyhow::{Context, Result};
use bytes::{Buf, BytesMut};
use futures_util::StreamExt;
use reqwest::{header, Client};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::pow_solver::Challenge;

const COMPLETION_PATH: &str = "/api/v0/chat/completion";
const CONTINUE_PATH: &str = "/api/v0/chat/continue";
const POW_REQUEST: &str = r#"{"target_path":"/api/v0/chat/completion"}"#;

/// Client for interacting with the `DeepSeek` API.
pub struct DeepSeekAPI {
    client: Client,
    pow_solver: Arc<Mutex<pow_solver::POWSolver>>,
    token: String,
}

impl DeepSeekAPI {
    /// Creates a new `DeepSeek` API client.
    /// Creates a new `DeepSeek` API client.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The authorization header cannot be built.
    /// - The HTTP client cannot be constructed.
    /// - The Proof‑of‑Work solver fails to initialize.
    pub async fn new(token: impl Into<String>) -> Result<Self> {
        let token = token.into();
        let client = Client::builder()
            .default_headers({
                let mut headers = header::HeaderMap::new();
                headers.insert(
                    header::AUTHORIZATION,
                    header::HeaderValue::from_str(&format!("Bearer {token}"))
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
        Ok(Self {
            client,
            pow_solver,
            token,
        })
    }

    /// Creates a new chat session.
    /// Creates a new chat session.
    ///
    /// # Errors
    /// Returns an error if the API request fails or the response cannot be parsed.
    pub async fn create_chat(&self) -> Result<crate::models::ChatSession> {
        #[derive(serde::Deserialize)]
        struct CreateChatResponse {
            data: CreateChatData,
        }
        #[derive(serde::Deserialize)]
        struct CreateChatData {
            biz_data: crate::models::ChatSession,
        }
        let response = self
            .client
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
    /// Gets information about a chat session.
    ///
    /// # Errors
    /// Returns an error if the API request fails, the response indicates an error,
    /// or the response cannot be parsed.
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
            "https://chat.deepseek.com/api/v0/chat/history_messages?chat_session_id={chat_id}"
        );
        let response: GetChatInfoResponse = self
            .client
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

    /// Sets the `PoW` header by solving a challenge.
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
        let challenge_response = self
            .client
            .post("https://chat.deepseek.com/api/v0/chat/create_pow_challenge")
            .body(POW_REQUEST)
            .send()
            .await?
            .error_for_status()?;
        let challenge_response_text = challenge_response.text().await?;

        let challenge_response: PowChallengeResponse =
            serde_json::from_str(&challenge_response_text)?;

        let challenge = challenge_response.data.biz_data.challenge;
        self.pow_solver.lock().await.solve_challenge(challenge)
    }

    /// Completes a chat message (non‑streaming).
    ///
    /// This method internally uses the streaming version (`complete_stream`) and
    /// collects all chunks, automatically handling any necessary continuations.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The Proof‑of‑Work challenge cannot be solved.
    /// - The API request fails or returns an error status.
    /// - The response cannot be parsed into a `Message`.
    pub async fn complete(
        &self,
        chat_id: &str,
        prompt: &str,
        parent_message_id: Option<i64>,
        search: bool,
        thinking: bool,
    ) -> Result<models::Message> {
        use futures_util::StreamExt;
        use tokio::pin;

        let stream = self.complete_stream(
            chat_id.to_string(),
            prompt.to_string(),
            parent_message_id,
            search,
            thinking,
        );
        pin!(stream);

        let mut final_message = None;
        while let Some(chunk) = stream.next().await {
            match chunk? {
                StreamChunk::Content(_) | StreamChunk::Thinking(_) => continue,
                StreamChunk::Message(msg) => {
                    final_message = Some(msg);
                    break;
                }
            }
        }

        final_message.context("No final message received")
    }

    /// Completes a chat message (streaming), yielding chunks of content or thinking.
    /// Completes a chat message (streaming), yielding chunks of content or thinking.
    ///
    /// This method automatically continues the generation if the response is incomplete,
    /// transparently issuing continuation requests until a complete message is obtained.
    ///
    /// # Errors
    /// Each yielded `Result` may contain an error if:
    /// - The Proof‑of‑Work challenge cannot be solved.
    /// - The API request fails.
    /// - The streaming response cannot be parsed.
    ///
    /// # Panics
    /// This function uses `unwrap()` internally when handling SSE lines, but only in cases
    /// where the data structure is guaranteed by the server format. If the server sends
    /// malformed data, those panics could occur.
    #[allow(clippy::too_many_lines)]
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
            // Initial request
            let pow_response = match this.set_pow_header().await {
                Ok(r) => r,
                Err(e) => {
                    yield Err(e);
                    return;
                }
            };
            let request = json!({
                "chat_session_id": chat_id.clone(),
                "prompt": prompt,
                "parent_message_id": parent_message_id,
                "ref_file_ids": [],
                "search_enabled": search,
                "thinking_enabled": thinking,
            });
            let response = match this.client
                .post(format!("https://chat.deepseek.com{COMPLETION_PATH}"))
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

            let mut current_stream = Box::pin(response_to_chunk_stream(response));
            let mut message_id_for_continuation: Option<i64> = None;

            loop {
                while let Some(chunk) = current_stream.next().await {
                    match chunk? {
                        StreamChunk::Content(c) => yield Ok(StreamChunk::Content(c)),
                        StreamChunk::Thinking(t) => yield Ok(StreamChunk::Thinking(t)),
                        StreamChunk::Message(msg) => {
                            if msg.status.as_deref() == Some("INCOMPLETE") {
                                message_id_for_continuation = msg.message_id;
                                break; // exit inner while to start continuation
                            } else {
                                yield Ok(StreamChunk::Message(msg));
                                return;
                            }
                        }
                    }
                }

                if let Some(msg_id) = message_id_for_continuation.take() {
                    // Start continuation
                    let pow_response = match this.set_pow_header().await {
                        Ok(r) => r,
                        Err(e) => {
                            yield Err(e);
                            return;
                        }
                    };
                    let request = json!({
                        "chat_session_id": chat_id.clone(),
                        "message_id": msg_id,
                        "fallback_to_resume": true,
                    });
                    let response = match this.client
                        .post(format!("https://chat.deepseek.com{CONTINUE_PATH}"))
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
                    current_stream = Box::pin(response_to_chunk_stream(response));
                    // Loop again to process this new stream
                } else {
                    // No continuation ID – should not happen, but break to be safe
                    break;
                }
            }
        }
    }

    /// Continues an incomplete message (streaming).
    ///
    /// This method is used internally by `complete_stream` for auto‑continuation,
    /// but can also be called manually if desired.
    ///
    /// # Errors
    /// Each yielded `Result` may contain an error if:
    /// - The Proof‑of‑Work challenge cannot be solved.
    /// - The API request fails.
    /// - The streaming response cannot be parsed.
    #[allow(clippy::too_many_lines)]
    pub fn continue_stream(
        &self,
        chat_id: String,
        message_id: i64,
        fallback_to_resume: bool,
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
                "message_id": message_id,
                "fallback_to_resume": fallback_to_resume,
            });
            let response = match this.client
                .post(format!("https://chat.deepseek.com{CONTINUE_PATH}"))
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

            let mut stream = Box::pin(response_to_chunk_stream(response));
            while let Some(chunk) = stream.next().await {
                yield chunk;
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

// Helper to turn an HTTP response into a stream of chunks.
fn response_to_chunk_stream(response: reqwest::Response) -> impl futures_util::Stream<Item = Result<StreamChunk>> {
    use async_stream::stream;
    stream! {
        let mut builder = crate::models::StreamingMessageBuilder::default();
        let mut current_property: Option<String> = None;
        let mut buffer = bytes::BytesMut::new();
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
                if line == b"event: finish"[..] {
                    if let Some(err) = toast_error {
                        yield Err(anyhow::anyhow!("API error: {err}"));
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
                if line == b"event: toast"[..] {
                    continue;
                }
                if !line.starts_with(b"data: ") {
                    continue;
                }
                let data_json = &line[6..];

                if let Ok(val) = serde_json::from_slice::<serde_json::Value>(data_json)
                    && val.get("type").and_then(|t| t.as_str()) == Some("error")
                        && let Some(content) = val.get("content").and_then(|c| c.as_str()) {
                            yield Err(anyhow::anyhow!("API error: {content}"));
                            return;
                        }

                let data: crate::models::StreamingUpdate = match serde_json::from_slice(data_json) {
                    Ok(d) => d,
                    Err(e) => {
                        yield Err(e.into());
                        return;
                    }
                };
                if data.v.is_none() && data.p.is_none() {
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
                    continue;
                }
                let is_new_object = data.v.as_ref().is_some_and(|v| v.is_object() && data.p.as_deref().unwrap_or("").is_empty());
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
                    if let Some(v) = data.v.as_ref()
                        && v.get("response").is_some()
                    {
                        builder = match crate::models::StreamingMessageBuilder::from_value(v.clone()) {
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
                    if let Some(ref cur) = current_property {
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
