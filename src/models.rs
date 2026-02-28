use anyhow::{anyhow, Result}

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
};
use serde::{Deserialize, Serialize}

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
};

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accumulated_token_usage: Option<i64>,
}

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
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

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
}

// Streaming update from the server
#[derive(Debug, Deserialize, Clone)]
pub struct StreamingUpdate {
    #[serde(default)]
    pub p: Option<String>, // JSON pointer path (missing = empty)
    pub v: Option<serde_json::Value>, // value; None for delete?
    pub o: Option<String>,            // operation (SET, APPEND, etc.)
}

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
}

// Builder that accumulates patches into a final Message
#[derive(Debug)]
pub struct StreamingMessageBuilder {
    // We'll store the intermediate state as a Value for simplicity,
    // but we only use it internally. The public interface remains type-safe.
    inner: serde_json::Value,
}

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
}

impl Default for StreamingMessageBuilder {
    fn default() -> Self {
        Self {
            inner: serde_json::json!({}

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
}),
        }

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
}
    }

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
}
}

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
}

impl StreamingMessageBuilder {
    /// Creates a new `StreamingMessageBuilder` from a JSON value.
    ///
    /// # Errors
    /// Returns an error if the provided value cannot be interpreted as a valid builder state.
    /// (Currently always returns `Ok`.)
    pub fn from_value(v: serde_json::Value) -> Result<Self> {
        Ok(Self { inner: v }

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
})
    }

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
}

    /// Applies a streaming update to the builder.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The path is empty or invalid.
    /// - The operation is unknown.
    /// - An `APPEND` operation is used on a non‑string field.
    /// - Internal pointer manipulations fail.
    pub fn apply_update(&mut self, update: &StreamingUpdate) -> Result<()> {
        let path = update.p.as_deref().ok_or_else(|| anyhow!("Missing path"))?;
        let value = update.v.as_ref().ok_or_else(|| anyhow!("Missing v"))?;
        let operation = update.o.as_deref().unwrap_or("SET");

        let keys: Vec<&str> = path.split('/').collect();
        if keys.is_empty() {
            anyhow::bail!("Empty path");
        }

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
}

        let mut current = &mut self.inner;
        // Navigate to the parent of the target key
        for &key in keys.iter().take(keys.len() - 1) {
            if !current.is_object() {
                *current = serde_json::json!({}

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
});
            }

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
}
            // Get or create the next level
            current = match current.as_object_mut() {
                Some(obj) => obj
                    .entry(key.to_string())
                    .or_insert_with(|| serde_json::json!({}

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
})),
                None => anyhow::bail!("Expected object at intermediate path segment"),
            }

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
};
        }

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
}

        let last_key = keys.last().ok_or_else(|| anyhow!("Empty path"))?;
        if !current.is_object() {
            *current = serde_json::json!({}

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
});
        }

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
}
        let current_obj = current
            .as_object_mut()
            .ok_or_else(|| anyhow!("Expected object at target path"))?;

        match operation {
            "SET" => {
                current_obj.insert((*last_key).to_string(), value.clone());
            }

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
}
            "APPEND" => {
                let entry = current_obj
                    .entry((*last_key).to_string())
                    .or_insert_with(|| serde_json::Value::String(String::new()));
                if let (serde_json::Value::String(existing), serde_json::Value::String(append)) =
                    (entry, value)
                {
                    existing.push_str(append);
                }

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
} else {
                    anyhow::bail!("APPEND only supported on strings at {path}

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
}");
                }

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
}
            }

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
}
            _ => anyhow::bail!("Unknown operation {operation}

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
} at {path}

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
}"),
        }

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
}
        Ok(())
    }

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
}

    /// Builds the final `Message` from the accumulated patches.
    ///
    /// # Errors
    /// Returns an error if the accumulated state cannot be deserialized into a `Message`.
    pub fn build(self) -> Result<Message> {
        if let Some(response) = self.inner.get("response") {
            serde_json::from_value(response.clone()).map_err(Into::into)
        }

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
} else {
            // Try to deserialize the whole object as Message
            serde_json::from_value(self.inner).map_err(Into::into)
        }

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
}
    }

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
}
}

/// Information about an uploaded file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub previewable: bool,
    pub file_size: i64,
    pub token_usage: Option<i64>,
    pub error_code: Option<String>,
    pub inserted_at: f64,
    pub updated_at: f64,
}
