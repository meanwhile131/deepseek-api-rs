//! DeepSeek API client for Rust
//!
//! This crate provides an asynchronous client for the DeepSeek chat API,
//! including Proof of Work (PoW) solving using a WebAssembly module.

mod pow_solver;
mod wasm_download;
pub mod models;

use reqwest::{Client, header};
use serde_json::{json, Value};
use std::sync::Arc;
use anyhow::{Result, anyhow, Context};
use futures_util::StreamExt;
use tokio::sync::Mutex;
use bytes::{Buf, BytesMut};

use crate::pow_solver::Challenge;

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
    pub async fn create_chat(&self) -> Result<Value> {
        let response = self.client
            .post("https://chat.deepseek.com/api/v0/chat_session/create")
            .body("{}")
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;

        response["data"]["biz_data"]
            .as_object()
            .cloned()
            .map(Value::Object)
            .ok_or_else(|| anyhow!("Failed to parse chat creation response: {:?}", response))
    }

    /// Gets information about a chat session.
    pub async fn get_chat_info(&self, chat_id: &str) -> Result<Value> {
        let url = format!(
            "https://chat.deepseek.com/api/v0/chat/history_messages?chat_session_id={}",
            chat_id
        );
        let response = self.client
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;

        if response["code"].as_i64() != Some(0) {
            anyhow::bail!(
                "Failed to get chat info: {}",
                response["msg"].as_str().unwrap_or("Unknown error")
            );
        }

        response["data"]["biz_data"]["chat_session"]
            .as_object()
            .cloned()
            .map(Value::Object)
            .ok_or_else(|| anyhow!("Failed to parse chat info response"))
    }

    /// Sets the PoW header by solving a challenge.
    async fn set_pow_header(&self) -> Result<String> {
        let challenge_response = self.client
            .post("https://chat.deepseek.com/api/v0/chat/create_pow_challenge")
            .body(POW_REQUEST)
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;

        let challenge_value = challenge_response["data"]["biz_data"]["challenge"]
            .as_object()
            .cloned()
            .ok_or_else(|| anyhow!("Failed to get challenge"))?;

        let challenge: Challenge = serde_json::from_value(Value::Object(challenge_value))?;
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

        let mut message = Value::Null;
        let mut current_property: Option<String> = None;
        let mut buffer = BytesMut::new();
        let mut finished = false;

        let mut bytes = response.bytes_stream();
        while let Some(chunk) = bytes.next().await {
            if finished {
                break;
            }
            let chunk = chunk?;
            buffer.extend_from_slice(&chunk);
            while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
                let line = buffer.split_to(pos);
                buffer.advance(1); // consume newline
                if line.is_empty() {
                    continue;
                }

                if line == &b"event: finish"[..] {
    
                    finished = true;
                    break;
                }
                if !line.starts_with(b"data: ") {
                    continue;
                }
                let data_str = &line[6..];
                let data: Value = serde_json::from_slice(data_str)?;
                if let Some(v) = data.get("v") {
                    if v.is_object() {
                        message = v.clone();
                        continue;
                    }
                    let path = data["p"].as_str().unwrap_or("");
                    if path.is_empty() {
                        // continuation
                        if let Some(ref cur) = current_property {
                            let mut data_with_path = data.clone();
                            data_with_path["p"] = Value::String(cur.to_string());
                            data_with_path["o"] = Value::String("APPEND".to_string());
                            self.handle_property_update(&mut message, &data_with_path)?;
                        }
                    } else {
                        current_property = Some(path.to_string());
                        self.handle_property_update(&mut message, &data)?;
                    }
                }
            }
        }

        serde_json::from_value(message)
            .context("Failed to parse message into Message struct")
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

            let mut message = Value::Null;
            let mut current_property: Option<String> = None;
            let mut buffer = BytesMut::new();
            let mut finished = false;

            let mut bytes = response.bytes_stream();
            while let Some(chunk) = bytes.next().await {
                if finished {
                    break;
                }
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
                        if let Ok(final_msg) = serde_json::from_value::<models::Message>(message.clone()) {
                            yield Ok(StreamChunk::Message(final_msg));
                        } else {
                            // If parsing fails, maybe yield an error? For now, just ignore.
                        }
                        finished = true;
                        break;
                    }
                    if !line.starts_with(b"data: ") {
                        continue;
                    }
                    let data: Value = match serde_json::from_slice(&line[6..]) {
                        Ok(d) => d,
                        Err(e) => {
                            yield Err(e.into());
                            return;
                        }
                    };
                    if let Some(v) = data.get("v") {
                        if v.is_object() {
                            message = v.clone();
                            continue;
                        }
                        let path = data["p"].as_str().unwrap_or("");
                        if path.is_empty() {
                            if let Some(ref cur) = current_property {
                                let mut data_with_path = data.clone();
                                data_with_path["p"] = Value::String(cur.to_string());
                                data_with_path["o"] = Value::String("APPEND".to_string());
                                if let Err(e) = this.handle_property_update(&mut message, &data_with_path) {
                                    yield Err(e);
                                    return;
                                }
                                if cur == "response/content" {
                                    if let Some(content) = v.as_str() {
                                        yield Ok(StreamChunk::Content(content.to_string()));
                                    }
                                } else if cur == "response/thinking_content" {
                                    if let Some(content) = v.as_str() {
                                        yield Ok(StreamChunk::Thinking(content.to_string()));
                                    }
                                }
                            }
                        } else {
                            current_property = Some(path.to_string());
                            if let Err(e) = this.handle_property_update(&mut message, &data) {
                                yield Err(e);
                                return;
                            }
                            if path == "response/content" {
                                if let Some(content) = v.as_str() {
                                    yield Ok(StreamChunk::Content(content.to_string()));
                                }
                            } else if path == "response/thinking_content" {
                                if let Some(content) = v.as_str() {
                                    yield Ok(StreamChunk::Thinking(content.to_string()));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn handle_property_update(&self, obj: &mut Value, update: &Value) -> Result<()> {
        let path = update["p"].as_str().ok_or_else(|| anyhow!("Missing path"))?;
        let value = &update["v"];
        let operation = update["o"].as_str().unwrap_or("SET");

        let keys: Vec<&str> = path.split('/').collect();
        let mut current = obj;
        for &key in keys.iter().take(keys.len() - 1) {
            current = current
                .as_object_mut()
                .and_then(|m| m.get_mut(key))
                .ok_or_else(|| anyhow!("Invalid path"))?;
        }
        let last_key = keys.last().unwrap();
        match operation {
            "SET" => {
                if let Some(map) = current.as_object_mut() {
                    map.insert((*last_key).to_string(), value.clone());
                } else {
                    anyhow::bail!("Cannot SET on non-object");
                }
            }
            "APPEND" => {
                if let Some(map) = current.as_object_mut() {
                    let entry = map.entry((*last_key).to_string()).or_insert(Value::String(String::new()));
                    if let (Value::String(existing), Value::String(append)) = (entry, value) {
                        *existing += append;
                    } else {
                        anyhow::bail!("APPEND only supported on strings");
                    }
                } else {
                    anyhow::bail!("Cannot APPEND on non-object");
                }
            }
            _ => anyhow::bail!("Unknown operation {}", operation),
        }
        Ok(())
    }
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