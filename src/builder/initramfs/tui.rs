//! Non-interactive TUI dashboard for initramfs build
//!
//! TEAM_474: Display-only progress dashboard with automatic TTY detection.
//!
//! Key design: NO user input. Display-only progress and status.

use super::builder::BuildEvent;
use super::manifest::ManifestTotals;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};
use std::collections::VecDeque;
use std::io::IsTerminal;
use std::time::{Duration, Instant};

/// Maximum activity items to display
const MAX_ACTIVITY_ITEMS: usize = 12;

/// Non-interactive dashboard state
pub struct Dashboard {
    arch: String,
    current_phase: String,
    phase_progress: (usize, usize),
    activity: VecDeque<ActivityItem>,
    stats: BuildStats,
    start_time: Instant,
    complete: bool,
    error: Option<String>,
    output_path: Option<String>,
    total_size: u64,
}

struct ActivityItem {
    icon: &'static str,
    text: String,
    size: Option<u64>,
    status: ItemStatus,
}

#[derive(Clone, Copy, PartialEq)]
enum ItemStatus {
    Done,
    InProgress,
}

#[derive(Default)]
struct BuildStats {
    directories: usize,
    binaries: usize,
    binary_bytes: u64,
    symlinks_done: usize,
    symlinks_total: usize,
    files_done: usize,
    files_total: usize,
    devices: usize,
}

impl Dashboard {
    pub fn new(arch: &str, totals: &ManifestTotals) -> Self {
        Self {
            arch: arch.to_string(),
            current_phase: "Initializing".to_string(),
            phase_progress: (0, 1),
            activity: VecDeque::with_capacity(MAX_ACTIVITY_ITEMS),
            stats: BuildStats {
                symlinks_total: totals.symlinks,
                files_total: totals.files,
                ..Default::default()
            },
            start_time: Instant::now(),
            complete: false,
            error: None,
            output_path: None,
            total_size: 0,
        }
    }

    pub fn handle_event(&mut self, event: &BuildEvent) {
        match event {
            BuildEvent::PhaseStart { name, total } => {
                self.current_phase = name.to_string();
                self.phase_progress = (0, *total);
            }
            BuildEvent::PhaseComplete { .. } => {
                self.phase_progress.0 = self.phase_progress.1;
            }
            BuildEvent::DirectoryCreated { path } => {
                self.stats.directories += 1;
                self.phase_progress.0 += 1;
                self.add_activity("📁", path.clone(), None, ItemStatus::Done);
            }
            BuildEvent::BinaryAdded { path, size } => {
                self.stats.binaries += 1;
                self.stats.binary_bytes += size;
                self.phase_progress.0 += 1;
                self.add_activity("+", path.clone(), Some(*size), ItemStatus::Done);
            }
            BuildEvent::SymlinkCreated { link, target } => {
                self.stats.symlinks_done += 1;
                self.phase_progress.0 += 1;
                self.add_activity("→", format!("{} -> {}", link, target), None, ItemStatus::Done);
            }
            BuildEvent::FileAdded { path, size } => {
                self.stats.files_done += 1;
                self.phase_progress.0 += 1;
                self.add_activity("📄", path.clone(), Some(*size), ItemStatus::Done);
            }
            BuildEvent::DeviceCreated { path } => {
                self.stats.devices += 1;
                self.phase_progress.0 += 1;
                self.add_activity("⚙", path.clone(), None, ItemStatus::Done);
            }
            BuildEvent::BuildComplete {
                output_path,
                total_size,
                ..
            } => {
                self.complete = true;
                self.output_path = Some(output_path.display().to_string());
                self.total_size = *total_size;
            }
            BuildEvent::BuildFailed { error } => {
                self.complete = true;
                self.error = Some(error.clone());
            }
        }
    }

    fn add_activity(&mut self, icon: &'static str, text: String, size: Option<u64>, status: ItemStatus) {
        if self.activity.len() >= MAX_ACTIVITY_ITEMS {
            self.activity.pop_front();
        }
        self.activity.push_back(ActivityItem {
            icon,
            text,
            size,
            status,
        });
    }

    pub fn is_complete(&self) -> bool {
        self.complete
    }

    pub fn render(&self, frame: &mut Frame) {
        let chunks = Layout::vertical([
            Constraint::Length(3),  // Header
            Constraint::Length(3),  // Progress
            Constraint::Min(8),     // Activity
            Constraint::Length(5),  // Statistics
        ])
        .split(frame.area());

        self.render_header(frame, chunks[0]);
        self.render_progress(frame, chunks[1]);
        self.render_activity(frame, chunks[2]);
        self.render_stats(frame, chunks[3]);
    }

    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let title = if self.error.is_some() {
            format!("  LEVITATE INITRAMFS BUILDER - ERROR                          {} ", self.arch)
        } else if self.complete {
            format!("  LEVITATE INITRAMFS BUILDER - COMPLETE                       {} ", self.arch)
        } else {
            format!("  LEVITATE INITRAMFS BUILDER                                  {} ", self.arch)
        };

        let style = if self.error.is_some() {
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        } else if self.complete {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        };

