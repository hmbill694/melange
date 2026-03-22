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
            }
        }
        
        Message::WindowResized(width) => {
            context.window_width = width;
            Task::none()
        }
    }
}