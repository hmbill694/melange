//! Application entry point.
//!
//! ## Startup Sequence
//! 1. Initialize tracing subscriber (reads RUST_LOG; defaults to "melange=debug,sqlx=warn")
//! 2. Launch iced application — window opens immediately
//! 3. Async DB init fires via Task::perform in app::new()
//! 4. On DB ready: App state receives the pool and renders the main UI
//!
//! ## Adding New Domain Modules
//! Create `src/modules/<domain>/` following the modulith layout documented in
//! `src/modules/mod.rs`. Declare the submodule in `src/modules/mod.rs` only —
//! never import a module directly from main.rs or from a sibling module.

mod app;
mod db;
mod modules;

use tracing_subscriber::EnvFilter;

fn main() -> iced::Result {
    // Step 1: Initialize tracing BEFORE anything else.
    // Reads RUST_LOG env var; falls back to the default filter string.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("melange=debug,sqlx=warn")),
        )
        .init();

    tracing::info!("Starting Melange");

    // Step 2: Launch the iced application.
    // Pattern: iced::application(boot, update, view).title(...).run()
    //
    // boot   = app::new    → (App, Task<Message>), called once at startup
    // update = app::update → (&mut App, Message) -> Task<Message>
    // view   = app::view   → (&App) -> Element<'_, Message>
    //
    // .title("Melange") sets the OS window title bar
    // .run() blocks until the window is closed; returns iced::Result
    iced::application(app::new, app::update, app::view)
        .title("Melange")
        .run()
}
