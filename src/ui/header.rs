//! Header component

use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " tone ",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    // Line 1: Endpoint
    let endpoint_line = Line::from(vec![
        Span::styled(" Endpoint: ", Style::default().fg(Color::DarkGray)),
        Span::styled(&app.endpoint, Style::default().fg(Color::Cyan)),
    ]);
    f.render_widget(Paragraph::new(endpoint_line), chunks[0]);

    // Line 2: User info
    let user_line = Line::from(vec![
        Span::styled(" User: ", Style::default().fg(Color::DarkGray)),
        Span::styled(&app.username, Style::default().fg(Color::Green)),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled("Mode: ", Style::default().fg(Color::DarkGray)),
        if app.readonly {
            Span::styled("READ-ONLY", Style::default().fg(Color::Yellow))
        } else {
            Span::styled("READ-WRITE", Style::default().fg(Color::Green))
        },
    ]);
    f.render_widget(Paragraph::new(user_line), chunks[1]);

    // Line 3: Shortcuts
    let shortcuts_line = Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled("?", Style::default().fg(Color::Yellow)),
        Span::styled(":help ", Style::default().fg(Color::DarkGray)),
        Span::styled(":", Style::default().fg(Color::Yellow)),
        Span::styled(":command ", Style::default().fg(Color::DarkGray)),
        Span::styled("/", Style::default().fg(Color::Yellow)),
        Span::styled(":filter ", Style::default().fg(Color::DarkGray)),
        Span::styled("R", Style::default().fg(Color::Yellow)),
        Span::styled(":refresh ", Style::default().fg(Color::DarkGray)),
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::styled(":quit", Style::default().fg(Color::DarkGray)),
    ]);
    f.render_widget(Paragraph::new(shortcuts_line), chunks[2]);
}
