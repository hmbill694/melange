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

    /// Navigation: Switch from Home screen to Create Project page.
    NavigateToCreateProject,

    /// Navigation: Return to Home screen from Create Project page.
    NavigateToHome,

    /// Form field update: Project name input changed.
    CreateProjectNameChanged(String),

    /// Form field update: File path input changed.
    CreateProjectPathChanged(String),

    /// Form submission: User clicked "Create Project" button.
    CreateProjectSubmitted,

    /// Form submission success: Project was created successfully.
    CreateProjectSucceeded(Project),

    /// Form submission failure: Project creation failed.
    CreateProjectFailed(String),

    /// Browse button clicked: request to open file picker dialog.
    BrowseForFilePath,

    /// Async result from file picker: contains selected path or None if cancelled.
    FilePathSelected(Option<String>),
}
