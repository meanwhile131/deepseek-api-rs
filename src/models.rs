use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Option<String>,
    pub parent_message_id: Option<String>,
    pub conversation_id: Option<String>,
    pub role: Option<String>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub response: Response,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_content: Option<String>,
}