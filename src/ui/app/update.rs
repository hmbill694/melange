//! Application-level UI update handlers.
//!
//! This module contains the update message handlers for the application-level UI,
//! extracted from the main app.rs file to achieve separation of concerns.

use crate::db::CoreDb;
use crate::kernel::loading::{LoadingState, MIN_LOADING_DURATION};
use crate::kernel::opencode::OpencodeStatus;
use crate::modules::project::{ProjectMessage, SqliteProjectRepository};
use crate::modules::project::domain::Project;
use crate::modules::project::service::ProjectService;
use crate::app::Message;
use iced::Task;

use std::path::PathBuf;
use tracing;

/// Application state fields that update handlers need access to.
/// This is a subset of App fields to minimize coupling.
pub struct UpdateContext {
    pub core_db: Option<CoreDb>,
    pub init_error: Option<String>,
    pub loading_state: LoadingState,
    pub tick_count: u32,
    pub opencode_status: Option<OpencodeStatus>,
    pub window_width: f32,
}

/// Home screen state fields for update handlers.
pub struct HomeScreenUpdateContext {
    pub projects: Vec<Project>,
    pub search_query: String,
    pub current_screen: crate::ui::app::state::CurrentScreen,
    pub create_project_state: crate::ui::app::state::CreateProjectState,
}

