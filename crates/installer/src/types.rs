use std::time::Instant;

/// Which panel has focus
#[derive(Clone, Copy, PartialEq)]
pub enum Focus {
    Checklist,
    Chat,
    Input,
}

/// Chat message role
#[derive(Clone, PartialEq)]
pub enum Role {
    User,
    Assistant,
}

/// LLM response types for the channel
pub enum LlmResult {
    Text(String),
    Command(String),
    Error(String),
}

/// Installation stage
#[derive(Clone)]
pub struct Stage {
    pub name: &'static str,
    pub hints: &'static [&'static str],
    pub done: bool,
    pub expanded: bool,
}

impl Stage {
    pub fn new(name: &'static str, hints: &'static [&'static str]) -> Self {
        Self {
            name,
            hints,
            done: false,
            expanded: true,
        }
    }
}

/// Chat message
#[derive(Clone)]
pub struct Message {
    pub role: Role,
    pub text: String,
}

/// Slash command definition
pub struct SlashCommand {
    pub name: &'static str,
    pub description: &'static str,
}

pub const SLASH_COMMANDS: &[SlashCommand] = &[
    SlashCommand { name: "/help", description: "Show available commands" },
    SlashCommand { name: "/clear", description: "Clear chat and start fresh" },
    SlashCommand { name: "/status", description: "Show installation progress" },
    SlashCommand { name: "/disks", description: "List available disks" },
    SlashCommand { name: "/uefi", description: "Show boot mode (UEFI/BIOS)" },
    SlashCommand { name: "/mounts", description: "Show current mount points" },
    SlashCommand { name: "/network", description: "Check network connectivity" },
    SlashCommand { name: "/shell", description: "Drop to emergency shell" },
    SlashCommand { name: "/history", description: "Show action history" },
    SlashCommand { name: "/undo", description: "Undo last action" },
    SlashCommand { name: "/quit", description: "Exit the installer" },
];

/// Pending command awaiting user confirmation
pub struct PendingCommand {
    pub command: String,
    pub is_destructive: bool,
    pub confirmation_input: String,
}

impl PendingCommand {
    pub fn new(command: String) -> Self {
        let is_destructive = Self::check_destructive(&command);
        Self {
            command,
            is_destructive,
            confirmation_input: String::new(),
        }
    }

    pub fn check_destructive(cmd: &str) -> bool {
        let cmd_lower = cmd.to_lowercase();
        cmd_lower.contains("mkfs.")
            || cmd_lower.contains("sgdisk")
            || cmd_lower.contains("fdisk")
            || cmd_lower.contains("parted")
            || cmd_lower.contains("dd ")
            || cmd_lower.contains("wipefs")
    }
}

/// Tracked action for history/undo
#[derive(Clone)]
pub struct TrackedAction {
    pub command: String,
    pub output: String,
    pub success: bool,
    pub undo_command: Option<String>,
    pub undone: bool,
}

/// Double Ctrl+C tracking
pub struct CtrlCTracker {
    pub last: Option<Instant>,
}

impl CtrlCTracker {
    pub fn new() -> Self {
        Self { last: None }
    }

    pub fn press(&mut self) -> bool {
        if let Some(last) = self.last {
            if last.elapsed().as_secs() < 2 {
                return true; // Should quit
            }
        }
        self.last = Some(Instant::now());
        false
    }
}
