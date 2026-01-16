use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::app::App;
use crate::types::Focus;

/// Render the input field
pub fn render_input(frame: &mut Frame, area: Rect, app: &mut App) {
    let focused = app.focus == Focus::Input;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title = if focused {
        " > Type here (Shift+Enter for newline, / for commands) "
    } else {
        " > "
    };

    let input_widget = Paragraph::new(app.input.as_str())
        .block(Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style));

    frame.render_widget(input_widget, area);

    // Show slash command menu if visible
    if app.slash_menu_visible {
        render_slash_menu(frame, area, app);
    }

    // Show cursor only when focused
    if focused {
        let lines: Vec<&str> = app.input.lines().collect();
        let (cursor_x, cursor_y) = if lines.is_empty() {
            (0, 0)
        } else {
            let last_line = lines.last().unwrap_or(&"");
            (last_line.len(), lines.len().saturating_sub(1))
        };

        frame.set_cursor_position(Position::new(
            area.x + cursor_x as u16 + 1,
            area.y + cursor_y as u16 + 1,
        ));
    }
}

/// Render the slash command dropdown menu
fn render_slash_menu(frame: &mut Frame, input_area: Rect, app: &mut App) {
    let filtered = app.filtered_slash_commands();
    if filtered.is_empty() {
        return;
    }

    // Calculate menu dimensions
    let menu_height = (filtered.len() as u16 + 2).min(8);
    let menu_width = 45;

    // Position menu above the input area
    let menu_area = Rect {
        x: input_area.x + 1,
        y: input_area.y.saturating_sub(menu_height),
        width: menu_width.min(input_area.width - 2),
        height: menu_height,
    };

    // Build menu items
    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(i, cmd)| {
            let style = if i == app.slash_menu_cursor {
                Style::default().fg(Color::Black).bg(Color::Cyan).bold()
            } else {
                Style::default().fg(Color::White)
            };
            let line = Line::from(vec![
                Span::styled(format!("{:<10}", cmd.name), style),
                Span::styled(
                    format!(" {}", cmd.description),
                    if i == app.slash_menu_cursor {
                        style
                    } else {
                        Style::default().fg(Color::DarkGray)
                    },
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    // Clear the area first
    frame.render_widget(Clear, menu_area);

    let menu = List::new(items)
        .block(Block::default()
            .title(" Commands (↑↓ Enter) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)));

    // Use persistent ListState to handle scrolling
    frame.render_stateful_widget(menu, menu_area, &mut app.slash_menu_state);
}
