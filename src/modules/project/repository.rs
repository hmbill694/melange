//! Repository abstraction and SQLite implementation for the project domain.

#![allow(dead_code)]

use async_trait::async_trait;
use sqlx::Row;
use std::path::PathBuf;
use uuid::Uuid;

use crate::db::CoreDb;
use crate::modules::project::domain::{Project, ProjectError, ProjectId};

// ─── ProjectRepository Trait ──────────────────────────────────────────────────

/// Async repository trait for persisting and retrieving `Project` entities.
///
/// `Send + Sync` bounds are required because iced runs on a multi-threaded
/// tokio runtime.
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    /// Persist a new project. Returns `ProjectError::AlreadyExists` if a
    /// project with the same `id` already exists.
    async fn save(&self, project: &Project) -> Result<(), ProjectError>;

    /// Retrieve a project by its identifier. Returns `Ok(None)` if not found.
    async fn find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, ProjectError>;

    /// Retrieve all projects ordered by `created_at` ascending.
    async fn find_all(&self) -> Result<Vec<Project>, ProjectError>;
}

// ─── SqliteProjectRepository ─────────────────────────────────────────────────

/// SQLite-backed implementation of `ProjectRepository`.
///
/// Holds a (cheap Arc) clone of `CoreDb` so the pool can be shared without
/// lifetime constraints.
#[derive(Clone)]
pub struct SqliteProjectRepository {
    db: CoreDb,
}

impl SqliteProjectRepository {
    /// Create a new repository using the given `CoreDb` handle.
    pub fn new(db: CoreDb) -> Self {
        SqliteProjectRepository { db }
    }
}

#[async_trait]
impl ProjectRepository for SqliteProjectRepository {
    async fn save(&self, project: &Project) -> Result<(), ProjectError> {
        let pool = self.db.pool();
        let id_str = project.id.to_string();
        let db_path_str = project.db_path.to_string_lossy().to_string();

        let file_path_str = project.file_path.to_string_lossy().to_string();

        let result = sqlx::query(
            "INSERT OR FAIL INTO projects (id, name, db_path, created_at, description, file_path) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&id_str)
        .bind(&project.name)
        .bind(&db_path_str)
        .bind(&project.created_at)
        .bind(&project.description)
        .bind(&file_path_str)
        .execute(pool)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(sqlx::Error::Database(db_err)) => {
                // Detect UNIQUE constraint violation for the primary key.
                if db_err.is_unique_violation()
                    || db_err.message().contains("UNIQUE constraint failed")
                {
                    Err(ProjectError::AlreadyExists(project.id.clone()))
                } else {
                    Err(ProjectError::Database(db_err.to_string()))
                }
            }
            Err(e) => Err(ProjectError::Database(e.to_string())),
        }
    }

    async fn find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, ProjectError> {
        let pool = self.db.pool();
        let id_str = id.to_string();

        let row = sqlx::query(
            "SELECT id, name, db_path, created_at, description, file_path FROM projects WHERE id = ?",
        )
        .bind(&id_str)
        .fetch_optional(pool)
        .await
        .map_err(|e| ProjectError::Database(e.to_string()))?;

        match row {
            None => Ok(None),
            Some(r) => {
                let id_text: String = r.get("id");
                let name: String = r.get("name");
                let db_path_text: String = r.get("db_path");
                let created_at: String = r.get("created_at");
                let description: Option<String> = r.get("description");
                let file_path_text: String = r.get("file_path");

                let uuid = Uuid::parse_str(&id_text)
                    .map_err(|e| ProjectError::Database(e.to_string()))?;

                Ok(Some(Project {
                    id: ProjectId::from(uuid),
                    name,
                    db_path: PathBuf::from(db_path_text),
                    created_at,
                    description,
                    file_path: PathBuf::from(file_path_text),
                }))
            }
        }
    }

