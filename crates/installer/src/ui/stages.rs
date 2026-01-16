use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem},
};

use crate::app::App;
use crate::types::Focus;

/// Render the installation stages panel
pub fn render_stages(frame: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus == Focus::Checklist;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let mut items: Vec<ListItem> = Vec::new();

    for (i, stage) in app.stages.iter().enumerate() {
        let is_selected = i == app.stage_cursor && focused;

        let checkbox = if stage.done { "[x]" } else { "[ ]" };
        let expand_icon = if stage.expanded { "▼" } else { "▶" };

        let style = if is_selected {
            Style::default().fg(Color::Yellow).bold()
        } else if stage.done {
            Style::default().fg(Color::Green)
        } else {
            Style::default().bold()
        };

        let header = Line::from(vec![
            Span::styled(format!("{} {} ", checkbox, expand_icon), style),
            Span::styled(stage.name, style),
        ]);
        items.push(ListItem::new(header));

        // Hints (only if expanded)
        if stage.expanded {
            for hint in stage.hints {
                let hint_style = if is_selected {
                    Style::default().fg(Color::Yellow).dim()
                } else {
                    Style::default().dim()
                };
                let hint_line = Line::from(vec![
                    Span::raw("      "),
                    Span::styled(*hint, hint_style),
                ]);
                items.push(ListItem::new(hint_line));
            }
        }

        items.push(ListItem::new(""));
    }

    let title = if focused {
        " Installation Steps (↑↓ move, Enter toggle) "
    } else {
        " Installation Steps "
    };

    let list = List::new(items)
        .block(Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style));

    frame.render_widget(list, area);
}
