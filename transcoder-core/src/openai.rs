use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct OpenAIChatRequest {
    pub model: String,
    pub messages: Vec<OpenAIMessage>,
    #[serde(default)]
    pub stream: bool,
    #[serde(default)]
    pub tools: Option<Vec<OpenAITool>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OpenAITool {
    pub r#type: String, // usually "function"
    pub function: OpenAIFunction,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OpenAIFunction {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct OpenAIMessage {
    pub role: String,
    pub content: Option<serde_json::Value>, 
    #[serde(default)]
    pub tool_calls: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub tool_call_id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}
