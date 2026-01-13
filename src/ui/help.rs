//! Help overlay component

use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, _app: &App) {
    let area = centered_rect(70, 80, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            " Help ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center);

    let help_text = vec![
        Line::from(vec![Span::styled(
            "Navigation",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  j/k, Up/Down  ", Style::default().fg(Color::Cyan)),
            Span::raw("Navigate up/down"),
        ]),
        Line::from(vec![
            Span::styled("  gg            ", Style::default().fg(Color::Cyan)),
            Span::raw("Go to top"),
        ]),
        Line::from(vec![
            Span::styled("  G             ", Style::default().fg(Color::Cyan)),
            Span::raw("Go to bottom"),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+f/b      ", Style::default().fg(Color::Cyan)),
            Span::raw("Page down/up"),
        ]),
        Line::from(vec![
            Span::styled("  b, Backspace  ", Style::default().fg(Color::Cyan)),
            Span::raw("Go back"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Commands",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  :             ", Style::default().fg(Color::Cyan)),
            Span::raw("Open command mode"),
        ]),
        Line::from(vec![
            Span::styled("  /             ", Style::default().fg(Color::Cyan)),
            Span::raw("Filter items"),
        ]),
        Line::from(vec![
            Span::styled("  Enter, d      ", Style::default().fg(Color::Cyan)),
            Span::raw("View details (JSON)"),
        ]),
        Line::from(vec![
            Span::styled("  R             ", Style::default().fg(Color::Cyan)),
            Span::raw("Refresh"),
        ]),
        Line::from(vec![
            Span::styled("  ?             ", Style::default().fg(Color::Cyan)),
            Span::raw("Show this help"),
        ]),
        Line::from(vec![
            Span::styled("  q             ", Style::default().fg(Color::Cyan)),
            Span::raw("Quit"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "VM Actions",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  r             ", Style::default().fg(Color::Cyan)),
            Span::raw("Resume VM"),
        ]),
        Line::from(vec![
            Span::styled("  u             ", Style::default().fg(Color::Cyan)),
            Span::raw("Suspend VM"),
        ]),
        Line::from(vec![
            Span::styled("  s             ", Style::default().fg(Color::Cyan)),
            Span::raw("Stop VM"),
        ]),
        Line::from(vec![
            Span::styled("  S             ", Style::default().fg(Color::Cyan)),
            Span::raw("Power off VM"),
        ]),
        Line::from(vec![
            Span::styled("  R             ", Style::default().fg(Color::Cyan)),
            Span::raw("Reboot VM"),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+d        ", Style::default().fg(Color::Red)),
            Span::raw("Terminate VM (destructive)"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Resources",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  :one-vms      ", Style::default().fg(Color::Cyan)),
            Span::raw("Virtual Machines"),
        ]),
        Line::from(vec![
            Span::styled("  :one-hosts    ", Style::default().fg(Color::Cyan)),
            Span::raw("Hosts"),
        ]),
        Line::from(vec![
            Span::styled("  :one-datastores", Style::default().fg(Color::Cyan)),
            Span::raw("Datastores"),
        ]),
        Line::from(vec![
            Span::styled("  :one-vnets    ", Style::default().fg(Color::Cyan)),
            Span::raw("Virtual Networks"),
        ]),
        Line::from(vec![
            Span::styled("  :one-images   ", Style::default().fg(Color::Cyan)),
            Span::raw("Images"),
        ]),
        Line::from(vec![
            Span::styled("  :one-templates", Style::default().fg(Color::Cyan)),
            Span::raw("VM Templates"),
        ]),
        Line::from(vec![
            Span::styled("  :one-clusters ", Style::default().fg(Color::Cyan)),
            Span::raw("Clusters"),
        ]),
        Line::from(vec![
            Span::styled("  :one-users    ", Style::default().fg(Color::Cyan)),
            Span::raw("Users"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::DarkGray)),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::styled(" or ", Style::default().fg(Color::DarkGray)),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::styled(" to close", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let paragraph = Paragraph::new(help_text).block(block);
    f.render_widget(paragraph, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
