//! Application-level UI state types.
//!
//! This module contains state definitions for the application-level UI components,
//! separated from the core application state to achieve clean separation of concerns.

use crate::modules::project::domain::Project;

/// State for the home screen project browser.
/// Moved here from app.rs to separate UI state from application state.
pub struct HomeScreenState {
    /// All loaded projects from the core database.
    pub projects: Vec<Project>,
    /// Current value of the search input filter.
    pub search_query: String,
}

/// Default implementation for HomeScreenState.
/// Initializes with empty projects list and empty search query.
impl Default for HomeScreenState {
    fn default() -> Self {
        Self {
            projects: vec![],
            search_query: String::new(),
        }
    }
}
