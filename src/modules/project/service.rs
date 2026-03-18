//! Business logic for the project bounded context.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::db::ProjectDb;
use crate::modules::project::domain::{
    CreateProjectCommand, Project, ProjectError, ProjectId,
};
use crate::modules::project::repository::ProjectRepository;

// ─── Timestamp helper ─────────────────────────────────────────────────────────

/// Format epoch seconds as an ISO 8601 UTC timestamp string: `"YYYY-MM-DDTHH:MM:SSZ"`.
///
/// Uses only integer arithmetic — no `chrono` or `time` dependency required.
fn format_iso8601(epoch_secs: u64) -> String {
    // Days in each month for a non-leap year and leap year.
    const DAYS_IN_MONTH: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    let secs_per_day = 86_400u64;
    let mut days = epoch_secs / secs_per_day;
    let time_of_day = epoch_secs % secs_per_day;

    let hour = time_of_day / 3600;
    let minute = (time_of_day % 3600) / 60;
    let second = time_of_day % 60;

    // Calculate year from days since Unix epoch (1970-01-01).
    let mut year = 1970u32;
    loop {
        let leap = is_leap_year(year);
        let days_in_year = if leap { 366u64 } else { 365u64 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }

    // Calculate month and day within year.
    let leap = is_leap_year(year);
    let mut month = 0usize;
    for m in 0..12 {
        let dim = if m == 1 && leap {
            29u64
        } else {
            DAYS_IN_MONTH[m] as u64
        };
        if days < dim {
            month = m;
            break;
        }
        days -= dim;
    }
    let day = days + 1; // 1-indexed

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year,
        month + 1,
        day,
        hour,
        minute,
        second
    )
}

fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Generate the current UTC timestamp as an ISO 8601 string.
fn now_iso8601() -> String {
    let epoch_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format_iso8601(epoch_secs)
}

// ─── ProjectService ───────────────────────────────────────────────────────────

/// Application service for project management.
///
/// Generic over the repository implementation so it can be tested with a mock
/// without touching the filesystem or a real database.
pub struct ProjectService<R: ProjectRepository> {
    repository: R,
    app_data_dir: PathBuf,
}

impl<R: ProjectRepository> ProjectService<R> {
    /// Create a new `ProjectService` with the given repository and app-data directory.
    pub fn new(repository: R, app_data_dir: PathBuf) -> Self {
        ProjectService {
            repository,
            app_data_dir,
        }
    }

    /// Create a new project from the given command.
    ///
    /// 1. Validates the name is non-empty.
    /// 2. Generates a new `ProjectId`.
    /// 3. Creates the on-disk project database.
    /// 4. Saves the project to the core repository.
    /// 5. Returns the newly created `Project`.
    pub async fn create_project(
        &self,
        cmd: CreateProjectCommand,
    ) -> Result<Project, ProjectError> {
        // 1. Validate name.
        if cmd.name.trim().is_empty() {
            return Err(ProjectError::InvalidName("name cannot be empty".to_string()));
        }

        // 2. Generate id.
        let id = ProjectId::new();

        // 3. Derive db_path.
        let db_path = self
            .app_data_dir
            .join("projects")
            .join(format!("{}.db", id));

        // 4. Create the project-scoped database file.
        ProjectDb::create(&db_path)
            .await
            .map_err(|e| ProjectError::Io(e.to_string()))?;

        // 5. Build timestamp and project entity.
        let created_at = now_iso8601();
        let project = Project {
            id,
            name: cmd.name.trim().to_string(),
            db_path,
            created_at,
        };

        // 6. Persist via repository.
        self.repository.save(&project).await?;

        Ok(project)
    }

    /// List all projects.
    pub async fn list_projects(&self) -> Result<Vec<Project>, ProjectError> {
        self.repository.find_all().await
    }

    /// Open an existing project's database by id.
    ///
    /// Returns `ProjectError::NotFound` if no project with the given id exists.
    pub async fn open_project(&self, id: &ProjectId) -> Result<ProjectDb, ProjectError> {
        let project = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ProjectError::NotFound(id.clone()))?;

