//! Application-level UI state types.
//!
//! This module contains state definitions for the application-level UI components,
//! separated from the core application state to achieve clean separation of concerns.

use crate::modules::project::domain::Project;

/// Represents which screen is currently displayed.
#[derive(Debug, Clone, PartialEq)]
pub enum CurrentScreen {
    /// Show the home screen with project list and search.
    Home,
    /// Show the Create Project form page.
    CreateProject,
}

impl Default for CurrentScreen {
    fn default() -> Self {
        CurrentScreen::Home
    }
}

/// State for the Create Project form.
#[derive(Debug, Clone)]
pub struct CreateProjectState {
    /// Current value of the project name input field.
    pub project_name: String,
    /// Current value of the file path input field.
    pub file_path: String,
    /// Whether form submission is currently in progress.
    pub is_submitting: bool,
    /// Optional error message to display to the user.
    pub error_message: Option<String>,
}

impl Default for CreateProjectState {
    fn default() -> Self {
        Self {
            project_name: String::new(),
            file_path: String::new(),
            is_submitting: false,
            error_message: None,
        }
    }
}

/// State for the home screen project browser.
/// Moved here from app.rs to separate UI state from application state.
pub struct HomeScreenState {
    /// All loaded projects from the core database.
    pub projects: Vec<Project>,
    /// Current value of the search input filter.
    pub search_query: String,
    /// Tracks which screen is currently displayed.
    pub current_screen: CurrentScreen,
    /// Form state for the Create Project screen.
    pub create_project_state: CreateProjectState,
}

/// Default implementation for HomeScreenState.
/// Initializes with empty projects list and empty search query.
impl Default for HomeScreenState {
    fn default() -> Self {
        Self {
            projects: vec![],
            search_query: String::new(),
            current_screen: CurrentScreen::default(),
            create_project_state: CreateProjectState::default(),
        }
    }
}
