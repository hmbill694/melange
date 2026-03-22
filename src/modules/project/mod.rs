//! Project bounded context.
//!
//! Contains the full DDD stack for project management:
//! - `domain`     — pure domain types (entities, value objects, errors)
//! - `repository` — persistence abstraction and SQLite implementation
//! - `service`    — application service with business logic
//! - `message`    — iced message types for the project UI
//!
//! Note: UI components have been moved to src/ui/project.rs

pub mod domain;
pub mod message;
pub mod repository;
pub mod service;

#[allow(unused_imports)]
pub use domain::{CreateProjectCommand, Project, ProjectError, ProjectId};
#[allow(unused_imports)]
pub use message::ProjectMessage;
#[allow(unused_imports)]
pub use repository::{ProjectRepository, SqliteProjectRepository};
#[allow(unused_imports)]
pub use service::ProjectService;
