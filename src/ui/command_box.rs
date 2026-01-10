//! Command box component

use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &App) {
    let area = bottom_rect(f.area());
    f.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(area);

    // Command input
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            " Command ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));

    let display_text = if app.command_text.is_empty() {
        if let Some(ref preview) = app.command_preview {
            format!(":{}_", preview)
        } else {
            ":_".to_string()
        }
    } else if let Some(ref preview) = app.command_preview {
        if preview.starts_with(&app.command_text) {
            let rest = &preview[app.command_text.len()..];
            format!(":{}|{}", app.command_text, rest)
        } else {
            format!(":{}_", app.command_text)
        }
    } else {
        format!(":{}_", app.command_text)
    };

    let input = Paragraph::new(Line::from(vec![Span::styled(
        display_text,
        Style::default().fg(Color::White),
    )]))
    .block(block);
    f.render_widget(input, chunks[0]);

    // Suggestions list
    if !app.command_suggestions.is_empty() {
        let suggestions_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Span::styled(
                " Suggestions ",
                Style::default().fg(Color::DarkGray),
            ));

        let items: Vec<ListItem> = app
            .command_suggestions
            .iter()
            .enumerate()
            .take(10)
            .map(|(i, s)| {
                let style = if i == app.command_suggestion_selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(Line::from(vec![Span::styled(format!(" {}", s), style)]))
            })
            .collect();

        let list = List::new(items).block(suggestions_block);
        f.render_widget(list, chunks[1]);
    }
}

fn bottom_rect(r: Rect) -> Rect {
    let height = 15.min(r.height.saturating_sub(5));
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(height)])
        .split(r)[1]
}
