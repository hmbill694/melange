//! Top-level iced application state, messages, and handlers.
//!
//! ## Architectural Rules (enforced here)
//! - This is the ONLY file that composes domain module Messages into the root Message enum.
//! - No business logic lives in this file — delegate everything to modules.
//! - `App` owns the `CoreDb`; repositories receive a cloned reference.
//! - Module messages are wrapped as variants: e.g., Message::Foo(modules::foo::Message).

mod service;

use anyhow::Result;
use iced::time::Duration;
use iced::{Element, Subscription, Task};
use std::time::Instant;

use crate::db::CoreDb;
use crate::kernel::loading::LoadingState;
use crate::kernel::opencode::{check_opencode_on_path, OpencodeStatus};
use crate::modules::project::ProjectMessage;
use crate::ui::app::{HomeScreenState, view_app, handle_update, UpdateContext, HomeScreenUpdateContext};


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

    /// Holds the result of the startup opencode binary check.
    /// `None` means the check has not yet completed.
    opencode_status: Option<OpencodeStatus>,

    /// State for the home screen project browser.
    /// Delegated to ui::app::state module.
    home_screen_state: HomeScreenState,

    /// Current window width in logical pixels; drives grid vs. list layout.
    window_width: f32,
}

impl Default for App {
    fn default() -> Self {
        Self {
            core_db: None,
            init_error: None,
            loading_state: LoadingState::Idle,
            tick_count: 0,
            opencode_status: None,
            home_screen_state: HomeScreenState::default(),
            window_width: 1024.0,
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

    /// Sent when the startup opencode check confirms the binary is on PATH.
    OpencodeReady,

    /// Sent when the startup opencode check finds the binary is NOT on PATH.
    OpencodeNotFound,

    /// Wraps all messages originating from the project module UI and tasks.
    Project(ProjectMessage),

    /// Fired by the window-resize subscription; carries the new width in logical pixels.
    WindowResized(f32),
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
        opencode_status: None,
        home_screen_state: HomeScreenState::default(),
        window_width: 1024.0,
    };

    let db_task = Task::perform(
        async { service::init_db().await },
        |result: Result<CoreDb>| match result {
            Ok(core_db) => Message::DbReady(core_db),
            Err(e) => Message::DbFailed(e.to_string()),
        },
    );

    let opencode_task = Task::perform(
        async { check_opencode_on_path().await },
        |status| match status {
            OpencodeStatus::Found => Message::OpencodeReady,
            OpencodeStatus::NotFound => Message::OpencodeNotFound,
        },
    );

    (initial_state, Task::batch([db_task, opencode_task]))
}



// ─── UPDATE FUNCTION ─────────────────────────────────────────────────────────

/// Handle an incoming [`Message`], mutate `state`, and optionally return a
/// follow-up [`Task`].
///
/// This is now a thin wrapper that delegates to ui::app::update::handle_update
/// to keep the application logic separated from the orchestration layer.
pub fn update(state: &mut App, message: Message) -> Task<Message> {
    // Create update contexts from App state
    let mut app_context = UpdateContext {
        core_db: state.core_db.clone(),
        init_error: state.init_error.clone(),
        loading_state: state.loading_state.clone(),
        tick_count: state.tick_count,
        opencode_status: state.opencode_status.clone(),
        window_width: state.window_width,
    };
    
    let mut home_context = HomeScreenUpdateContext {
        projects: state.home_screen_state.projects.clone(),
        search_query: state.home_screen_state.search_query.clone(),
    };
    
    // Delegate to the dedicated update handler
    let task = handle_update(&mut app_context, &mut home_context, message);
    
    // Sync context changes back to App state
    // (Fields that might have been modified by handle_update)
    state.core_db = app_context.core_db;
    state.init_error = app_context.init_error;
    state.loading_state = app_context.loading_state;
    state.tick_count = app_context.tick_count;
    state.opencode_status = app_context.opencode_status;
    state.window_width = app_context.window_width;
    state.home_screen_state.projects = home_context.projects;
    state.home_screen_state.search_query = home_context.search_query;
    
    task
}

// ─── SUBSCRIPTION FUNCTION ───────────────────────────────────────────────────

/// Returns an active subscription when loading, or `Subscription::none()` otherwise.
///
/// Fires `Message::Tick` every 100ms while `loading_state` is `Loading`, driving
/// the spinner animation. Automatically stops once loading completes.
pub fn subscription(state: &App) -> Subscription<Message> {
    let tick_sub = match state.loading_state {
        LoadingState::Loading { .. } => {
            iced::time::every(Duration::from_millis(100)).map(|_| Message::Tick)
        }
        _ => Subscription::none(),
    };

    // Subscribe to window resize events to keep `window_width` in sync.
    let resize_sub = iced::window::resize_events()
        .map(|(_, size)| Message::WindowResized(size.width));

    Subscription::batch([tick_sub, resize_sub])
}

// ─── VIEW FUNCTION ───────────────────────────────────────────────────────────

/// Render the application UI based on the current [`App`] state.
///
/// Priority chain (highest to lowest):
/// 1. opencode not found → hard block screen
/// 2. DB init error → centered error message
/// 3. Still loading OR opencode check not yet resolved → loading spinner
/// 4. Ready → main UI
///
/// This is now a thin wrapper that delegates to ui::app::view::view_app
/// to keep view composition logic separated from the orchestration layer.
pub fn view(state: &App) -> Element<'_, Message> {
    // Delegate to the dedicated view function
    view_app(
        state.opencode_status.clone(),
        state.init_error.clone(),
        state.loading_state.clone(),
        state.tick_count,
        &state.home_screen_state.projects,
        &state.home_screen_state.search_query,
        state.window_width,
    )
}
