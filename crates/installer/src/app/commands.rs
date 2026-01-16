use super::App;
use crate::types::*;

impl App {
    pub fn handle_slash_command(&mut self, cmd: &str) {
        let cmd = cmd.trim();

        self.messages.push(Message {
            role: Role::User,
            text: cmd.to_string(),
        });

        let response = match cmd {
            "/help" => self.cmd_help(),
            "/clear" => self.cmd_clear(),
            "/disks" => return self.cmd_disks(),
            "/status" => self.cmd_status(),
            "/uefi" => self.cmd_uefi(),
            "/mounts" => self.cmd_mounts(),
            "/network" => self.cmd_network(),
            "/shell" => return self.cmd_shell(),
            "/history" => self.cmd_history(),
            "/undo" => self.cmd_undo(),
            "/quit" | "/exit" => return self.cmd_quit(),
            _ => format!("Unknown command: {}. Type /help for available commands.", cmd),
        };

        self.messages.push(Message {
            role: Role::Assistant,
            text: response,
        });
        self.chat_scroll = usize::MAX;
    }

    fn cmd_help(&self) -> String {
        let mut help = String::from("## Available Commands\n\n");
        for c in SLASH_COMMANDS {
            help.push_str(&format!("**{}** - {}\n", c.name, c.description));
        }
        help.push_str("\nOr just type naturally: \"list disks\", \"set hostname to mypc\", etc.");
        help
    }

    fn cmd_clear(&mut self) -> String {
        self.messages.clear();
        "Chat cleared. How can I help you?".to_string()
    }

    fn cmd_disks(&mut self) {
        self.input = "list disks".to_string();
        self.submit_input();
    }

    fn cmd_status(&self) -> String {
        let mut status = String::from("## Installation Status\n\n");
        for stage in &self.stages {
            let icon = if stage.done { "✓" } else { "○" };
            status.push_str(&format!("{} {}\n", icon, stage.name));
        }
        status
    }

    fn cmd_uefi(&self) -> String {
        if std::path::Path::new("/sys/firmware/efi").exists() {
            "**Boot Mode:** UEFI ✓\n\nYour system supports UEFI boot. The installer will create an EFI System Partition.".to_string()
        } else {
            "**Boot Mode:** Legacy BIOS\n\nYour system is using legacy BIOS boot. The installer will use MBR partitioning.".to_string()
        }
    }

    fn cmd_mounts(&self) -> String {
        match std::process::Command::new("findmnt")
            .args(["--target", "/mnt", "-o", "TARGET,SOURCE,FSTYPE", "--noheadings"])
            .output()
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.trim().is_empty() {
                    "**No mounts under /mnt**\n\nTarget partitions are not mounted yet.".to_string()
                } else {
                    format!("## Current Mounts\n\n```\n{}\n```", stdout.trim())
                }
            }
            Err(_) => "Failed to check mounts.".to_string()
        }
    }

    fn cmd_network(&self) -> String {
        match std::process::Command::new("ping")
            .args(["-c", "1", "-W", "2", "archlinux.org"])
            .output()
        {
            Ok(output) if output.status.success() => {
                "**Network:** Connected ✓\n\nYou can download packages and updates.".to_string()
            }
            _ => "**Network:** Not connected ✗\n\nYou may need to configure networking first.".to_string()
        }
    }

    fn cmd_shell(&mut self) {
        self.drop_to_shell = true;
    }

    fn cmd_history(&self) -> String {
        if self.action_history.is_empty() {
            return "**Action History**\n\nNo actions recorded yet.".to_string();
        }

        let mut history = String::from("## Action History\n\n");
        for (i, action) in self.action_history.iter().enumerate().rev() {
            let status = if action.undone {
                "↩️ undone"
            } else if action.success {
                "✓"
            } else {
                "✗"
            };
            let undo_info = if action.undo_command.is_some() && !action.undone {
                " (can undo)"
            } else {
                ""
            };
            history.push_str(&format!("{}. `{}` {}{}\n", i + 1, action.command, status, undo_info));
        }
        history
    }

    fn cmd_undo(&mut self) -> String {
        let action = self.action_history
            .iter_mut()
            .rev()
            .find(|a| !a.undone && a.undo_command.is_some());

        let Some(action) = action else {
            return "**Nothing to undo.**\n\nNo reversible actions in history.".to_string();
        };

        let undo_cmd = action.undo_command.clone().unwrap();
        match std::process::Command::new("sh").arg("-c").arg(&undo_cmd).output() {
            Ok(output) => {
                action.undone = true;
                if output.status.success() {
                    format!(
                        "**Undone:**\n```\n$ {}\n```\n\nOriginal command `{}` has been reversed.",
                        undo_cmd, action.command
                    )
                } else {
                    format!(
                        "**Undo failed:**\n```\n$ {}\n```\n\n{}",
                        undo_cmd, String::from_utf8_lossy(&output.stderr)
                    )
                }
            }
            Err(e) => format!("**Undo error:** {}", e)
        }
    }

    fn cmd_quit(&mut self) {
        self.should_quit = true;
    }

    pub fn execute_pending_command(&mut self) {
        let Some(pending) = self.pending_command.take() else {
            return;
        };

        let output = match std::process::Command::new("sh")
            .arg("-c")
            .arg(&pending.command)
            .output()
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let success = output.status.success();

                let undo_command = Self::generate_undo_command(&pending.command);

                self.action_history.push(TrackedAction {
                    command: pending.command.clone(),
                    output: format!("{}{}", stdout, stderr),
                    success,
                    undo_command,
                    undone: false,
                });

                let combined = format!("{}{}", stdout, stderr);
                let truncated = Self::truncate_output(&combined, 10);

                if success {
                    format!(
                        "**Executed:**\n```\n$ {}\n```\n\n**Output:**\n```\n{}\n```",
                        pending.command, truncated
                    )
                } else {
                    format!(
                        "**Failed** (exit code {}):\n```\n$ {}\n```\n\n**Output:**\n```\n{}\n```",
                        output.status.code().unwrap_or(-1), pending.command, truncated
                    )
                }
            }
            Err(e) => {
                format!("**Error executing command:**\n```\n$ {}\n```\n\n{}", pending.command, e)
            }
        };

        if let Some(msg) = self.messages.last_mut() {
            if msg.role == Role::Assistant {
                msg.text = output;
            }
        }
        self.chat_scroll = usize::MAX;
    }

    pub fn cancel_pending_command(&mut self) {
        if self.pending_command.take().is_some() {
            if let Some(msg) = self.messages.last_mut() {
                if msg.role == Role::Assistant {
                    msg.text = "Command cancelled.".to_string();
                }
            }
            self.chat_scroll = usize::MAX;
        }
    }

    fn generate_undo_command(command: &str) -> Option<String> {
        let cmd = command.to_lowercase();
        if cmd.starts_with("mount ") {
            let parts: Vec<&str> = command.split_whitespace().collect();
            parts.last().map(|mp| format!("umount {}", mp))
        } else if cmd.starts_with("mkdir ") {
            let path = command.replace("mkdir", "").replace("-p", "").trim().to_string();
            Some(format!("rmdir {}", path))
        } else if cmd.starts_with("useradd ") {
            command.split_whitespace().last().map(|user| format!("userdel {}", user))
        } else {
            None
        }
    }

    fn truncate_output(output: &str, max_lines: usize) -> String {
        let lines: Vec<&str> = output.lines().collect();
        if lines.len() > max_lines {
            format!("{}\n... ({} more lines)", lines[..max_lines].join("\n"), lines.len() - max_lines)
        } else {
            output.trim().to_string()
        }
    }
}
