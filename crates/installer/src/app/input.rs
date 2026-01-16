use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::widgets::ListState;

use super::App;
use crate::types::Focus;

impl App {
    pub fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        self.status_message = None;

        // Handle Ctrl+C (double-tap to quit)
        if code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
            if self.ctrl_c.press() {
                self.should_quit = true;
            } else {
                self.status_message = Some("Press Ctrl+C again to quit".into());
            }
            return;
        }

        // Handle pending command confirmation
        if self.handle_pending_command_input(code) {
            return;
        }

        // Handle slash menu navigation
        if self.handle_slash_menu_input(code) {
            return;
        }

        // Handle normal input
        self.handle_normal_input(code, modifiers);
    }

    fn handle_pending_command_input(&mut self, code: KeyCode) -> bool {
        let Some(ref mut pending) = self.pending_command else {
            return false;
        };

        match code {
            KeyCode::Enter => {
                if pending.is_destructive {
                    if pending.confirmation_input.trim().to_lowercase() == "yes" {
                        self.execute_pending_command();
                    } else {
                        self.status_message = Some("Type 'yes' to confirm destructive action".into());
                    }
                } else {
                    self.execute_pending_command();
                }
            }
            KeyCode::Esc => {
                self.cancel_pending_command();
            }
            KeyCode::Char(c) if pending.is_destructive => {
                pending.confirmation_input.push(c);
            }
            KeyCode::Backspace if pending.is_destructive => {
                pending.confirmation_input.pop();
            }
            _ => {}
        }
        true
    }

    fn handle_slash_menu_input(&mut self, code: KeyCode) -> bool {
        if !self.slash_menu_visible || self.focus != Focus::Input {
            return false;
        }

        let filtered = self.filtered_slash_commands();
        match code {
            KeyCode::Up => {
                self.slash_menu_cursor = self.slash_menu_cursor.saturating_sub(1);
                self.slash_menu_state.select(Some(self.slash_menu_cursor));
            }
            KeyCode::Down => {
                if self.slash_menu_cursor < filtered.len().saturating_sub(1) {
                    self.slash_menu_cursor += 1;
                }
                self.slash_menu_state.select(Some(self.slash_menu_cursor));
            }
            KeyCode::Enter => {
                if let Some(cmd) = filtered.get(self.slash_menu_cursor) {
                    self.input = cmd.name.to_string();
                    self.slash_menu_visible = false;
                    self.submit_input();
                }
            }
            KeyCode::Esc => {
                self.slash_menu_visible = false;
                self.input.clear();
            }
            KeyCode::Tab => {
                if let Some(cmd) = filtered.get(self.slash_menu_cursor) {
                    self.input = cmd.name.to_string();
                    self.slash_menu_visible = false;
                }
            }
            _ => return false,
        }
        true
    }

    fn handle_normal_input(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        match code {
            KeyCode::Esc => {
                if self.slash_menu_visible {
                    self.slash_menu_visible = false;
                } else {
                    self.should_quit = true;
                }
            }

            // Panel navigation
            KeyCode::Left => {
                self.focus = Focus::Checklist;
            }
            KeyCode::Right => {
                self.focus = Focus::Input;
            }
            KeyCode::Tab if !self.slash_menu_visible => {
                self.focus = match self.focus {
                    Focus::Checklist => Focus::Chat,
                    Focus::Chat => Focus::Input,
                    Focus::Input => Focus::Checklist,
                };
            }

            // Panel-specific: Up/Down
            KeyCode::Up => match self.focus {
                Focus::Checklist => {
                    self.stage_cursor = self.stage_cursor.saturating_sub(1);
                }
                Focus::Chat => {
                    self.chat_scroll = self.chat_scroll.saturating_sub(1);
                }
                Focus::Input => {
                    self.focus = Focus::Chat;
                }
            },
            KeyCode::Down => match self.focus {
                Focus::Checklist => {
                    if self.stage_cursor < self.stages.len() - 1 {
                        self.stage_cursor += 1;
                    }
                }
                Focus::Chat => {
                    self.chat_scroll = self.chat_scroll.saturating_add(1);
                }
                Focus::Input => {}
            },

            // Panel-specific: Enter
            KeyCode::Enter => match self.focus {
                Focus::Checklist => {
                    self.stages[self.stage_cursor].expanded =
                        !self.stages[self.stage_cursor].expanded;
                }
                Focus::Chat => {
                    self.focus = Focus::Input;
                }
                Focus::Input => {
                    if modifiers.contains(KeyModifiers::SHIFT) {
                        self.input.push('\n');
                    } else {
                        self.submit_input();
                    }
                }
            },

            // Text input
            KeyCode::Backspace if self.focus == Focus::Input => {
                self.input.pop();
                if !self.input.starts_with('/') {
                    self.slash_menu_visible = false;
                }
                self.slash_menu_cursor = 0;
                self.slash_menu_state = ListState::default();
            }
            KeyCode::Char(c) if self.focus == Focus::Input => {
                self.input.push(c);
                if self.input == "/" {
                    self.slash_menu_visible = true;
                    self.slash_menu_cursor = 0;
                    self.slash_menu_state = ListState::default();
                } else if self.input.starts_with('/') {
                    self.slash_menu_cursor = 0;
                    self.slash_menu_state = ListState::default();
                } else {
                    self.slash_menu_visible = false;
                }
            }

            _ => {}
        }
    }
}
