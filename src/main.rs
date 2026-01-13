//! tone - Terminal UI for OpenNebula
//!
//! A terminal user interface for navigating, observing, and managing
//! OpenNebula cloud resources.

mod app;
mod event;
mod one;
mod resource;
mod ui;

/// Version injected at compile time via TONE_VERSION env var (set by CI/CD),
/// or "dev" for local builds.
pub const VERSION: &str = match option_env!("TONE_VERSION") {
    Some(v) => v,
    None => "dev",
};

use anyhow::Result;
use app::App;
use clap::{Parser, ValueEnum};
use crossterm::{
    event::{poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;
use std::path::PathBuf;
use std::time::Duration;
use tracing::Level;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use ui::splash::{render as render_splash, SplashState};

/// Terminal UI for OpenNebula
#[derive(Parser, Debug)]
#[command(name = "tone", version, about, long_about = None)]
struct Args {
    /// OpenNebula XML-RPC endpoint URL (also reads from ONE_XMLRPC env var)
    #[arg(short, long)]
    endpoint: Option<String>,

    /// Log level for debugging
    #[arg(long, value_enum, default_value = "off")]
    log_level: LogLevel,

    /// Run in read-only mode (block all write operations)
    #[arg(long)]
    readonly: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum LogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    fn to_tracing_level(self) -> Option<Level> {
        match self {
            LogLevel::Off => None,
            LogLevel::Error => Some(Level::ERROR),
            LogLevel::Warn => Some(Level::WARN),
            LogLevel::Info => Some(Level::INFO),
            LogLevel::Debug => Some(Level::DEBUG),
            LogLevel::Trace => Some(Level::TRACE),
        }
    }
}

fn setup_logging(level: LogLevel) -> Option<tracing_appender::non_blocking::WorkerGuard> {
    let tracing_level = level.to_tracing_level()?;

    let log_path = get_log_path();

    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .expect("Failed to open log file");

    let (non_blocking, guard) = tracing_appender::non_blocking(file);

    tracing_subscriber::fmt()
        .with_max_level(tracing_level)
        .with_writer(non_blocking.with_max_level(tracing_level))
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .init();

    tracing::info!("tone started with log level: {:?}", level);
    tracing::info!("Log file: {:?}", log_path);

    Some(guard)
}

fn get_log_path() -> PathBuf {
    if let Some(config_dir) = dirs::config_dir() {
        return config_dir.join("tone").join("tone.log");
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".tone").join("tone.log");
    }
    PathBuf::from("tone.log")
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let _log_guard = setup_logging(args.log_level);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize and run
    let result = initialize_with_splash(&mut terminal, &args).await;

    match result {
        Ok(Some(mut app)) => {
            let run_result = run_app(&mut terminal, &mut app).await;
            cleanup_terminal(&mut terminal)?;

            if let Err(err) = run_result {
                eprintln!("Error: {err:?}");
            }
        }
        Ok(None) => {
            cleanup_terminal(&mut terminal)?;
        }
        Err(err) => {
            cleanup_terminal(&mut terminal)?;
            eprintln!("Initialization error: {err:?}");
        }
    }

    Ok(())
}

fn cleanup_terminal<B: Backend + std::io::Write>(terminal: &mut Terminal<B>) -> Result<()>
where
    B::Error: Send + Sync + 'static,
{
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

async fn initialize_with_splash<B: Backend>(
    terminal: &mut Terminal<B>,
    args: &Args,
) -> Result<Option<App>>
where
    B::Error: Send + Sync + 'static,
{
    let mut splash = SplashState::new();

    // Render initial splash
    terminal.draw(|f| render_splash(f, &splash))?;

    if check_abort()? {
        return Ok(None);
    }

    // Step 1: Initialize OpenNebula client
    splash.set_message("Connecting to OpenNebula...");
    terminal.draw(|f| render_splash(f, &splash))?;

    let client = if let Some(ref endpoint) = args.endpoint {
        one::OneClient::with_endpoint(endpoint).await?
    } else {
        one::OneClient::new().await?
    };

    tracing::info!(
        "Connected to OpenNebula at {} as {}",
        client.credentials.endpoint,
        client.credentials.username
    );

    splash.complete_step();

    if check_abort()? {
        return Ok(None);
    }

    // Step 3: Fetch initial data (VMs)
    splash.set_message("Fetching virtual machines...");
    terminal.draw(|f| render_splash(f, &splash))?;

    let (vms, initial_error) = {
        match resource::fetch_resources("one-vms", &client, &[]).await {
            Ok(items) => (items, None),
            Err(e) => {
                let error_msg = one::client::format_one_error(&e);
                (Vec::new(), Some(error_msg))
            }
        }
    };

    splash.complete_step();
    splash.set_message("Ready!");
    terminal.draw(|f| render_splash(f, &splash))?;

    tokio::time::sleep(Duration::from_millis(200)).await;

    let mut app = App::from_initialized(client, vms, args.readonly);

    if let Some(err) = initial_error {
        app.error_message = Some(err);
    }

    Ok(Some(app))
}

fn check_abort() -> Result<bool> {
    if poll(Duration::from_millis(50))? {
        if let Event::Key(key) = read()? {
            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()>
where
    B::Error: Send + Sync + 'static,
{
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if event::handle_events(app).await? {
            return Ok(());
        }

        // Auto-refresh (disabled by default)
        if app.needs_refresh() {
            let _ = app.refresh_current().await;
        }
    }
}
