use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inserted_at: Option<f64>,
    #[serde(default)]
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thinking_content: Option<String>,
}

// New structs for API responses
#[derive(Debug, Clone, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub seq_id: i64,
    pub agent: String,
    pub title: Option<String>,
    pub title_type: String,
    pub version: i64,
    pub current_message_id: Option<i64>,
    pub pinned: bool,
    pub inserted_at: f64,
    pub updated_at: f64,
}

// Streaming update from the server
#[derive(Debug, Deserialize, Clone)]
pub struct StreamingUpdate {
    #[serde(default)]
    pub p: Option<String>, // JSON pointer path (missing = empty)
    pub v: Option<serde_json::Value>, // value; None for delete?
    pub o: Option<String>, // operation (SET, APPEND, etc.)
}

// Builder that accumulates patches into a final Message
#[derive(Debug, Default)]
pub struct StreamingMessageBuilder {
    // We'll store the intermediate state as a Value for simplicity,
    // but we only use it internally. The public interface remains type-safe.
    inner: serde_json::Value,
}

impl StreamingMessageBuilder {
    pub fn from_value(v: serde_json::Value) -> Result<Self> {
        Ok(Self { inner: v })
    }

    pub fn apply_update(&mut self, update: &StreamingUpdate) -> Result<()> {
        let path = update.p.as_deref().ok_or_else(|| anyhow!("Missing path"))?;
        let value = update.v.as_ref().ok_or_else(|| anyhow!("Missing v"))?;
        let operation = update.o.as_deref().unwrap_or("SET");

        let keys: Vec<&str> = path.split('/').collect();
        let mut current = &mut self.inner;
        for &key in keys.iter().take(keys.len() - 1) {
            current = current
                .as_object_mut()
                .and_then(|m| m.get_mut(key))
                .ok_or_else(|| anyhow!("Invalid path: {}", path))?;
        }
        let last_key = keys.last().unwrap();
        match operation {
            "SET" => {
                if let Some(map) = current.as_object_mut() {
                    map.insert((*last_key).to_string(), value.clone());
                } else {
                    anyhow::bail!("Cannot SET on non-object at {}", path);
                }
            }
            "APPEND" => {
                if let Some(map) = current.as_object_mut() {
                    let entry = map.entry((*last_key).to_string()).or_insert(serde_json::Value::String(String::new()));
                    if let (serde_json::Value::String(existing), serde_json::Value::String(append)) = (entry, value) {
                        *existing += append;
                    } else {
                        anyhow::bail!("APPEND only supported on strings at {}", path);
                    }
                } else {
                    anyhow::bail!("Cannot APPEND on non-object at {}", path);
                }
            }
            _ => anyhow::bail!("Unknown operation {} at {}", operation, path),
        }
        Ok(())
    }

    pub fn build(self) -> Result<Message> {
        let response = self.inner
            .get("response")
            .ok_or_else(|| anyhow!("Missing 'response' field"))?
            .clone();
        serde_json::from_value(response).map_err(Into::into)
    }
}
