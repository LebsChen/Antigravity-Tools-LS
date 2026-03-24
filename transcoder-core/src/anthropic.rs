use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AnthropicMessageRequest {
    pub model: String,
    pub messages: Vec<AnthropicMessage>,
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub system: Option<serde_json::Value>, 
    #[serde(default)]
    pub stream: bool,
    #[serde(default)]
    pub tools: Option<Vec<AnthropicTool>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AnthropicTool {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub input_schema: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct AnthropicMessage {
    pub role: String,
    pub content: serde_json::Value,
}

pub struct AnthropicStreamEvent {
    pub event: String,
    pub data: String,
}