        let db = ProjectDb::open(&project.db_path)
            .await
            .map_err(|e| ProjectError::Io(e.to_string()))?;

        Ok(db)
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // ── Mock repository ──────────────────────────────────────────────────────

    struct MockProjectRepository {
        projects: Mutex<HashMap<String, Project>>,
    }

    impl MockProjectRepository {
        fn new() -> Self {
            MockProjectRepository {
                projects: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl ProjectRepository for MockProjectRepository {
        async fn save(&self, project: &Project) -> Result<(), ProjectError> {
            let mut map = self.projects.lock().unwrap();
            let key = project.id.to_string();
            if map.contains_key(&key) {
                return Err(ProjectError::AlreadyExists(project.id.clone()));
            }
            map.insert(key, project.clone());
            Ok(())
        }

        async fn find_by_id(
            &self,
            id: &ProjectId,
        ) -> Result<Option<Project>, ProjectError> {
            let map = self.projects.lock().unwrap();
            Ok(map.get(&id.to_string()).cloned())
        }

        async fn find_all(&self) -> Result<Vec<Project>, ProjectError> {
            let map = self.projects.lock().unwrap();
            Ok(map.values().cloned().collect())
        }
    }

    // ── Test cases ───────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_create_project_returns_project_with_given_name() {
        let repo = MockProjectRepository::new();
        let service = ProjectService::new(repo, std::env::temp_dir());

        let result = service
            .create_project(CreateProjectCommand {
                name: "My Project".to_string(),
            })
            .await;

        assert!(result.is_ok(), "create_project should succeed: {:?}", result);
        let p = result.unwrap();
        assert_eq!(p.name, "My Project");
        // id should be a valid UUID — verifiable via to_string length
        assert_eq!(p.id.to_string().len(), 36, "id should be a UUID string");
        // db_path should end with ".db"
        assert!(
            p.db_path.to_string_lossy().ends_with(".db"),
            "db_path should end with .db"
        );
    }

    #[tokio::test]
    async fn test_create_project_with_blank_name_returns_invalid_name() {
        let repo = MockProjectRepository::new();
        let service = ProjectService::new(repo, std::env::temp_dir());

        let result = service
            .create_project(CreateProjectCommand {
                name: "   ".to_string(),
            })
            .await;

        assert!(
            matches!(result, Err(ProjectError::InvalidName(_))),
            "blank name should return InvalidName, got: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_create_project_calls_repository_save() {
        let repo = MockProjectRepository::new();
        let service = ProjectService::new(repo, std::env::temp_dir());

        service
            .create_project(CreateProjectCommand {
                name: "X".to_string(),
            })
            .await
            .unwrap();

        let all = service.list_projects().await.unwrap();
        assert_eq!(all.len(), 1, "should have one project");
        assert_eq!(all[0].name, "X");
    }

    #[tokio::test]
    async fn test_list_projects_delegates_to_repository() {
        let repo = MockProjectRepository::new();

        // Pre-populate with 2 projects directly.
        {
            let mut map = repo.projects.lock().unwrap();
            let p1 = Project {
                id: ProjectId::new(),
                name: "First".to_string(),
                db_path: PathBuf::from("/tmp/first.db"),
                created_at: "2026-01-01T00:00:00Z".to_string(),
            };
            let p2 = Project {
                id: ProjectId::new(),
                name: "Second".to_string(),
                db_path: PathBuf::from("/tmp/second.db"),
                created_at: "2026-01-02T00:00:00Z".to_string(),
            };
            map.insert(p1.id.to_string(), p1);
            map.insert(p2.id.to_string(), p2);
        }

        let service = ProjectService::new(repo, std::env::temp_dir());
        let all = service.list_projects().await.unwrap();
        assert_eq!(all.len(), 2, "should return both pre-populated projects");
    }
}
