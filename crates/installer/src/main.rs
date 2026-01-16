use crossterm::{
    event::{self, Event, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use std::io::{self, stdout};
use std::process::Stdio;

mod app;
mod llm;
mod types;
mod ui;

use app::App;

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Show loading screen
    terminal.draw(|frame| {
        let area = frame.area();
        let text = Paragraph::new("Loading LLM server...\n\nThis may take a moment.")
            .block(Block::default().title(" LevitateOS Installer ").borders(Borders::ALL))
            .alignment(Alignment::Center);
        frame.render_widget(text, area);
    })?;

    // Start LLM server
    let llm = llm::LlmServer::start("vendor/models/FunctionGemma")
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut app = App::new(llm);

    while !app.should_quit {
        // Handle drop to shell
        if app.drop_to_shell {
            app.drop_to_shell = false;

            disable_raw_mode()?;
            stdout().execute(LeaveAlternateScreen)?;

            println!("\n=== Emergency Shell ===");
            println!("Type 'exit' to return to installer\n");

            let _ = std::process::Command::new("sh")
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status();

            println!("\nReturning to installer...");
            enable_raw_mode()?;
            stdout().execute(EnterAlternateScreen)?;
            terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        }

        app.check_llm_response();
        terminal.draw(|frame| ui::render(frame, &mut app))?;
        handle_events(&mut app)?;
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn handle_events(app: &mut App) -> io::Result<()> {
    if event::poll(std::time::Duration::from_millis(16))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                app.handle_key(key.code, key.modifiers);
            }
        }
    }
    Ok(())
}