/// Handle an incoming Message, mutate state, and optionally return a follow-up Task.
///
/// Parameters:
/// - context: mutable reference to UpdateContext (app-level state)
/// - home_context: mutable reference to HomeScreenUpdateContext (home screen state)
/// - message: the Message to handle
///
/// Returns: Task<Message> - follow-up tasks or Task::none()
pub fn handle_update(
    context: &mut UpdateContext,
    home_context: &mut HomeScreenUpdateContext,
    message: Message,
) -> Task<Message> {
    
    match message {
        Message::DbReady(core_db) => {
            tracing::info!("Core database pool received");
            context.core_db = Some(core_db.clone());
            
            // Create loading completion task based on elapsed time
            let loading_task = 
                if let LoadingState::Loading { started_at } = context.loading_state {
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
                };

            // Create project list load task
            let repo = SqliteProjectRepository::new(core_db);
            let app_data_dir = dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("melange");
            let service = ProjectService::new(repo, app_data_dir);
            
            let projects_task = Task::perform(
                async move { service.list_projects().await },
                |result| match result {
                    Ok(projects) => Message::Project(ProjectMessage::ProjectsLoaded(projects)),
                    Err(e) => Message::Project(ProjectMessage::LoadFailed(e.to_string())),
                },
            );

            Task::batch([loading_task, projects_task])
        }
        
        Message::DbFailed(err) => {
            tracing::error!("Database initialization failed: {}", err);
            context.init_error = Some(err);
            context.loading_state = LoadingState::Done;
            Task::none()
        }
        
        Message::Tick => {
            context.tick_count = context.tick_count.wrapping_add(1);
            Task::none()
        }
        
        Message::LoadingDone => {
            context.loading_state = LoadingState::Done;
            tracing::info!("Loading complete — minimum duration satisfied");
            Task::none()
        }
        
        Message::OpencodeReady => {
            tracing::info!("opencode binary found on PATH");
            context.opencode_status = Some(OpencodeStatus::Found);
            Task::none()
        }
        
        Message::OpencodeNotFound => {
            tracing::warn!("opencode binary NOT found on PATH");
            context.opencode_status = Some(OpencodeStatus::NotFound);
            Task::none()
        }
        
        Message::Project(inner_message) => {
            match inner_message {
                ProjectMessage::SearchChanged(q) => {
                    home_context.search_query = q;
                    Task::none()
                }
                
                ProjectMessage::ProjectsLoaded(list) => {
                    home_context.projects = list;
                    Task::none()
                }
                
                ProjectMessage::LoadFailed(e) => {
                    tracing::error!("Failed to load projects: {}", e);
                    Task::none()
                }
                
                ProjectMessage::NavigateToCreateProject => {
                    home_context.current_screen = crate::ui::app::state::CurrentScreen::CreateProject;
                    Task::none()
                }
                
                ProjectMessage::NavigateToHome => {
                    home_context.current_screen = crate::ui::app::state::CurrentScreen::Home;
                    Task::none()
                }
                
                ProjectMessage::CreateProjectNameChanged(name) => {
                    home_context.create_project_state.project_name = name;
                    home_context.create_project_state.error_message = None;
                    Task::none()
                }
                
                ProjectMessage::CreateProjectPathChanged(path) => {
                    home_context.create_project_state.file_path = path;
                    home_context.create_project_state.error_message = None;
                    Task::none()
                }
                
                ProjectMessage::CreateProjectSubmitted => {
                    // Validate required fields
                    if home_context.create_project_state.project_name.trim().is_empty() {
                        home_context.create_project_state.error_message = Some("Project name is required".to_string());
                        return Task::none();
                    }
                    
                    if home_context.create_project_state.file_path.trim().is_empty() {
                        home_context.create_project_state.error_message = Some("File path is required".to_string());
                        return Task::none();
                    }
                    
                    // Set submitting state
                    home_context.create_project_state.is_submitting = true;
                    home_context.create_project_state.error_message = None;
                    
                    // Extract context for async task
                    let core_db = context.core_db.clone().expect("CoreDB should be available");
                    let name = home_context.create_project_state.project_name.clone();
                    let path_str = home_context.create_project_state.file_path.clone();
                    
                    // Build async task for project creation
                    Task::perform(
                        async move {
                            use crate::modules::project::service::ProjectService;
                            use crate::modules::project::domain::CreateProjectCommand;
                            use std::path::PathBuf;
                            
                            let repo = crate::modules::project::SqliteProjectRepository::new(core_db);
                            let app_data_dir = dirs::data_dir()
                                .unwrap_or_else(|| std::path::PathBuf::from("."))
                                .join("melange");
                            let service = ProjectService::new(repo, app_data_dir);
                            
                            let command = CreateProjectCommand {
                                name: name.trim().to_string(),
                                description: None,
                                file_path: PathBuf::from(path_str.trim()),
                            };
                            
                            service.create_project(command).await
                        },
                        |result| match result {
                            Ok(project) => Message::Project(ProjectMessage::CreateProjectSucceeded(project)),
                            Err(e) => Message::Project(ProjectMessage::CreateProjectFailed(e.to_string())),
                        },
                    )
                }
                
                ProjectMessage::CreateProjectSucceeded(project) => {
                    // Add new project to list
                    home_context.projects.insert(0, project);
                    // Navigate back to home
                    home_context.current_screen = crate::ui::app::state::CurrentScreen::Home;
                    // Reset form state
                    home_context.create_project_state = crate::ui::app::state::CreateProjectState::default();
                    Task::none()
                }
                
                ProjectMessage::CreateProjectFailed(error) => {
                    home_context.create_project_state.is_submitting = false;
                    home_context.create_project_state.error_message = Some(error);
                    Task::none()
                }

                ProjectMessage::BrowseForFilePath => {
                    tracing::info!("Browse button clicked - opening file picker dialog");

                    // Spawn async task to open native folder picker
                    // Using spawn_blocking because rfd::FileDialog is synchronous and must run on main thread on macOS

                    let dialog_task = Task::perform(
                        async {
                            // File dialogs on macOS must run on the main thread
                            // We use spawn_blocking to avoid blocking the async runtime
                            let path_result = tokio::task::spawn_blocking(|| {
                                rfd::FileDialog::new()
                                    .set_title("Select Project Folder")
                                    .pick_folder()
                            })
                            .await
                            .ok()
                            .flatten();

                            path_result.map(|p| p.to_string_lossy().to_string())
                        },
                        |path_option| {
                            tracing::info!("File picker completed, path selected: {:?}", path_option.is_some());
                            Message::Project(ProjectMessage::FilePathSelected(path_option))
                        },
                    );

                    dialog_task
                }

                ProjectMessage::FilePathSelected(path_option) => {
                    tracing::info!("FilePathSelected message received: {:?}", path_option.is_some());

                    // If user selected a path (didn't cancel), update the form
                    if let Some(selected_path) = path_option {
                        tracing::info!("Updating file_path to: {}", selected_path);
                        home_context.create_project_state.file_path = selected_path;
                        home_context.create_project_state.error_message = None;
                    } else {
                        tracing::info!("No path selected (dialog cancelled)");
                    }
                    // If None (user cancelled), do nothing - leave form unchanged

                    Task::none()
                }
            }
        }
        
        Message::WindowResized(width) => {
            context.window_width = width;
            Task::none()
        }
    }
}