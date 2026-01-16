mod stages;
mod chat;
mod input;

use ratatui::prelude::*;

use crate::app::App;

use stages::render_stages;
use chat::render_chat;

/// Main UI layout
pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let layout = Layout::horizontal([
        Constraint::Percentage(35),
        Constraint::Percentage(65),
    ])
    .split(area);

    render_stages(frame, layout[0], app);
    render_chat(frame, layout[1], app);
}
