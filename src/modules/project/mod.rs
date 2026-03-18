//! Project bounded context.
//!
//! Contains the full DDD stack for project management:
//! - `domain`     — pure domain types (entities, value objects, errors)
//! - `repository` — persistence abstraction and SQLite implementation
//! - `service`    — application service with business logic

pub mod domain;
pub mod repository;
pub mod service;

pub use domain::{CreateProjectCommand, Project, ProjectError, ProjectId};
pub use repository::{ProjectRepository, SqliteProjectRepository};
pub use service::ProjectService;