        let header = Paragraph::new(title)
            .style(style)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(header, area);
    }

    fn render_progress(&self, frame: &mut Frame, area: Rect) {
        let (done, total) = self.phase_progress;
        let ratio = if total > 0 {
            done as f64 / total as f64
        } else {
            0.0
        };

        let label = format!("Phase: {}  [{}/{}]", self.current_phase, done, total);
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Green))
            .label(label)
            .ratio(ratio.min(1.0));
        frame.render_widget(gauge, area);
    }

    fn render_activity(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .activity
            .iter()
            .map(|item| {
                let status_icon = match item.status {
                    ItemStatus::Done => "✓",
                    ItemStatus::InProgress => "◉",
                };

                let size_str = item
                    .size
                    .map(|s| format!(" ({:.1} KB)", s as f64 / 1024.0))
                    .unwrap_or_default();

                let line = Line::from(vec![
                    Span::styled(
                        format!(" {} ", status_icon),
                        Style::default().fg(match item.status {
                            ItemStatus::Done => Color::Green,
                            ItemStatus::InProgress => Color::Yellow,
                        }),
                    ),
                    Span::raw(item.icon),
                    Span::raw(" "),
                    Span::raw(&item.text),
                    Span::styled(size_str, Style::default().fg(Color::DarkGray)),
                ]);
                ListItem::new(line)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .title(" ACTIVITY ")
                .borders(Borders::ALL),
        );
        frame.render_widget(list, area);
    }

    fn render_stats(&self, frame: &mut Frame, area: Rect) {
        let elapsed = self.start_time.elapsed();
        let elapsed_str = format!("{:.1}s", elapsed.as_secs_f64());
        let size_str = format!("{:.1} KB", self.total_size as f64 / 1024.0);

        let stats_text = if let Some(error) = &self.error {
            format!("  ERROR: {}", error)
        } else if self.complete {
            format!(
                "  Output: {}    Size: {}    Elapsed: {}",
                self.output_path.as_deref().unwrap_or("?"),
                size_str,
                elapsed_str
            )
        } else {
            format!(
                "  Directories {:3}    Symlinks {:3}/{:3}    Total Size {:>10}\n  \
                   Binaries    {:3}    Files    {:3}/{:3}    Elapsed    {:>10}",
                self.stats.directories,
                self.stats.symlinks_done,
                self.stats.symlinks_total,
                format!("{:.1} KB", self.stats.binary_bytes as f64 / 1024.0),
                self.stats.binaries,
                self.stats.files_done,
                self.stats.files_total,
                elapsed_str,
            )
        };

        let stats = Paragraph::new(stats_text).block(
            Block::default()
                .title(" STATISTICS ")
                .borders(Borders::ALL),
        );
        frame.render_widget(stats, area);
    }
}

/// Auto-detect if TUI should be used
pub fn should_use_tui() -> bool {
    std::io::stdout().is_terminal()
        && std::env::var("NO_TUI").is_err()
        && std::env::var("CI").is_err()
}

/// Print simple line output for non-TTY environments
pub fn print_simple_event(event: &BuildEvent) {
    match event {
        BuildEvent::PhaseStart { name, total } => {
            println!("  {} ({} items)...", name, total);
        }
        BuildEvent::BinaryAdded { path, size } => {
            println!("    + {} ({:.1} KB)", path, *size as f64 / 1024.0);
        }
        BuildEvent::SymlinkCreated { link, target } => {
            // Only print occasionally to avoid spam
            if link.ends_with("/sh") || link.ends_with("/ash") || link.ends_with("/init") {
                println!("    -> {} -> {}", link, target);
            }
        }
        BuildEvent::FileAdded { path, .. } => {
            println!("    + {}", path);
        }
        BuildEvent::BuildComplete {
            output_path,
            total_size,
            duration,
        } => {
            println!(
                "  Initramfs created: {} ({:.1} KB) in {:.2}s",
                output_path.display(),
                *total_size as f64 / 1024.0,
                duration.as_secs_f64()
            );
        }
        BuildEvent::BuildFailed { error } => {
            eprintln!("  ERROR: {}", error);
        }
        _ => {}
    }
}

/// Run build with TUI dashboard
pub fn run_build_with_tui(
    arch: &str,
    totals: &ManifestTotals,
    build_fn: impl FnOnce(Box<dyn Fn(BuildEvent) + Send>) -> anyhow::Result<std::path::PathBuf> + Send + 'static,
) -> anyhow::Result<std::path::PathBuf> {
    use std::sync::mpsc;

    let mut terminal = ratatui::init();

    let mut dashboard = Dashboard::new(arch, totals);
    let (tx, rx) = mpsc::channel();

    // Build in separate thread
    let build_handle = std::thread::spawn(move || {
        let sender = tx;
        build_fn(Box::new(move |event| {
            let _ = sender.send(event);
        }))
    });

    // Render loop (non-blocking)
    loop {
        // Process available events
        while let Ok(event) = rx.try_recv() {
            dashboard.handle_event(&event);
        }

        terminal.draw(|f| dashboard.render(f))?;

        if dashboard.is_complete() {
            std::thread::sleep(Duration::from_millis(800)); // Brief pause to show result
            break;
        }

        std::thread::sleep(Duration::from_millis(16)); // ~60fps
    }

    // Cleanup
    ratatui::restore();

    build_handle.join().unwrap()
}
