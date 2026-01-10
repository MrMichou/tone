//! Dialog components for confirmations and warnings

use crate::app::{App, Mode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &App) {
    match app.mode {
        Mode::Confirm => render_confirm(f, app),
        Mode::Warning => render_warning(f, app),
        _ => {}
    }
}

fn render_confirm(f: &mut Frame, app: &App) {
    let Some(pending) = &app.pending_action else {
        return;
    };

    let area = centered_rect(50, 8, f.area());
    f.render_widget(Clear, area);

    let border_color = if pending.destructive {
        Color::Red
    } else {
        Color::Yellow
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            if pending.destructive {
                " Destructive Action "
            } else {
                " Confirm "
            },
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    // Message
    let message = Paragraph::new(Line::from(vec![Span::styled(
        &pending.message,
        Style::default().fg(Color::White),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(message, chunks[0]);

    // Buttons
    let yes_style = if pending.selected_yes {
        Style::default()
            .fg(Color::Black)
            .bg(if pending.destructive {
                Color::Red
            } else {
                Color::Green
            })
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let no_style = if !pending.selected_yes {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let buttons = Line::from(vec![
        Span::raw("       "),
        Span::styled(" Yes ", yes_style),
        Span::raw("    "),
        Span::styled(" No ", no_style),
        Span::raw("       "),
    ]);
    let buttons_para = Paragraph::new(buttons).alignment(Alignment::Center);
    f.render_widget(buttons_para, chunks[1]);

    // Hint
    let hint = Paragraph::new(Line::from(vec![Span::styled(
        "y/n or Enter to confirm, Esc to cancel",
        Style::default().fg(Color::DarkGray),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(hint, chunks[2]);
}

fn render_warning(f: &mut Frame, app: &App) {
    let Some(message) = &app.warning_message else {
        return;
    };

    let area = centered_rect(50, 6, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title(Span::styled(
            " Warning ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(1)])
        .split(inner);

    let message_para = Paragraph::new(Line::from(vec![Span::styled(
        message,
        Style::default().fg(Color::White),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(message_para, chunks[0]);

    let hint = Paragraph::new(Line::from(vec![Span::styled(
        "Press Enter or Esc to close",
        Style::default().fg(Color::DarkGray),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(hint, chunks[1]);
}

fn centered_rect(percent_x: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(height),
            Constraint::Percentage(40),
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
