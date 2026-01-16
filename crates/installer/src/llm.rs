use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

const SERVER_PORT: u16 = 8765;
const SERVER_SCRIPT: &str = include_str!("../python/llm_server.py");

#[derive(Deserialize)]
pub struct LlmResponse {
    pub success: bool,
    #[serde(rename = "type")]
    pub response_type: Option<String>,  // "command" or "text"
    pub response: Option<String>,
    pub command: Option<String>,
    pub error: Option<String>,
}

/// A single message in the conversation
#[derive(Clone, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
struct LlmRequest {
    messages: Vec<ChatMessage>,
}

pub struct LlmServer {
    process: Child,
}

impl LlmServer {
    pub fn start(model_path: &str) -> Result<Self, String> {
        // Write the server script to a temp file
        let script_path = "/tmp/levitate_llm_server.py";
        std::fs::write(script_path, SERVER_SCRIPT)
            .map_err(|e| format!("Failed to write server script: {}", e))?;

        // Start the server
        let process = Command::new("python3")
            .arg(script_path)
            .arg("--model")
            .arg(model_path)
            .arg("--port")
            .arg(SERVER_PORT.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start LLM server: {}", e))?;

        let server = LlmServer { process };

        // Wait for server to be ready
        server.wait_for_ready()?;

        Ok(server)
    }

    fn wait_for_ready(&self) -> Result<(), String> {
        for _ in 0..60 {
            if TcpStream::connect(format!("127.0.0.1:{}", SERVER_PORT)).is_ok() {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(500));
        }
        Err("LLM server failed to start within 30 seconds".to_string())
    }

    pub fn query(&self, messages: &[ChatMessage]) -> Result<LlmResponse, String> {
        let request = LlmRequest {
            messages: messages.to_vec(),
        };
        let body = serde_json::to_string(&request).map_err(|e| e.to_string())?;

        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", SERVER_PORT))
            .map_err(|e| format!("Failed to connect to LLM server: {}", e))?;

        let http_request = format!(
            "POST /query HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );

        stream.write_all(http_request.as_bytes())
            .map_err(|e| format!("Failed to send request: {}", e))?;

        let mut response = String::new();
        stream.read_to_string(&mut response)
            .map_err(|e| format!("Failed to read response: {}", e))?;

        // Parse HTTP response
        let body_start = response.find("\r\n\r\n")
            .ok_or("Invalid HTTP response")?;
        let json_body = &response[body_start + 4..];

        serde_json::from_str(json_body)
            .map_err(|e| format!("Failed to parse response: {}\nBody: {}", e, json_body))
    }
}

impl Drop for LlmServer {
    fn drop(&mut self) {
        let _ = self.process.kill();
    }
}
