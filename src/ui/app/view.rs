//! Application-level UI view composition.
//!
//! This module contains the view composition logic for the application-level UI,
//! extracted from the main app.rs file to achieve separation of concerns.

use crate::app::Message;
use crate::kernel::loading::LoadingState;
use crate::kernel::opencode::OpencodeStatus;
use crate::modules::project::domain::Project;
use crate::ui::app_bar::app_bar;
use crate::ui::loading::loading_indicator;
use crate::ui::opencode::opencode_not_found_screen;
use crate::ui::project::home_screen;
use iced::widget::{column, container, text};
use iced::{Element, Fill};

/// Render the application UI based on the current App state.
///
/// Priority chain (highest to lowest):
/// 1. opencode not found → hard block screen
/// 2. DB init error → centered error message
/// 3. Still loading OR opencode check not yet resolved → loading spinner
/// 4. Ready → main UI
///
/// Parameters:
/// - opencode_status: Optional<OpencodeStatus> - result of opencode check
/// - init_error: Optional<String> - database initialization error if any
/// - loading_state: LoadingState - current loading lifecycle state
/// - tick_count: u32 - animation frame counter for spinner
/// - projects: reference to slice of Projects - loaded projects list
/// - search_query: reference to str - current search filter
/// - window_width: f32 - current window width for responsive layout
///
/// Returns: Element with Message type
pub fn view_app<'a>(
    opencode_status: Option<OpencodeStatus>,
    init_error: Option<String>,
    loading_state: LoadingState,
    tick_count: u32,
    projects: &'a [Project],
    search_query: &'a str,
    window_width: f32,
) -> Element<'a, Message> {
    // Determine screen_content based on priority logic
    let screen_content: Element<'a, Message> = if opencode_status == Some(OpencodeStatus::NotFound)
    {
        // Priority 1: opencode not found → hard block (highest priority)
        opencode_not_found_screen()
    } else if let Some(ref err) = init_error {
        // Priority 2: DB initialization failed
        container(text(format!("Failed to initialize database: {}", err)))
            .center(Fill)
            .into()
    } else if loading_state != LoadingState::Done || opencode_status.is_none() {
        // Priority 3: still loading OR opencode check not yet resolved
        loading_indicator("Initialising…", tick_count)
    } else {
        // Priority 4: ready — home screen project browser
        home_screen(projects, search_query, window_width).map(Message::Project)
    };

    // Compose app bar above screen content in a column layout
    column![app_bar(), screen_content].into()
}
