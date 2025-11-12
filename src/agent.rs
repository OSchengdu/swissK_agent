use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::json;

use crate::utils::eval_expr;

#[derive(Deserialize)]
struct ToolCall {
    name: String,
    arguments: serde_json::Value,
}

#[derive(Deserialize)]
struct AgentResponse {
    response: Option<String>,
    tool_calls: Option<Vec<ToolCall>>,
    done: bool,
}

pub fn handle_agent(client: &Client, prompt: &str) -> String {
    // 记忆：从 history 构建 messages (可扩展：用数据库存储)
    let mut messages = vec![json!({"role": "system", "content": "You are an agent."})];
    messages.push(json!({"role": "user", "content": prompt}));

    let req = json!({
        "model": "qwen3:8b",
        "messages": messages,
        "stream": true,
        "tools": [ /* 如 main 所述 */ ]
    });

    // 工具调用循环（计划：Thought -> Action -> Observation）
    let mut full_response = String::new();
    // ... (从用户提供的代码中复制 agent 处理逻辑)

    full_response
}
