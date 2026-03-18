//! Top-level iced application state, messages, and handlers.
//!
//! ## Architectural Rules (enforced here)
//! - This is the ONLY file that composes domain module Messages into the root Message enum.
//! - No business logic lives in this file — delegate everything to modules.
//! - `App` owns the `CoreDb`; repositories receive a cloned reference.
//! - Module messages are wrapped as variants: e.g., Message::Foo(modules::foo::Message).

use anyhow::{anyhow, Result};
use iced::widget::{column, container, text};
use iced::time::Duration;
use iced::{Element, Fill, Subscription, Task};
use std::path::PathBuf;
use std::time::Instant;

use crate::db::CoreDb;
use crate::kernel::loading::{loading_indicator, LoadingState, MIN_LOADING_DURATION};

// ─── STATE ───────────────────────────────────────────────────────────────────

/// Top-level application state.
///
/// `App` must be `'static` — iced requires no non-`'static` references in state.
pub struct App {
    /// The core database handle. `None` until the async init task completes.
    core_db: Option<CoreDb>,

    /// Holds a startup error message if DB initialization fails.
    /// Displayed instead of the main UI when `Some`.
    init_error: Option<String>,

    /// Tracks the three-phase loading lifecycle (Idle → Loading → Done).
    loading_state: LoadingState,

    /// Increments on every `Tick` for spinner animation frame selection.
    tick_count: u32,
}

impl Default for App {
    fn default() -> Self {
        Self {
            core_db: None,
            init_error: None,
            loading_state: LoadingState::Idle,
            tick_count: 0,
        }
    }
}

// ─── MESSAGES ────────────────────────────────────────────────────────────────

/// Root message enum. All domain module messages are composed here as variants.
#[derive(Debug, Clone)]
pub enum Message {
    /// Sent when async DB init succeeds. Carries the ready `CoreDb`.
    DbReady(CoreDb),

    /// Sent when async DB init fails. Carries the error as a human-readable string.
    DbFailed(String),

    /// Fired by the 100ms subscription while loading; advances the spinner animation frame.
    Tick,

    /// Fired by the one-shot Task after both DB is ready AND 300ms has elapsed.
    /// Transitions `loading_state` to `Done`.
    LoadingDone,
    // Future variants added here as new modules are introduced.
}

// ─── BOOT FUNCTION ───────────────────────────────────────────────────────────

/// Called by iced once at startup. Returns the initial state and a `Task` that
/// resolves the core database asynchronously.
pub fn new() -> (App, Task<Message>) {
    let initial_state = App {
        loading_state: LoadingState::Loading {
            started_at: Instant::now(),
        },
        tick_count: 0,
        core_db: None,
        init_error: None,
    };

    let task = Task::perform(
        async { init_db().await },
        |result| match result {
            Ok(core_db) => Message::DbReady(core_db),
            Err(e) => Message::DbFailed(e.to_string()),
        },
    );

    (initial_state, task)
}

/// Private async helper that opens the core database. Called only by [`new`].
async fn init_db() -> Result<CoreDb> {
    let base = dirs::data_dir().ok_or_else(|| anyhow!("Cannot determine app data directory"))?;
    let app_data_dir: PathBuf = base.join("melange");
    tracing::info!("Initializing core database at {:?}", app_data_dir);
    let core_db = CoreDb::open(&app_data_dir).await?;
    tracing::info!("Core database ready");
    Ok(core_db)
}

// ─── UPDATE FUNCTION ─────────────────────────────────────────────────────────

/// Handle an incoming [`Message`], mutate `state`, and optionally return a
/// follow-up [`Task`].
pub fn update(state: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::DbReady(core_db) => {
            tracing::info!("Core database pool received");
            state.core_db = Some(core_db);

            if let LoadingState::Loading { started_at } = state.loading_state {
                Task::perform(
                    async move {
                        let elapsed = started_at.elapsed();
                        if elapsed < MIN_LOADING_DURATION {
                            tokio::time::sleep(MIN_LOADING_DURATION - elapsed).await;
                        }
                    },
                    |_| Message::LoadingDone,
                )
            } else {
                Task::none()
            }
        }
        Message::DbFailed(err) => {
            tracing::error!("Database initialization failed: {}", err);
            state.init_error = Some(err);
            state.loading_state = LoadingState::Done;
            Task::none()
        }
        Message::Tick => {
            state.tick_count = state.tick_count.wrapping_add(1);
            Task::none()
        }
        Message::LoadingDone => {
            state.loading_state = LoadingState::Done;
            tracing::info!("Loading complete — minimum duration satisfied");
            Task::none()
        }
    }
}

// ─── SUBSCRIPTION FUNCTION ───────────────────────────────────────────────────

/// Returns an active subscription when loading, or `Subscription::none()` otherwise.
///
/// Fires `Message::Tick` every 100ms while `loading_state` is `Loading`, driving
/// the spinner animation. Automatically stops once loading completes.
pub fn subscription(state: &App) -> Subscription<Message> {
    match state.loading_state {
        LoadingState::Loading { .. } => {
            iced::time::every(Duration::from_millis(100)).map(|_| Message::Tick)
        }
        _ => Subscription::none(),
    }
}

// ─── VIEW FUNCTION ───────────────────────────────────────────────────────────

/// Render the application UI based on the current [`App`] state.
pub fn view(state: &App) -> Element<'_, Message> {
    if let Some(ref err) = state.init_error {
        // Centered error message
        container(text(format!("Failed to initialize database: {}", err)))
            .center(Fill)
            .into()
    } else if state.loading_state != LoadingState::Done {
        // Loading state — show animated spinner until minimum duration satisfied
        loading_indicator("Initialising database…", state.tick_count)
    } else {
        // Ready state — main UI placeholder
        container(column![text("Melange").size(32), text("Ready.")].spacing(10))
            .center(Fill)
            .into()
    }
}
