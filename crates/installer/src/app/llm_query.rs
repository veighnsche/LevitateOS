use std::sync::Arc;
use std::thread;

use super::App;
use crate::llm::ChatMessage;
use crate::types::*;

impl App {
    pub fn submit_input(&mut self) {
        if self.input.trim().is_empty() || self.waiting_for_llm {
            return;
        }

        // Handle slash commands locally
        if self.input.trim().starts_with('/') {
            let cmd = std::mem::take(&mut self.input);
            self.handle_slash_command(&cmd);
            return;
        }

        let user_text = std::mem::take(&mut self.input);

        // Add user message
        self.messages.push(Message {
            role: Role::User,
            text: user_text.clone(),
        });

        // Add thinking placeholder
        self.messages.push(Message {
            role: Role::Assistant,
            text: "_Thinking..._".to_string(),
        });

        self.waiting_for_llm = true;
        self.chat_scroll = usize::MAX;

        // Convert history to ChatMessage format
        let chat_messages: Vec<ChatMessage> = self.messages
            .iter()
            .filter(|m| m.text != "_Thinking..._")
            .map(|m| ChatMessage {
                role: match m.role {
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                },
                content: m.text.clone(),
            })
            .collect();

        // Query LLM in background thread
        let llm = Arc::clone(&self.llm);
        let tx = self.llm_tx.clone();

        thread::spawn(move || {
            let result = match llm.query(&chat_messages) {
                Ok(resp) => Self::parse_llm_response(resp),
                Err(e) => LlmResult::Error(format!("Error: {}", e)),
            };
            let _ = tx.send(result);
        });
    }

    fn parse_llm_response(resp: crate::llm::LlmResponse) -> LlmResult {
        if !resp.success {
            return LlmResult::Error(resp.error.unwrap_or_else(|| "Unknown error".to_string()));
        }

        if resp.response_type.as_deref() == Some("command") {
            if let Some(cmd) = resp.command {
                return LlmResult::Command(cmd);
            }
        }

        LlmResult::Text(resp.response.unwrap_or_else(|| "No response".to_string()))
    }

    pub fn check_llm_response(&mut self) {
        let Ok(result) = self.llm_rx.try_recv() else {
            return;
        };

        self.waiting_for_llm = false;

        match result {
            LlmResult::Text(text) => {
                self.update_thinking_message(text);
            }
            LlmResult::Command(cmd) => {
                self.show_command_confirmation(&cmd);
                self.pending_command = Some(PendingCommand::new(cmd));
            }
            LlmResult::Error(err) => {
                self.update_thinking_message(format!("**Error:** {}", err));
            }
        }

        self.chat_scroll = usize::MAX;
    }

    fn update_thinking_message(&mut self, new_text: String) {
        if let Some(msg) = self.messages.last_mut() {
            if msg.role == Role::Assistant && msg.text == "_Thinking..._" {
                msg.text = new_text;
            }
        }
    }

    fn show_command_confirmation(&mut self, cmd: &str) {
        let is_destructive = PendingCommand::check_destructive(cmd);

        let text = if is_destructive {
            format!(
                "⚠️ **Destructive Action**\n\n```\n$ {}\n```\n\n\
                This action **CANNOT be undone**. Type `yes` and press Enter to confirm, or press Esc to cancel.",
                cmd
            )
        } else {
            format!(
                "**Ready to execute:**\n\n```\n$ {}\n```\n\n\
                Press **Enter** to execute or **Esc** to cancel.",
                cmd
            )
        };

        self.update_thinking_message(text);
    }
}
