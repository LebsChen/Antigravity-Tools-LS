use async_trait::async_trait;
use anyhow::Result;
use serde_json::json;
use crate::mappers::{ProtocolMapper, MapperChunk};
use crate::anthropic::{AnthropicMessageRequest};

pub struct AnthropicMapper;

#[async_trait]
impl ProtocolMapper for AnthropicMapper {
    type Request = AnthropicMessageRequest;

    fn get_protocol() -> String {
        "anthropic".to_string()
    }

    fn get_model(req: &Self::Request) -> &str {
        &req.model
    }

    fn build_prompt(req: &Self::Request) -> Result<String> {
        let mut prompt = String::new();
        if let Some(tools) = &req.tools {
            let unified_tools = tools.iter().map(|t| crate::tools::UnifiedToolDefinition {
                name: t.name.clone(),
                description: t.description.clone(),
                parameters: t.input_schema.clone().unwrap_or_else(|| json!({})),
            }).collect::<Vec<_>>();
            let tool_prompt = crate::tools::build_tool_system_prompt(&unified_tools);
            if !tool_prompt.is_empty() {
                prompt.push_str(&tool_prompt);
                prompt.push_str("\n\n");
                prompt.push_str("IMPORTANT: If you need to use any of the tools above, you MUST output a <tool_call> XML tag.\n");
            }
        }

        if let Some(sys) = &req.system {
            if let Some(s) = sys.as_str() { prompt.push_str(s); prompt.push_str("\n\n"); }
            else if let Some(arr) = sys.as_array() {
                for block in arr { if let Some(t) = block.get("text").and_then(|v| v.as_str()) { prompt.push_str(t); prompt.push_str("\n"); } }
                prompt.push_str("\n");
            }
        }
        for msg in &req.messages {
            prompt.push_str(&format!("{}: ", msg.role));
            if let Some(s) = msg.content.as_str() { prompt.push_str(s); }
            else if let Some(arr) = msg.content.as_array() {
                for block in arr { if let Some(t) = block.get("text").and_then(|v| v.as_str()) { prompt.push_str(t); } }
            }
            prompt.push('\n');
        }
        Ok(prompt)
    }

    fn initial_chunks() -> Vec<MapperChunk> {
        vec![
            MapperChunk {
                event: Some("message_start".into()),
                data: r#"{"type":"message_start","message":{"id":"msg_cascade","type":"message","role":"assistant","content":[]}}"#.into(),
            },
            MapperChunk {
                event: Some("content_block_start".into()),
                data: r#"{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#.into(),
            },
        ]
    }

    async fn map_delta(
        _model: &str,
        delta: String,
        is_final: bool,
        tool_call_buffer: &mut String,
        in_tool_call: &mut bool,
        tool_call_index: &mut u32,
    ) -> Result<Vec<MapperChunk>> {
        let mut results = vec![];

        if is_final {
            results.push(MapperChunk { event: Some("content_block_stop".into()), data: r#"{"type":"content_block_stop","index":0}"#.into() });
            results.push(MapperChunk { event: Some("message_delta".into()), data: r#"{"type":"message_delta","delta":{"stop_reason":"end_turn","stop_sequence":null}}"#.into() });
            results.push(MapperChunk { event: Some("message_stop".into()), data: r#"{"type":"message_stop"}"#.into() });
            return Ok(results);
        }

        let mut pending_text = delta;
        while !pending_text.is_empty() {
            if !*in_tool_call {
                if let Some(start_idx) = pending_text.find("<tool_call>") {
                    let before_text = &pending_text[..start_idx];
                    if !before_text.is_empty() {
                        let delta_json = json!({ "type": "content_block_delta", "index": 0, "delta": { "type": "text_delta", "text": before_text } });
                        results.push(MapperChunk { event: Some("content_block_delta".into()), data: delta_json.to_string() });
                    }
                    *in_tool_call = true;
                    pending_text = pending_text[start_idx + "<tool_call>".len()..].to_string();
                } else {
                    let delta_json = json!({ "type": "content_block_delta", "index": 0, "delta": { "type": "text_delta", "text": pending_text } });
                    results.push(MapperChunk { event: Some("content_block_delta".into()), data: delta_json.to_string() });
                    pending_text = String::new();
                }
            } else {
                if let Some(end_idx) = pending_text.find("</tool_call>") {
                    let inner_text = &pending_text[..end_idx];
                    tool_call_buffer.push_str(inner_text);
                    
                    // Flush tool
                    if let Ok(json_obj) = serde_json::from_str::<serde_json::Value>(tool_call_buffer.trim()) {
                        let name = json_obj.get("name").and_then(|v| v.as_str()).unwrap_or("unknown_tool").to_string();
                        let args = json_obj.get("arguments").cloned().unwrap_or_else(|| json!({}));
                        
                        results.push(MapperChunk {
                            event: Some("content_block_start".into()),
                            data: json!({ "type": "content_block_start", "index": *tool_call_index + 1, "content_block": { "type": "tool_use", "id": format!("toolu_cascade_{}", *tool_call_index), "name": name, "input": {} } }).to_string(),
                        });
                        
                        let args_str = serde_json::to_string(&args).unwrap_or_else(|_| "{}".to_string());
                        results.push(MapperChunk {
                            event: Some("content_block_delta".into()),
                            data: json!({ "type": "content_block_delta", "index": *tool_call_index + 1, "delta": { "type": "input_json_delta", "partial_json": args_str } }).to_string(),
                        });
                        
                        results.push(MapperChunk {
                            event: Some("content_block_stop".into()),
                            data: json!({ "type": "content_block_stop", "index": *tool_call_index + 1 }).to_string(),
                        });
                        *tool_call_index += 1;
                    }

                    pending_text = pending_text[end_idx + "</tool_call>".len()..].to_string();
                    *in_tool_call = false;
                    tool_call_buffer.clear();
                } else {
                    tool_call_buffer.push_str(&pending_text);
                    pending_text = String::new();
                }
            }
        }
        Ok(results)
    }
}