    async fn find_all(&self) -> Result<Vec<Project>, ProjectError> {
        let pool = self.db.pool();

        let rows = sqlx::query(
            "SELECT id, name, db_path, created_at, description, file_path FROM projects ORDER BY created_at ASC",
        )
        .fetch_all(pool)
        .await
        .map_err(|e| ProjectError::Database(e.to_string()))?;

        let mut projects = Vec::with_capacity(rows.len());
        for r in rows {
            let id_text: String = r.get("id");
            let name: String = r.get("name");
            let db_path_text: String = r.get("db_path");
            let created_at: String = r.get("created_at");
            let description: Option<String> = r.get("description");
            let file_path_text: String = r.get("file_path");

            let uuid = Uuid::parse_str(&id_text)
                .map_err(|e| ProjectError::Database(e.to_string()))?;

            projects.push(Project {
                id: ProjectId::from(uuid),
                name,
                db_path: PathBuf::from(db_path_text),
                created_at,
                description,
                file_path: PathBuf::from(file_path_text),
            });
        }

        Ok(projects)
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    async fn make_test_repo() -> SqliteProjectRepository {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("migrations/core").run(&pool).await.unwrap();
        let db = CoreDb::from_pool(pool);
        SqliteProjectRepository::new(db)
    }

    #[tokio::test]
    async fn test_save_and_find_by_id_returns_project() {
        let repo = make_test_repo().await;
        let project = Project {
            id: ProjectId::new(),
            name: "Alpha".to_string(),
            db_path: PathBuf::from("/tmp/alpha.db"),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            description: None,
            file_path: PathBuf::from("/tmp/alpha"),
        };

        let save_result = repo.save(&project).await;
        assert!(save_result.is_ok(), "save should succeed: {:?}", save_result);

        let find_result = repo.find_by_id(&project.id).await;
        assert!(find_result.is_ok());
        let found = find_result.unwrap();
        assert!(found.is_some(), "project should be found");
        let p = found.unwrap();
        assert_eq!(p.name, "Alpha");
        assert_eq!(p.db_path, PathBuf::from("/tmp/alpha.db"));
    }

    #[tokio::test]
    async fn test_find_by_id_returns_none_for_unknown_id() {
        let repo = make_test_repo().await;
        let unknown_id = ProjectId::new();
        let result = repo.find_by_id(&unknown_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none(), "should return None for unknown id");
    }

    #[tokio::test]
    async fn test_find_all_returns_all_saved_projects() {
        let repo = make_test_repo().await;

        let project_a = Project {
            id: ProjectId::new(),
            name: "A".to_string(),
            db_path: PathBuf::from("/tmp/a.db"),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            description: None,
            file_path: PathBuf::from("/tmp/a"),
        };
        let project_b = Project {
            id: ProjectId::new(),
            name: "B".to_string(),
            db_path: PathBuf::from("/tmp/b.db"),
            created_at: "2026-01-02T00:00:00Z".to_string(),
            description: None,
            file_path: PathBuf::from("/tmp/b"),
        };

        repo.save(&project_a).await.unwrap();
        repo.save(&project_b).await.unwrap();

        let all = repo.find_all().await;
        assert!(all.is_ok());
        let all = all.unwrap();
        assert_eq!(all.len(), 2, "should return both projects");

        let names: Vec<&str> = all.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"A"), "names should contain A");
        assert!(names.contains(&"B"), "names should contain B");
    }

    #[tokio::test]
    async fn test_save_duplicate_id_returns_already_exists() {
        let repo = make_test_repo().await;
        let project = Project {
            id: ProjectId::new(),
            name: "Dup".to_string(),
            db_path: PathBuf::from("/tmp/dup.db"),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            description: None,
            file_path: PathBuf::from("/tmp/dup"),
        };

        repo.save(&project).await.unwrap();
        let second = repo.save(&project).await;

        assert!(
            matches!(second, Err(ProjectError::AlreadyExists(_))),
            "second save should return AlreadyExists, got: {:?}",
            second
        );
    }
}
