use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct OllamaRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
}

#[derive(Deserialize)]
pub struct OllamaStreamResponse {
    pub response: Option<String>,
    pub done: bool,
    pub error: Option<String>,
}

// 调用方法示例：post_generate (可扩展：加重试、认证)
pub fn post_generate(client: &Client, req: &OllamaRequest) -> String {
    // 当前阻塞调用
    // 扩展异步：use tokio; async fn post_generate_async(...) { client.post(...).await }
    "Stub for API call".to_string()  // 实际逻辑移到 app.rs 或 agent.rs
}
