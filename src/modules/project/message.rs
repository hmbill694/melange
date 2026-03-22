//! Message types for the project bounded context.
//!
//! `ProjectMessage` is composed into the root `Message` enum in `app.rs` as
//! `Message::Project(ProjectMessage)`.

use crate::modules::project::domain::Project;

/// All messages originating from the project UI and async tasks.
#[derive(Debug, Clone)]
pub enum ProjectMessage {
    /// Fired when the search bar text changes.
    SearchChanged(String),

    /// Fired when the async project-load task succeeds.
    ProjectsLoaded(Vec<Project>),

    /// Fired when the async project-load task fails.
    LoadFailed(String),
}
