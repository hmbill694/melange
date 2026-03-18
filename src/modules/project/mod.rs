//! Project bounded context.
//!
//! Contains the full DDD stack for project management:
//! - `domain`     — pure domain types (entities, value objects, errors)
//! - `repository` — persistence abstraction and SQLite implementation
//! - `service`    — application service with business logic

pub mod domain;
pub mod repository;
pub mod service;

#[allow(unused_imports)]
pub use domain::{CreateProjectCommand, Project, ProjectError, ProjectId};
#[allow(unused_imports)]
pub use repository::{ProjectRepository, SqliteProjectRepository};
#[allow(unused_imports)]
pub use service::ProjectService;
