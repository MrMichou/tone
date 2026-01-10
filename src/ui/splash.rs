//! Splash screen component

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

const LOGO: &str = r#"
  _
 | |_ ___  _ __   ___
 | __/ _ \| '_ \ / _ \
 | || (_) | | | |  __/
  \__\___/|_| |_|\___|

"#;

pub struct SplashState {
    pub current_step: usize,
    pub total_steps: usize,
    pub message: String,
}

impl SplashState {
    pub fn new() -> Self {
        Self {
            current_step: 0,
            total_steps: 4,
            message: "Initializing...".to_string(),
        }
    }

    pub fn set_message(&mut self, msg: &str) {
        self.message = msg.to_string();
    }

    pub fn complete_step(&mut self) {
        self.current_step = (self.current_step + 1).min(self.total_steps);
    }
}

pub fn render(f: &mut Frame, state: &SplashState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(10),
            Constraint::Length(3),
            Constraint::Percentage(30),
        ])
        .split(f.area());

    // Logo
    let logo = Paragraph::new(LOGO)
        .style(
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(logo, chunks[1]);

    // Progress area
    let progress_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(chunks[2]);

    // Progress bar
    let progress_width = chunks[2].width.saturating_sub(4) as usize;
    let filled = (state.current_step * progress_width) / state.total_steps.max(1);
    let empty = progress_width.saturating_sub(filled);
    let progress_bar = format!("[{}{}]", "=".repeat(filled), " ".repeat(empty));

    let progress = Paragraph::new(Line::from(vec![Span::styled(
        progress_bar,
        Style::default().fg(Color::Cyan),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(progress, progress_chunks[0]);

    // Status message
    let status = Paragraph::new(Line::from(vec![Span::styled(
        &state.message,
        Style::default().fg(Color::DarkGray),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(status, progress_chunks[1]);

    // Border
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " Terminal UI for OpenNebula ",
            Style::default().fg(Color::DarkGray),
        ))
        .title_alignment(Alignment::Center);
    f.render_widget(block, f.area());
}
