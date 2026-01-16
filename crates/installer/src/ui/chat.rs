use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::App;
use crate::types::{Focus, Role};
use super::input::render_input;

/// Render the chat panel (messages + input)
pub fn render_chat(frame: &mut Frame, area: Rect, app: &mut App) {
    // Input height: 3 lines min, grows with content up to 6
    let input_lines = app.input.lines().count().max(1);
    let input_height = (input_lines + 2).min(6) as u16;

    // Status bar if there's a message
    let status_height = if app.status_message.is_some() { 1 } else { 0 };

    let layout = Layout::vertical([
        Constraint::Min(1),
        Constraint::Length(input_height),
        Constraint::Length(status_height),
    ])
    .split(area);

    render_messages(frame, layout[0], app);
    render_input(frame, layout[1], app);

    // Render status message
    if let Some(msg) = &app.status_message {
        let status = Paragraph::new(msg.as_str())
            .style(Style::default().fg(Color::Yellow).bold());
        frame.render_widget(status, layout[2]);
    }
}

/// Render chat messages
fn render_messages(frame: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus == Focus::Chat;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let inner_area = Block::default()
        .borders(Borders::ALL)
        .inner(area);

    // Build message lines
    let mut lines: Vec<Line> = Vec::new();
    for msg in &app.messages {
        match msg.role {
            Role::User => {
                for line in msg.text.lines() {
                    lines.push(Line::styled(
                        format!("> {}", line),
                        Style::default().fg(Color::Cyan),
                    ));
                }
            }
            Role::Assistant => {
                let parsed = tui_markdown::from_str(&msg.text);
                for line in parsed.lines {
                    lines.push(line.clone());
                }
            }
        }
        lines.push(Line::raw(""));
    }

    // Calculate scroll
    let visible_height = inner_area.height as usize;
    let total_lines = lines.len();
    let max_scroll = total_lines.saturating_sub(visible_height);
    let scroll = app.chat_scroll.min(max_scroll);

    let title = if focused {
        " Chat (↑↓ scroll, Enter to input) "
    } else {
        " Chat "
    };

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((scroll as u16, 0))
        .block(Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style));

    frame.render_widget(paragraph, area);
}
