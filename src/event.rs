//! Event handling module
//!
//! Handles keyboard input and user events.

use crate::app::{App, Mode};
use crate::resource::{extract_json_value, invoke_sdk_method};
use anyhow::Result;
use crossterm::event::{poll, read, Event, KeyCode, KeyModifiers};
use std::time::Duration;

/// Handle events and return true if the application should quit
pub async fn handle_events(app: &mut App) -> Result<bool> {
    if poll(Duration::from_millis(100))? {
        if let Event::Key(key) = read()? {
            return handle_key(app, key.code, key.modifiers).await;
        }
    }
    Ok(false)
}

async fn handle_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> Result<bool> {
    // Handle Ctrl+C globally
    if code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
        return Ok(true);
    }

    match app.mode {
        Mode::Normal => handle_normal_mode(app, code, modifiers).await,
        Mode::Command => handle_command_mode(app, code, modifiers).await,
        Mode::Help => handle_help_mode(app, code),
        Mode::Confirm => handle_confirm_mode(app, code, modifiers).await,
        Mode::Warning => handle_warning_mode(app, code),
        Mode::Describe => handle_describe_mode(app, code, modifiers),
    }
}

async fn handle_normal_mode(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> Result<bool> {
    // Handle gg (go to top) with timing
    if code == KeyCode::Char('g') {
        let now = std::time::Instant::now();
        if let Some((KeyCode::Char('g'), prev_time)) = app.last_key_press {
            if now.duration_since(prev_time) < Duration::from_millis(500) {
                app.go_to_top();
                app.last_key_press = None;
                return Ok(false);
            }
        }
        app.last_key_press = Some((code, now));
        return Ok(false);
    }

    // Reset key tracking for other keys
    app.last_key_press = None;

    match code {
        // Quit
        KeyCode::Char('q') => return Ok(true),

        // Navigation
        KeyCode::Char('j') | KeyCode::Down => app.next(),
        KeyCode::Char('k') | KeyCode::Up => app.previous(),
        KeyCode::Char('G') => app.go_to_bottom(),
        KeyCode::PageDown | KeyCode::Char('f') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.page_down(10);
        }
        KeyCode::PageUp | KeyCode::Char('b') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.page_up(10);
        }

        // Filter
        KeyCode::Char('/') => {
            app.filter_active = true;
        }
        KeyCode::Esc if app.filter_active => {
            app.clear_filter();
        }

        // Handle Ctrl+D for destructive actions (must come before 'd' for describe)
        KeyCode::Char('d') if modifiers.contains(KeyModifiers::CONTROL) => {
            if let Some(resource) = app.current_resource() {
                for action in &resource.actions {
                    if action.shortcut.as_deref() == Some("ctrl+d") {
                        if app.readonly {
                            app.show_warning("Read-only mode: actions are disabled");
                            return Ok(false);
                        }
                        if let Some(item) = app.selected_item() {
                            let resource_id = extract_json_value(item, &resource.id_field);
                            if let Some(pending) = app.create_pending_action(action, &resource_id) {
                                app.enter_confirm_mode(pending);
                            }
                        }
                        return Ok(false);
                    }
                }
            }
        }

        // Describe / Details
        KeyCode::Enter | KeyCode::Char('d') => {
            app.enter_describe_mode().await;
        }

        // Command mode
        KeyCode::Char(':') => {
            app.enter_command_mode();
        }

        // Help
        KeyCode::Char('?') => {
            app.enter_help_mode();
        }

        // Refresh
        KeyCode::Char('R') => {
            app.refresh_current().await?;
        }

        // Back navigation
        KeyCode::Char('b') | KeyCode::Backspace => {
            app.navigate_back().await?;
        }

        // Handle sub-resource shortcuts
        KeyCode::Char(c) => {
            if let Some(resource) = app.current_resource() {
                for sub in &resource.sub_resources {
                    if sub.shortcut == c.to_string() && app.selected_item().is_some() {
                        app.navigate_to_sub_resource(&sub.resource_key).await?;
                        return Ok(false);
                    }
                }

                // Handle action shortcuts
                for action in &resource.actions {
                    if action.shortcut.as_deref() == Some(&c.to_string()) {
                        if app.readonly && action.sdk_method != "get" {
                            app.show_warning("Read-only mode: actions are disabled");
                            return Ok(false);
                        }
                        if let Some(item) = app.selected_item() {
                            let resource_id = extract_json_value(item, &resource.id_field);
                            if let Some(pending) = app.create_pending_action(action, &resource_id) {
                                app.enter_confirm_mode(pending);
                            }
                        }
                        return Ok(false);
                    }
                }
            }
        }

        _ => {}
    }

    // Handle filter input
    if app.filter_active {
        match code {
            KeyCode::Char(c) => {
                app.filter_text.push(c);
                app.apply_filter();
            }
            KeyCode::Backspace => {
                app.filter_text.pop();
                app.apply_filter();
            }
            KeyCode::Enter => {
                app.filter_active = false;
            }
            _ => {}
        }
    }

    Ok(false)
}

