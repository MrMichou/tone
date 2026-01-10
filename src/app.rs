//! Application State
//!
//! Central application state management for tone.

use crate::config::Config;
use crate::one::OneClient;
use crate::resource::{
    extract_json_value, fetch_resources_paginated, get_all_resource_keys, get_resource,
    ResourceDef, ResourceFilter,
};
use anyhow::Result;
use crossterm::event::KeyCode;
use serde_json::Value;

/// Application modes
#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,   // Viewing list
    Command,  // : command input
    Help,     // ? help popup
    Confirm,  // Confirmation dialog
    Warning,  // Warning/info dialog (OK only)
    Describe, // Viewing JSON details of selected item
}

/// Pending action that requires confirmation
#[derive(Debug, Clone)]
pub struct PendingAction {
    pub service: String,
    pub sdk_method: String,
    pub resource_id: String,
    pub message: String,
    #[allow(dead_code)]
    pub default_no: bool,
    pub destructive: bool,
    pub selected_yes: bool,
}

/// Parent context for hierarchical navigation
#[derive(Debug, Clone)]
pub struct ParentContext {
    pub resource_key: String,
    pub item: Value,
    pub display_name: String,
}

/// Pagination state
#[derive(Debug, Clone, Default)]
pub struct PaginationState {
    pub next_token: Option<String>,
    pub token_stack: Vec<Option<String>>,
    pub current_page: usize,
    pub has_more: bool,
}

/// Main application state
pub struct App {
    // OpenNebula Client
    pub client: OneClient,

    // Current resource being viewed
    pub current_resource_key: String,

    // Dynamic data storage (JSON)
    pub items: Vec<Value>,
    pub filtered_items: Vec<Value>,

    // Navigation state
    pub selected: usize,
    pub mode: Mode,
    pub filter_text: String,
    pub filter_active: bool,

    // Hierarchical navigation
    pub parent_context: Option<ParentContext>,
    pub navigation_stack: Vec<ParentContext>,

    // Command input
    pub command_text: String,
    pub command_suggestions: Vec<String>,
    pub command_suggestion_selected: usize,
    pub command_preview: Option<String>,

    // Confirmation
    pub pending_action: Option<PendingAction>,

    // UI state
    pub loading: bool,
    pub error_message: Option<String>,
    pub describe_scroll: usize,
    pub describe_data: Option<Value>,

    // Auto-refresh
    pub last_refresh: std::time::Instant,

    // Persistent configuration
    pub config: Config,

    // Key press tracking
    pub last_key_press: Option<(KeyCode, std::time::Instant)>,

    // Read-only mode
    pub readonly: bool,

    // Warning message
    pub warning_message: Option<String>,

    // Pagination
    pub pagination: PaginationState,

    // Endpoint info
    pub endpoint: String,
    pub username: String,
}

impl App {
    /// Create App from pre-initialized components
    #[allow(clippy::too_many_arguments)]
    pub fn from_initialized(
        client: OneClient,
        initial_items: Vec<Value>,
        config: Config,
        readonly: bool,
    ) -> Self {
        let filtered_items = initial_items.clone();
        let endpoint = client.credentials.endpoint.clone();
        let username = client.credentials.username.clone();

        Self {
            client,
            current_resource_key: "one-vms".to_string(),
            items: initial_items,
            filtered_items,
            selected: 0,
            mode: Mode::Normal,
            filter_text: String::new(),
            filter_active: false,
            parent_context: None,
            navigation_stack: Vec::new(),
            command_text: String::new(),
            command_suggestions: Vec::new(),
            command_suggestion_selected: 0,
            command_preview: None,
            pending_action: None,
            loading: false,
            error_message: None,
            describe_scroll: 0,
            describe_data: None,
            last_refresh: std::time::Instant::now(),
            config,
            last_key_press: None,
            readonly,
            warning_message: None,
            pagination: PaginationState::default(),
            endpoint,
            username,
        }
    }

    /// Check if auto-refresh is needed (disabled)
    pub fn needs_refresh(&self) -> bool {
        false
    }

    /// Reset refresh timer
    pub fn mark_refreshed(&mut self) {
        self.last_refresh = std::time::Instant::now();
    }

    // =========================================================================
    // Resource Definition Access
    // =========================================================================

