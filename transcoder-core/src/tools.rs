use serde_json::Value;

/// 代表统一的通用外部工具定义
#[derive(Debug, Clone)]
pub struct UnifiedToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

/// 将一组第三方工具定义转换为能被预设底层文本补全利用的 System Prompt (XML)
pub fn build_tool_system_prompt(tools: &[UnifiedToolDefinition]) -> String {
    if tools.is_empty() {
        return String::new();
    }

    let mut prompt = String::from(
        "You are an AI assistant that has access to the following tools.\n\
        To use a tool, you MUST output a `<tool_call>` XML tag containing a JSON object with `name` and `arguments` fields.\n\n\
        <tools>\n"
    );

    for tool in tools {
        prompt.push_str("  <tool>\n");
        prompt.push_str(&format!("    <name>{}</name>\n", tool.name));
        prompt.push_str(&format!("    <description>{}</description>\n", tool.description));
        let params_json = serde_json::to_string(&tool.parameters).unwrap_or_else(|_| "{}".to_string());
        prompt.push_str(&format!("    <parameters>{}</parameters>\n", params_json));
        prompt.push_str("  </tool>\n");
    }

    prompt.push_str(
        "</tools>\n\n\
        Usage Rule: If you need to use a tool, STOP generating text for the user immediately and output exactly:\n\
        <tool_call>\n\
        {\"name\": \"tool_name\", \"arguments\": {\"param1\": \"value1\"}}\n\
        </tool_call>\n\
        \nAfter outputting the tool call, simply halt your response.\n\n"
    );

    prompt
}