async fn handle_command_mode(
    app: &mut App,
    code: KeyCode,
    _modifiers: KeyModifiers,
) -> Result<bool> {
    match code {
        KeyCode::Esc => {
            app.exit_mode();
        }
        KeyCode::Enter => {
            let should_quit = app.execute_command().await?;
            app.exit_mode();
            return Ok(should_quit);
        }
        KeyCode::Char(c) => {
            app.command_text.push(c);
            app.update_command_suggestions();
        }
        KeyCode::Backspace => {
            app.command_text.pop();
            app.update_command_suggestions();
        }
        KeyCode::Tab | KeyCode::Down => {
            app.next_suggestion();
        }
        KeyCode::BackTab | KeyCode::Up => {
            app.prev_suggestion();
        }
        KeyCode::Right => {
            app.apply_suggestion();
        }
        _ => {}
    }
    Ok(false)
}

fn handle_help_mode(app: &mut App, code: KeyCode) -> Result<bool> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') | KeyCode::Enter => {
            app.exit_mode();
        }
        _ => {}
    }
    Ok(false)
}

async fn handle_confirm_mode(
    app: &mut App,
    code: KeyCode,
    _modifiers: KeyModifiers,
) -> Result<bool> {
    match code {
        KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
            app.exit_mode();
        }
        KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
            if let Some(ref mut pending) = app.pending_action {
                pending.selected_yes = !pending.selected_yes;
            }
        }
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            execute_pending_action(app).await?;
            app.exit_mode();
        }
        KeyCode::Enter => {
            if app
                .pending_action
                .as_ref()
                .map(|p| p.selected_yes)
                .unwrap_or(false)
            {
                execute_pending_action(app).await?;
            }
            app.exit_mode();
        }
        _ => {}
    }
    Ok(false)
}

fn handle_warning_mode(app: &mut App, code: KeyCode) -> Result<bool> {
    match code {
        KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
            app.warning_message = None;
            app.exit_mode();
        }
        _ => {}
    }
    Ok(false)
}

fn handle_describe_mode(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> Result<bool> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('d') => {
            app.exit_mode();
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.describe_scroll = app.describe_scroll.saturating_add(1);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.describe_scroll = app.describe_scroll.saturating_sub(1);
        }
        KeyCode::Char('g') => {
            app.describe_scroll = 0;
        }
        KeyCode::Char('G') => {
            app.describe_scroll_to_bottom(20);
        }
        KeyCode::PageDown | KeyCode::Char('f') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.describe_scroll = app.describe_scroll.saturating_add(20);
        }
        KeyCode::PageUp | KeyCode::Char('b') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.describe_scroll = app.describe_scroll.saturating_sub(20);
        }
        _ => {}
    }
    Ok(false)
}

async fn execute_pending_action(app: &mut App) -> Result<()> {
    let Some(pending) = app.pending_action.take() else {
        return Ok(());
    };

    app.loading = true;

    let params = serde_json::json!({
        "id": pending.resource_id.parse::<i32>().unwrap_or(0)
    });

    match invoke_sdk_method(&pending.service, &pending.sdk_method, &app.client, &params).await {
        Ok(_) => {
            // Refresh after action
            let _ = app.refresh_current().await;
        }
        Err(e) => {
            app.error_message = Some(crate::one::client::format_one_error(&e));
        }
    }

    app.loading = false;
    Ok(())
}