    pub fn current_resource(&self) -> Option<&'static ResourceDef> {
        get_resource(&self.current_resource_key)
    }

    pub fn get_available_commands(&self) -> Vec<String> {
        let mut commands: Vec<String> = get_all_resource_keys()
            .iter()
            .map(|s| s.to_string())
            .collect();

        commands.sort();
        commands
    }

    // =========================================================================
    // Data Fetching
    // =========================================================================

    pub async fn refresh_current(&mut self) -> Result<()> {
        self.fetch_page(self.pagination.next_token.clone()).await
    }

    async fn fetch_page(&mut self, page_token: Option<String>) -> Result<()> {
        if self.current_resource().is_none() {
            self.error_message = Some(format!("Unknown resource: {}", self.current_resource_key));
            return Ok(());
        }

        self.loading = true;
        self.error_message = None;

        let filters = self.build_filters_from_context();

        match fetch_resources_paginated(
            &self.current_resource_key,
            &self.client,
            &filters,
            page_token.as_deref(),
        )
        .await
        {
            Ok(result) => {
                let prev_selected = self.selected;
                self.items = result.items;
                self.apply_filter();

                self.pagination.has_more = result.next_token.is_some();
                self.pagination.next_token = result.next_token;

                if prev_selected < self.filtered_items.len() {
                    self.selected = prev_selected;
                } else {
                    self.selected = 0;
                }
            }
            Err(e) => {
                self.error_message = Some(crate::one::client::format_one_error(&e));
                self.items.clear();
                self.filtered_items.clear();
                self.selected = 0;
                self.pagination = PaginationState::default();
            }
        }

        self.loading = false;
        self.mark_refreshed();
        Ok(())
    }

    pub async fn next_page(&mut self) -> Result<()> {
        if !self.pagination.has_more {
            return Ok(());
        }

        let current_token = self.pagination.next_token.clone();
        self.pagination.token_stack.push(current_token.clone());
        self.pagination.current_page += 1;

        self.fetch_page(current_token).await
    }

    pub async fn prev_page(&mut self) -> Result<()> {
        if self.pagination.current_page <= 1 {
            return Ok(());
        }

        self.pagination.token_stack.pop();
        let prev_token = self.pagination.token_stack.pop().flatten();
        self.pagination.current_page -= 1;

        self.fetch_page(prev_token).await
    }

    pub fn reset_pagination(&mut self) {
        self.pagination = PaginationState::default();
    }

    fn build_filters_from_context(&self) -> Vec<ResourceFilter> {
        let Some(parent) = &self.parent_context else {
            return Vec::new();
        };

        if let Some(parent_resource) = get_resource(&parent.resource_key) {
            for sub in &parent_resource.sub_resources {
                if sub.resource_key == self.current_resource_key {
                    let parent_id = extract_json_value(&parent.item, &sub.parent_id_field);
                    if parent_id != "-" {
                        return vec![ResourceFilter::new(&sub.filter_param, vec![parent_id])];
                    }
                }
            }
        }

        Vec::new()
    }

    // =========================================================================
    // Filtering
    // =========================================================================

    pub fn apply_filter(&mut self) {
        let filter = self.filter_text.to_lowercase();

        if filter.is_empty() {
            self.filtered_items = self.items.clone();
        } else {
            let resource = self.current_resource();
            self.filtered_items = self
                .items
                .iter()
                .filter(|item| {
                    if let Some(res) = resource {
                        let name = extract_json_value(item, &res.name_field).to_lowercase();
                        let id = extract_json_value(item, &res.id_field).to_lowercase();
                        name.contains(&filter) || id.contains(&filter)
                    } else {
                        item.to_string().to_lowercase().contains(&filter)
                    }
                })
                .cloned()
                .collect();
        }

        if self.selected >= self.filtered_items.len() && !self.filtered_items.is_empty() {
            self.selected = self.filtered_items.len() - 1;
        }
    }

    pub fn clear_filter(&mut self) {
        self.filter_text.clear();
        self.filter_active = false;
        self.apply_filter();
    }

    // =========================================================================
    // Navigation
    // =========================================================================

    pub fn selected_item(&self) -> Option<&Value> {
        self.filtered_items.get(self.selected)
    }

    pub fn selected_item_json(&self) -> Option<String> {
        if let Some(ref data) = self.describe_data {
            return Some(serde_json::to_string_pretty(data).unwrap_or_default());
        }
        self.selected_item()
            .map(|item| serde_json::to_string_pretty(item).unwrap_or_default())
    }

    pub fn describe_line_count(&self) -> usize {
        self.selected_item_json()
            .map(|s| s.lines().count())
            .unwrap_or(0)
    }

    pub fn describe_scroll_to_bottom(&mut self, visible_lines: usize) {
        let total = self.describe_line_count();
        self.describe_scroll = total.saturating_sub(visible_lines);
    }

    pub fn next(&mut self) {
        if !self.filtered_items.is_empty() {
            self.selected = (self.selected + 1).min(self.filtered_items.len() - 1);
        }
    }

    pub fn previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn go_to_top(&mut self) {
        self.selected = 0;
    }

    pub fn go_to_bottom(&mut self) {
        if !self.filtered_items.is_empty() {
            self.selected = self.filtered_items.len() - 1;
        }
    }

    pub fn page_down(&mut self, page_size: usize) {
        if !self.filtered_items.is_empty() {
            self.selected = (self.selected + page_size).min(self.filtered_items.len() - 1);
        }
    }

    pub fn page_up(&mut self, page_size: usize) {
        self.selected = self.selected.saturating_sub(page_size);
    }

    // =========================================================================
    // Mode Transitions
    // =========================================================================

    pub fn enter_command_mode(&mut self) {
        self.mode = Mode::Command;
        self.command_text.clear();
        self.command_suggestions = self.get_available_commands();
        self.command_suggestion_selected = 0;
        self.command_preview = None;
    }

    pub fn update_command_suggestions(&mut self) {
        let input = self.command_text.to_lowercase();
        let all_commands = self.get_available_commands();

        if input.is_empty() {
            self.command_suggestions = all_commands;
        } else {
            self.command_suggestions = all_commands
                .into_iter()
                .filter(|cmd| cmd.contains(&input))
                .collect();
        }

        if self.command_suggestion_selected >= self.command_suggestions.len() {
            self.command_suggestion_selected = 0;
        }

        self.update_preview();
    }

    fn update_preview(&mut self) {
        if self.command_suggestions.is_empty() {
            self.command_preview = None;
        } else {
            self.command_preview = self
                .command_suggestions
                .get(self.command_suggestion_selected)
                .cloned();
        }
    }

    pub fn next_suggestion(&mut self) {
        if !self.command_suggestions.is_empty() {
            self.command_suggestion_selected =
                (self.command_suggestion_selected + 1) % self.command_suggestions.len();
            self.update_preview();
        }
    }

    pub fn prev_suggestion(&mut self) {
        if !self.command_suggestions.is_empty() {
            if self.command_suggestion_selected == 0 {
                self.command_suggestion_selected = self.command_suggestions.len() - 1;
            } else {
                self.command_suggestion_selected -= 1;
            }
            self.update_preview();
        }
    }

    pub fn apply_suggestion(&mut self) {
        if let Some(preview) = &self.command_preview {
            self.command_text = preview.clone();
            self.update_command_suggestions();
        }
    }

    pub fn enter_help_mode(&mut self) {
        self.mode = Mode::Help;
    }

    pub async fn enter_describe_mode(&mut self) {
        if self.filtered_items.is_empty() {
            return;
        }

        self.mode = Mode::Describe;
        self.describe_scroll = 0;
        self.describe_data = None;

        if let Some(item) = self.selected_item().cloned() {
            self.describe_data = Some(item);
        }
    }

    pub fn enter_confirm_mode(&mut self, pending: PendingAction) {
        self.pending_action = Some(pending);
        self.mode = Mode::Confirm;
    }

    pub fn show_warning(&mut self, message: &str) {
        self.warning_message = Some(message.to_string());
        self.mode = Mode::Warning;
    }

    pub fn create_pending_action(
        &self,
        action: &crate::resource::ActionDef,
        resource_id: &str,
    ) -> Option<PendingAction> {
        let config = action.get_confirm_config()?;
        let resource_name = self
            .selected_item()
            .and_then(|item| {
                if let Some(resource_def) = self.current_resource() {
                    let name = extract_json_value(item, &resource_def.name_field);
                    if name != "-" && !name.is_empty() {
                        return Some(name);
                    }
                }
                None
            })
            .unwrap_or_else(|| resource_id.to_string());

        let message = config
            .message
            .unwrap_or_else(|| action.display_name.clone());
        let default_no = !config.default_yes;

        Some(PendingAction {
            service: self.current_resource()?.service.clone(),
            sdk_method: action.sdk_method.clone(),
            resource_id: resource_id.to_string(),
            message: format!("{} '{}'?", message, resource_name),
            default_no,
            destructive: config.destructive,
            selected_yes: config.default_yes,
        })
    }

    pub fn exit_mode(&mut self) {
        self.mode = Mode::Normal;
        self.pending_action = None;
        self.describe_data = None;
    }

    // =========================================================================
    // Resource Navigation
    // =========================================================================

    pub async fn navigate_to_resource(&mut self, resource_key: &str) -> Result<()> {
        if get_resource(resource_key).is_none() {
            self.error_message = Some(format!("Unknown resource: {}", resource_key));
            return Ok(());
        }

        self.parent_context = None;
        self.navigation_stack.clear();
        self.current_resource_key = resource_key.to_string();
        self.selected = 0;
        self.filter_text.clear();
        self.filter_active = false;
        self.mode = Mode::Normal;

        self.reset_pagination();
        self.refresh_current().await?;
        Ok(())
    }

    pub async fn navigate_to_sub_resource(&mut self, sub_resource_key: &str) -> Result<()> {
        let Some(selected_item) = self.selected_item().cloned() else {
            return Ok(());
        };

        let Some(current_resource) = self.current_resource() else {
            return Ok(());
        };

        let is_valid = current_resource
            .sub_resources
            .iter()
            .any(|s| s.resource_key == sub_resource_key);

        if !is_valid {
            self.error_message = Some(format!(
                "{} is not a sub-resource of {}",
                sub_resource_key, self.current_resource_key
            ));
            return Ok(());
        }

        let display_name = extract_json_value(&selected_item, &current_resource.name_field);
        let id = extract_json_value(&selected_item, &current_resource.id_field);
        let display = if display_name != "-" {
            display_name
        } else {
            id
        };

        if let Some(ctx) = self.parent_context.take() {
            self.navigation_stack.push(ctx);
        }

        self.parent_context = Some(ParentContext {
            resource_key: self.current_resource_key.clone(),
            item: selected_item,
            display_name: display,
        });

        self.current_resource_key = sub_resource_key.to_string();
        self.selected = 0;
        self.filter_text.clear();
        self.filter_active = false;

        self.reset_pagination();
        self.refresh_current().await?;
        Ok(())
    }

    pub async fn navigate_back(&mut self) -> Result<()> {
        if let Some(parent) = self.parent_context.take() {
            self.parent_context = self.navigation_stack.pop();
            self.current_resource_key = parent.resource_key;
            self.selected = 0;
            self.filter_text.clear();
            self.filter_active = false;

            self.reset_pagination();
            self.refresh_current().await?;
        }
        Ok(())
    }

    pub fn get_breadcrumb(&self) -> Vec<String> {
        let mut path = Vec::new();

        for ctx in &self.navigation_stack {
            path.push(format!("{}:{}", ctx.resource_key, ctx.display_name));
        }

        if let Some(ctx) = &self.parent_context {
            path.push(format!("{}:{}", ctx.resource_key, ctx.display_name));
        }

        path.push(self.current_resource_key.clone());
        path
    }

    // =========================================================================
    // Command Execution
    // =========================================================================

    pub async fn execute_command(&mut self) -> Result<bool> {
        let command_text = if self.command_text.is_empty() {
            self.command_preview.clone().unwrap_or_default()
        } else if let Some(preview) = &self.command_preview {
            if preview.contains(&self.command_text) {
                preview.clone()
            } else {
                self.command_text.clone()
            }
        } else {
            self.command_text.clone()
        };

        let parts: Vec<&str> = command_text.split_whitespace().collect();

        if parts.is_empty() {
            return Ok(false);
        }

        let cmd = parts[0];

        match cmd {
            "q" | "quit" => return Ok(true),
            "back" => {
                self.navigate_back().await?;
            }
            _ => {
                if get_resource(cmd).is_some() {
                    if let Some(resource) = self.current_resource() {
                        let is_sub = resource.sub_resources.iter().any(|s| s.resource_key == cmd);
                        if is_sub && self.selected_item().is_some() {
                            self.navigate_to_sub_resource(cmd).await?;
                        } else {
                            self.navigate_to_resource(cmd).await?;
                        }
                    } else {
                        self.navigate_to_resource(cmd).await?;
                    }
                } else {
                    self.error_message = Some(format!("Unknown command: {}", cmd));
                }
            }
        }

        Ok(false)
    }
}
