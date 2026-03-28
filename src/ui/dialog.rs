//! Dialog components for confirmations and warnings

use crate::app::{App, Mode};
use crate::resource::extract_json_value;
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
        Mode::HostSelect => render_host_select(f, app),
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

fn render_host_select(f: &mut Frame, app: &App) {
    if app.host_list.is_empty() {
        return;
    }

    let vm_name = app.migrate_vm_name.as_deref().unwrap_or("?");

    // Calculate height: header + hosts (max 12) + hint
    let visible_hosts = app.host_list.len().min(12);
    let height = (visible_hosts as u16) + 5; // title border + header + hosts + hint + border

    let area = centered_rect(60, height, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            format!(" Migrate VM '{}' — Select Target Host ", vm_name),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Column headers
            Constraint::Min(1),    // Host list
            Constraint::Length(1), // Hint
        ])
        .split(inner);

    // Column headers
    let header = Line::from(vec![Span::styled(
        format!("{:<6} {:<25} {:<12} {:>6}", "ID", "NAME", "CLUSTER", "VMS"),
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    )]);
    f.render_widget(Paragraph::new(header), chunks[0]);

    // Scrolling offset
    let scroll_offset = if app.host_select_index >= visible_hosts {
        app.host_select_index - visible_hosts + 1
    } else {
        0
    };

    // Host rows
    let mut lines: Vec<Line> = Vec::new();
    for (i, host) in app
        .host_list
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible_hosts)
    {
        let id = extract_json_value(host, "ID");
        let name = extract_json_value(host, "NAME");
        let cluster = extract_json_value(host, "CLUSTER");
        let vms = extract_json_value(host, "HOST_SHARE.RUNNING_VMS");

        let row = format!("{:<6} {:<25} {:<12} {:>6}", id, name, cluster, vms);

        let style = if i == app.host_select_index {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        lines.push(Line::from(Span::styled(row, style)));
    }

    f.render_widget(Paragraph::new(lines), chunks[1]);

    // Hint
    let hint = Paragraph::new(Line::from(vec![Span::styled(
        "j/k to navigate, Enter to select, Esc to cancel",
        Style::default().fg(Color::DarkGray),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(hint, chunks[2]);
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
