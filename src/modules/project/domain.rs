//! Pure domain types for the project bounded context.
//!
//! No I/O, no async, no database imports. All types here are plain Rust.

use std::fmt;
use uuid::Uuid;

// ─── ProjectId ───────────────────────────────────────────────────────────────

/// Newtype wrapper around a `Uuid` to uniquely identify a project.
///
/// Using a newtype prevents accidental mixing of `ProjectId` with other
/// UUID-typed identifiers at compile time.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProjectId(Uuid);

impl ProjectId {
    /// Generate a new random (v4) project identifier.
    pub fn new() -> ProjectId {
        ProjectId(Uuid::new_v4())
    }

    /// Borrow the inner `Uuid`.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl fmt::Display for ProjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for ProjectId {
    fn from(uuid: Uuid) -> Self {
        ProjectId(uuid)
    }
}

// ─── Project ─────────────────────────────────────────────────────────────────

/// The core project entity.
///
/// `created_at` is stored as an ISO 8601 UTC string (e.g. `"2026-03-17T00:00:00Z"`)
/// to match the SQLite `TEXT` column and avoid a `chrono`/`time` dependency.
#[derive(Debug, Clone)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub db_path: std::path::PathBuf,
    pub created_at: String,
}

// ─── CreateProjectCommand ─────────────────────────────────────────────────────

/// Value object carrying the inputs required to create a new project.
///
/// Validation rule (enforced in `ProjectService`): `name` must not be empty
/// or consist only of whitespace.
#[derive(Debug, Clone)]
pub struct CreateProjectCommand {
    pub name: String,
}

// ─── ProjectError ─────────────────────────────────────────────────────────────

/// Domain-level error enum for the project bounded context.
#[derive(Debug, Clone)]
pub enum ProjectError {
    /// `find_by_id` returned `None`.
    NotFound(ProjectId),

    /// `save` was called with a duplicate `id`.
    AlreadyExists(ProjectId),

    /// `name` was empty or contained only whitespace.
    InvalidName(String),

    /// Wraps a sqlx / anyhow error message.
    Database(String),

    /// Wraps a `std::io` error message.
    Io(String),
}

impl fmt::Display for ProjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProjectError::NotFound(id) => write!(f, "Project not found: {}", id),
            ProjectError::AlreadyExists(id) => write!(f, "Project already exists: {}", id),
            ProjectError::InvalidName(msg) => write!(f, "Invalid project name: {}", msg),
            ProjectError::Database(msg) => write!(f, "Database error: {}", msg),
            ProjectError::Io(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for ProjectError {}

impl From<sqlx::Error> for ProjectError {
    fn from(err: sqlx::Error) -> Self {
        ProjectError::Database(err.to_string())
    }
}

impl From<anyhow::Error> for ProjectError {
    fn from(err: anyhow::Error) -> Self {
        ProjectError::Database(err.to_string())
    }
}
