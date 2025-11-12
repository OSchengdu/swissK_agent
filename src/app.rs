use std::fs::OpenOptions;
use std::io::Write;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use ollama::{OllamaRequest, OllamaStreamResponse};
use serde_json::json;

use crate::agent::handle_agent;
use crate::utils::eval_expr;

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Text,
    Image,
    Rag,
    Agent,
}

impl Mode {
    pub fn next(&mut self) {
        *self = match self {
            Mode::Text => Mode::Image,
            Mode::Image => Mode::Rag,
            Mode::Rag => Mode::Agent,
            Mode::Agent => Mode::Text,
        };
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Mode::Text => "text",
            Mode::Image => "image",
            Mode::Rag => "rag",
            Mode::Agent => "agent",
        }
    }
}

#[derive(Clone)]
pub struct Message {
    pub input: String,
    pub output: String,
}

pub struct App {
    pub mode: Mode,
    pub session: String,
    pub input: String,
    pub history: Vec<Message>,
    pub show_history: bool,
    pub tx: Sender<Task>,
    pub rx: Receiver<String>,
    pub rag_loaded: bool,
    pub waiting: bool,
}

#[derive(Clone)]
pub enum Task {
    Query(String, Mode),
}

impl App {
    pub fn new() -> Self {
        let (tx, task_rx) = mpsc::channel();
        let (resp_tx, rx) = mpsc::channel();

        // 多线程模式：当前 std::thread + mpsc（阻塞，适合简单任务）
        // 扩展：用 tokio::spawn + mpsc::unbounded_channel() 换成异步
        // 或 rayon::ThreadPoolBuilder::new().num_threads(4).build() 线程池
        thread::spawn(move || {
            let client = reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(300))
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new());

            for task in task_rx {
                match task {
                    Task::Query(prompt, mode) => {
                        let mut result = String::new();
                        if mode == Mode::Agent {
                            // Agent 调用（扩展：添加更多工具在 agent.rs）
                            result = handle_agent(&client, &prompt);
                        } else {
                            let (model, final_prompt) = match mode {
                                Mode::Text => ("qwen3:8b".to_string(), prompt.clone()),
                                Mode::Image => {
                                    if prompt.starts_with("image:") {
                                        ("qwen-vl:8b".to_string(), prompt.clone())
                                    } else {
                                        resp_tx.send("Image mode: use 'image:/path'".to_string()).ok();
                                        continue;
                                    }
                                }
                                Mode::Rag => {
                                    if prompt.starts_with("rag:load ") {
                                        resp_tx.send("RAG doc loaded (simulated).".to_string()).ok();
                                        continue;
                                    }
                                    ("qwen3:8b".to_string(), format!("[RAG] {}", prompt))
                                }
                                _ => continue,
                            };

                            let req = json!({
                                "model": model,
                                "prompt": final_prompt,
                                "stream": true  // 流式调用方法
                            });

                            let mut has_error = false;
                            match client.post("http://127.0.0.1:11434/api/generate").json(&req).send() {
                                Ok(response) if response.status().is_success() => {
                                    if let Ok(lines) = response.bytes() {
                                        for line in lines.lines().flatten() {
                                            if let Ok(stream_resp) = serde_json::from_str::<OllamaStreamResponse>(&line) {
                                                if let Some(text) = stream_resp.response {
                                                    result.push_str(&text);
                                                }
                                                if let Some(err) = stream_resp.error {
                                                    result = format!("Error: {}", err);
                                                    has_error = true;
                                                    break;
                                                }
                                                if stream_resp.done {
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                                Ok(response) => {
                                    result = format!("HTTP Error: {}", response.status());
                                    has_error = true;
                                }
                                Err(e) => {
                                    result = format!("Request Error: {}", e);
                                    has_error = true;
                                }
                            }

                            if !has_error {
                                result = result.trim().to_string();
                            }
                        }
                        resp_tx.send(result).ok();
                    }
                }
            }
        });

        Self {
            mode: Mode::Text,
            session: "default".to_string(),
            input: String::new(),
            history: vec![],
            show_history: false,
            tx,
            rx,
            rag_loaded: false,
            waiting: false,
        }
    }

    pub fn send(&mut self) {
        if self.input.trim().is_empty() {
            return;
        }

        let input = self.input.drain(..).collect::<String>();
        let task = if input.starts_with("rag:load ") {
            self.rag_loaded = true;
            Task::Query(input.clone(), self.mode)
        } else {
            Task::Query(input.clone(), self.mode)
        };

        self.tx.send(task).ok();
        self.waiting = true;
        self.history.push(Message {
            input: input.clone(),
            output: "...".to_string(),
        });
    }

    pub fn save_to_json(&self, response: &str) {
        let filename = if self.mode == Mode::Image {
            format!("{}.media.json", self.session)
        } else {
            format!("{}.chat.json", self.session)
        };
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(filename) {
            let data = json!({
                "input": self.history.last().map(|m| &m.input).unwrap_or(&"".to_string()),
                "output": response,
                "mode": self.mode.as_str()
            });
            writeln!(f, "{}", data).ok();
        }
    }

    pub fn quote(&self, idx: usize) -> Option<&str> {
        if idx == 0 || idx > self.history.len() {
            None
        } else {
            Some(&self.history[self.history.len() - idx].output)
        }
    }
}
