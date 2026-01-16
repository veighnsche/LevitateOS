mod commands;
mod input;
mod llm_query;

use ratatui::widgets::ListState;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;

use crate::llm::LlmServer;
use crate::types::*;

/// Application state
pub struct App {
    // UI state
    pub focus: Focus,
    pub stages: Vec<Stage>,
    pub stage_cursor: usize,
    pub messages: Vec<Message>,
    pub chat_scroll: usize,
    pub input: String,
    pub should_quit: bool,
    pub status_message: Option<String>,

    // Slash command menu
    pub slash_menu_visible: bool,
    pub slash_menu_cursor: usize,
    pub slash_menu_state: ListState,

    // Command execution
    pub pending_command: Option<PendingCommand>,
    pub action_history: Vec<TrackedAction>,
    pub drop_to_shell: bool,

    // LLM
    pub llm: Arc<LlmServer>,
    pub waiting_for_llm: bool,
    pub(crate) llm_rx: Receiver<LlmResult>,
    pub(crate) llm_tx: Sender<LlmResult>,

    // Internal
    pub(crate) ctrl_c: CtrlCTracker,
}

impl App {
    pub fn new(llm: LlmServer) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            focus: Focus::Input,
            stages: Self::default_stages(),
            stage_cursor: 0,
            messages: vec![Message {
                role: Role::Assistant,
                text: concat!(
                    "# Welcome to LevitateOS Installer!\n\n",
                    "**LLM:** `FunctionGemma 2B`\n",
                    "**LoRA:** `levitate-installer`\n\n",
                    "Type what you want to do in natural language.\n",
                    "Try: `list disks` to see available drives."
                ).into(),
            }],
            chat_scroll: 0,
            input: String::new(),
            should_quit: false,
            status_message: None,
            slash_menu_visible: false,
            slash_menu_cursor: 0,
            slash_menu_state: ListState::default(),
            pending_command: None,
            action_history: Vec::new(),
            drop_to_shell: false,
            llm: Arc::new(llm),
            llm_rx: rx,
            llm_tx: tx,
            waiting_for_llm: false,
            ctrl_c: CtrlCTracker::new(),
        }
    }

    fn default_stages() -> Vec<Stage> {
        vec![
            Stage::new("Disk Configuration", &[
                "\"list disks\"",
                "\"use the whole 500gb drive\"",
                "\"encrypted root partition\"",
                "\"dual boot with windows\"",
            ]),
            Stage { expanded: false, ..Stage::new("System Installation", &[
                "\"install the system\"",
                "\"copy to disk\"",
            ])},
            Stage { expanded: false, ..Stage::new("System Configuration", &[
                "\"hostname is my-laptop\"",
                "\"timezone los angeles\"",
                "\"keyboard us\"",
                "\"language english\"",
            ])},
            Stage { expanded: false, ..Stage::new("User Setup", &[
                "\"create user vince with sudo\"",
                "\"set password\"",
                "\"add user to wheel group\"",
            ])},
            Stage { expanded: false, ..Stage::new("Bootloader", &[
                "\"install bootloader\"",
                "(automatic after disk config)",
            ])},
            Stage { expanded: false, ..Stage::new("Finalize", &[
                "\"done\"",
                "\"reboot now\"",
                "\"exit without reboot\"",
            ])},
        ]
    }

    pub fn filtered_slash_commands(&self) -> Vec<&'static SlashCommand> {
        let filter = self.input.to_lowercase();
        SLASH_COMMANDS
            .iter()
            .filter(|cmd| cmd.name.starts_with(&filter) || filter == "/")
            .collect()
    }
}
