# Implementation Plan: Database Foundation

**Requirements source:** `agent-docs/plans/database_foundation_1773808818_requirements.md`  
**Target output file:** this document  

---

## Overview

This plan establishes a dual-database SQLite layer (CoreDb + ProjectDb), a DDD project module with domain/repository/service layers, and updates `app.rs` to use the new typed DB wrappers. All work is broken into discrete, ordered phases. Each phase must be completed before the next begins.

---

## Phase 1 — Cargo.toml: Add Dependencies

- [ ] **1.1** Open `Cargo.toml` and append three new entries under `[dependencies]`:
  - `uuid = { version = "1", features = ["v4"] }` — UUID generation with the v4 (random) algorithm
  - `async-trait = "0.1"` — makes async methods work inside traits (object-safe, mockable)
  - `dirs = "5"` — cross-platform resolution of the OS app-data directory

  **Why:** These are the only missing crates. All other required crates (`sqlx`, `anyhow`, `tokio`, `tracing`) are already present in `Cargo.toml`.

---

## Phase 2 — Migration File Reorganisation

- [ ] **2.1** Create directory `migrations/core/` (if it does not exist).

- [ ] **2.2** Move (rename/copy then delete) `migrations/20260317000000_init.sql` → `migrations/core/20260317000000_init.sql`.  
  - The file content stays identical: a no-op comment-only migration.  
  - **Why:** sqlx `migrate!("migrations/core")` requires the migration files to live in the `migrations/core/` subdirectory. The file must retain its original timestamp prefix so the sqlx migration checksum history is unaffected on any DB that has already applied it.

- [ ] **2.3** Create `migrations/core/20260317000001_create_projects.sql` with the following DDL:
  ```
  CREATE TABLE IF NOT EXISTS projects (
      id         TEXT NOT NULL PRIMARY KEY,   -- UUID stored as text
      name       TEXT NOT NULL,
      db_path    TEXT NOT NULL,               -- absolute path to the project's .db file
      created_at TEXT NOT NULL                -- ISO 8601 timestamp
  );
  ```
  **Why:** This is the canonical projects table for the core database. `IF NOT EXISTS` makes it idempotent.

- [ ] **2.4** Create directory `migrations/project/` (if it does not exist).

- [ ] **2.5** Create `migrations/project/20260317000000_init.sql` as a no-op placeholder:
  ```sql
  -- Initial project migration — scaffold only.
  -- Project-scoped tables will be added in subsequent migrations.
  ```
  **Why:** sqlx `migrate!("migrations/project")` requires at least one file to be present, and the project DB will have its own migration tracking table (`_sqlx_migrations`) separate from the core DB.

---

## Phase 3 — `src/db/mod.rs` Rewrite

**Goal:** Replace the single `create_pool` free function with two typed wrapper structs: `CoreDb` and `ProjectDb`.

- [ ] **3.1** Remove the existing `create_pool` function entirely.

- [ ] **3.2** Define `CoreDb` struct:
  ```
  struct CoreDb:
    field: pool — SqlitePool (private)
  
  derive: Clone
  ```
  `Clone` is cheap because `SqlitePool` internally uses an `Arc`.

- [ ] **3.3** Implement `CoreDb::open(app_data_dir: &Path) -> Result<CoreDb>` (async):
  ```
  function CoreDb::open(app_data_dir):
    1. Construct db_path = app_data_dir.join("melange.db")
    2. Ensure parent directory exists:
         std::fs::create_dir_all(app_data_dir) — propagate error with `?`
    3. Build connection string:
         url = format "sqlite://{}", db_path.display()
    4. Open pool:
         pool = SqlitePool::connect(&url).await?
    5. Run embedded core migrations:
         sqlx::migrate!("migrations/core").run(&pool).await?
    6. Log "Core database ready at <db_path>"
    7. Return Ok(CoreDb { pool })
  ```
  **Why:** The `app_data_dir` argument is passed in (resolved externally in `app.rs`) so the function remains testable. `create_dir_all` is idempotent.

- [ ] **3.4** Add an accessor method `CoreDb::pool(&self) -> &SqlitePool` (public) so repositories can borrow the pool without exposing the field directly.

- [ ] **3.5** Define `ProjectDb` struct:
  ```
  struct ProjectDb:
    field: pool — SqlitePool (private)
  
  derive: Clone
  ```

- [ ] **3.6** Implement `ProjectDb::create(db_path: &Path) -> Result<ProjectDb>` (async):
  ```
  function ProjectDb::create(db_path):
    1. Ensure parent directory exists:
         std::fs::create_dir_all(db_path.parent().unwrap_or(Path::new(".")))
    2. Build url = format "sqlite://{}", db_path.display()
    3. Open pool = SqlitePool::connect(&url).await?
    4. Run embedded project migrations:
         sqlx::migrate!("migrations/project").run(&pool).await?
    5. Log "Project database created at <db_path>"
    6. Return Ok(ProjectDb { pool })
  ```

- [ ] **3.7** Implement `ProjectDb::open(db_path: &Path) -> Result<ProjectDb>` (async):
  ```
  function ProjectDb::open(db_path):
    1. Verify the file exists:
         if !db_path.exists() → return Err(anyhow!("Project DB not found: {db_path}"))
    2. Build url = format "sqlite://{}", db_path.display()
    3. Open pool = SqlitePool::connect(&url).await?
    4. Run embedded project migrations (idempotent):
         sqlx::migrate!("migrations/project").run(&pool).await?
    5. Log "Project database opened at <db_path>"
    6. Return Ok(ProjectDb { pool })
  ```

- [ ] **3.8** Add `ProjectDb::pool(&self) -> &SqlitePool` accessor (public).

- [ ] **3.9** Update the module-level doc comment to reflect the new structure and remove any references to the old `create_pool` function.

  **Note on `sqlx::migrate!` path argument:** The string literal passed to `migrate!` is resolved relative to the crate root (the directory containing `Cargo.toml`) at compile time. So `migrate!("migrations/core")` and `migrate!("migrations/project")` will correctly find `migrations/core/` and `migrations/project/` respectively.

- [ ] **3.10** Add a `pub(crate) fn from_pool(pool: SqlitePool) -> CoreDb` constructor on `CoreDb` so that integration tests in other files can construct a `CoreDb` from a raw in-memory pool without going through the file-system path.

---

## Phase 4 — `src/modules/project/domain.rs` (New File)

**Goal:** Define all pure domain types for the project bounded context. No I/O, no async, no DB imports.

- [ ] **4.1** Create file `src/modules/project/domain.rs`.

- [ ] **4.2** Define `ProjectId` newtype:
  ```
  struct ProjectId(uuid::Uuid)
  derive: Debug, Clone, PartialEq, Eq, Hash
  
  impl ProjectId:
    function new() -> ProjectId:
      return ProjectId(Uuid::new_v4())
    function as_uuid(&self) -> &Uuid:
      return &self.0
    function to_string(&self) -> String:
      return self.0.to_string()
  ```
  **Why newtype:** Prevents mixing up ProjectId with other UUID-typed IDs at compile time.

- [ ] **4.3** Define `Project` entity:
  ```
  struct Project:
    id: ProjectId
    name: String
    db_path: std::path::PathBuf
    created_at: String    -- ISO 8601 UTC timestamp string, e.g. "2026-03-17T00:00:00Z"
  
  derive: Debug, Clone
  ```
  `created_at` is stored and transported as a `String` to match the SQLite TEXT column, avoiding an extra chrono/time dependency for now.

- [ ] **4.4** Define `CreateProjectCommand` value object:
  ```
  struct CreateProjectCommand:
    name: String
  
  derive: Debug, Clone
  ```
  **Validation rule (enforced in service):** `name` must not be empty or blank.

- [ ] **4.5** Define `ProjectError` enum — implement `std::fmt::Display` and `std::error::Error` manually (do NOT add `thiserror` unless it is already in `Cargo.toml`):
  ```
  enum ProjectError:
    NotFound(ProjectId)            -- find_by_id returned None
    AlreadyExists(ProjectId)       -- save called with a duplicate id
    InvalidName(String)            -- empty or invalid project name
    Database(String)               -- wraps sqlx/anyhow error message
    Io(String)                     -- wraps std::io errors
  
  derive: Debug, Clone
  
  impl Display for ProjectError:
    match self:
      NotFound(id)     → write "Project not found: {id}"
      AlreadyExists(id)→ write "Project already exists: {id}"
      InvalidName(msg) → write "Invalid project name: {msg}"
      Database(msg)    → write "Database error: {msg}"
      Io(msg)          → write "IO error: {msg}"
  
  impl std::error::Error for ProjectError
  
  impl From<sqlx::Error> for ProjectError:
    convert → ProjectError::Database(err.to_string())
  
  impl From<anyhow::Error> for ProjectError:
    convert → ProjectError::Database(err.to_string())
  ```

---

## Phase 5 — `src/modules/project/repository.rs` (New File)

**Goal:** Define the repository abstraction and its SQLite implementation backed by `CoreDb`.

- [ ] **5.1** Create file `src/modules/project/repository.rs`.

- [ ] **5.2** Define the `ProjectRepository` async trait:
  ```
  #[async_trait]
  trait ProjectRepository: Send + Sync:
    async fn save(&self, project: &Project) -> Result<(), ProjectError>
    async fn find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, ProjectError>
    async fn find_all(&self) -> Result<Vec<Project>, ProjectError>
  ```
  `Send + Sync` bounds are required because iced runs on a multi-threaded tokio runtime.

- [ ] **5.3** Define `SqliteProjectRepository` struct:
  ```
  struct SqliteProjectRepository:
    db: CoreDb   -- holds a clone of CoreDb (cheap Arc clone)
  
  derive: Clone
  
  impl SqliteProjectRepository:
    function new(db: CoreDb) -> Self:
      return SqliteProjectRepository { db }
  ```

- [ ] **5.4** Implement `ProjectRepository` for `SqliteProjectRepository`:

  **`save` method pseudocode:**
  ```
  async fn save(&self, project: &Project):
    pool = self.db.pool()
    id_str = project.id.to_string()
    db_path_str = project.db_path.to_string_lossy().to_string()
    
    execute: INSERT OR FAIL INTO projects (id, name, db_path, created_at)
             VALUES (?, ?, ?, ?)
             bind: id_str, project.name, db_path_str, project.created_at
    
    if sqlx returns a UNIQUE constraint violation (SqliteError code 2067 or "UNIQUE constraint"):
      return Err(ProjectError::AlreadyExists(project.id.clone()))
    else propagate other errors as ProjectError::Database
    return Ok(())
  ```
  Use `sqlx::query("INSERT OR FAIL INTO projects ...")`.bind(...).execute(pool).await` (dynamic query API, not macro form — avoids need for offline cache update).

  **`find_by_id` method pseudocode:**
  ```
  async fn find_by_id(&self, id: &ProjectId):
    pool = self.db.pool()
    id_str = id.to_string()
    
    row = SELECT id, name, db_path, created_at FROM projects WHERE id = ?
          bind: id_str
          .fetch_optional(pool).await → map sqlx error to ProjectError::Database
    
    if row is None → return Ok(None)
    
    map row columns to Project:
      id: ProjectId(Uuid::parse_str(&row.get::<String, _>("id")))
      name: row.get::<String, _>("name")
      db_path: PathBuf::from(row.get::<String, _>("db_path"))
      created_at: row.get::<String, _>("created_at")
    
    return Ok(Some(project))
  ```

  **`find_all` method pseudocode:**
  ```
  async fn find_all(&self):
    pool = self.db.pool()
    
    rows = SELECT id, name, db_path, created_at FROM projects ORDER BY created_at ASC
           .fetch_all(pool).await → map error to ProjectError::Database
    
    map each row to Project (same mapping logic as find_by_id)
    return Ok(vec of projects)
  ```

- [ ] **5.5** Write integration tests in a `#[cfg(test)]` module at the bottom of `repository.rs`:

  **Test setup helper pseudocode:**
  ```
  async fn make_test_repo() -> SqliteProjectRepository:
    pool = SqlitePool::connect("sqlite::memory:").await.unwrap()
    sqlx::migrate!("migrations/core").run(&pool).await.unwrap()
    db = CoreDb::from_pool(pool)   -- uses the pub(crate) constructor from Phase 3.10
    return SqliteProjectRepository::new(db)
  ```

  **Test cases (each annotated `#[tokio::test]`):**
  ```
  test "save and find_by_id returns the project":
    repo = make_test_repo().await
    project = Project { id: ProjectId::new(), name: "Alpha", db_path: "/tmp/alpha.db", created_at: "2026-01-01T00:00:00Z" }
    repo.save(&project).await → assert Ok(())
    result = repo.find_by_id(&project.id).await → assert Ok(Some(p))
    assert p.name == "Alpha"
    assert p.db_path == PathBuf::from("/tmp/alpha.db")
  
  test "find_by_id returns None for unknown id":
    repo = make_test_repo().await
    unknown_id = ProjectId::new()
    result = repo.find_by_id(&unknown_id).await → assert Ok(None)
  
  test "find_all returns all saved projects":
    repo = make_test_repo().await
    save project A, save project B
    all = repo.find_all().await → assert Ok(vec of len 2)
    assert names contain "A" and "B"
  
  test "save duplicate id returns AlreadyExists error":
    repo = make_test_repo().await
    project = Project { id: ProjectId::new(), ... }
    repo.save(&project).await → Ok(())
    second = repo.save(&project).await
    assert second is Err(ProjectError::AlreadyExists(_))
  ```
  Each test is fully self-contained (in-memory DB, no shared state, no files on disk).

---

## Phase 6 — `src/modules/project/service.rs` (New File)

**Goal:** Implement business logic in a `ProjectService` generic over the repository trait.

- [ ] **6.1** Create file `src/modules/project/service.rs`.

- [ ] **6.2** Define `ProjectService<R>`:
  ```
  struct ProjectService<R: ProjectRepository>:
    repository: R
    app_data_dir: PathBuf   -- needed to derive project db_path
  
  impl<R: ProjectRepository> ProjectService<R>:
    function new(repository: R, app_data_dir: PathBuf) -> Self:
      return ProjectService { repository, app_data_dir }
  ```

- [ ] **6.3** Implement `create_project` method:
  ```
  async fn create_project(&self, cmd: CreateProjectCommand) -> Result<Project, ProjectError>:
    
    1. Validate: if cmd.name.trim().is_empty() → Err(ProjectError::InvalidName("name cannot be empty"))
    
    2. id = ProjectId::new()
    
    3. db_path = self.app_data_dir
                     .join("projects")
                     .join(format!("{}.db", id.to_string()))
    
    4. Create project database:
         ProjectDb::create(&db_path).await
           map Err → ProjectError::Io(err.to_string())
         (Drop the returned handle — callers use open_project to get an active handle later)
    
    5. Build created_at timestamp:
         use std::time::SystemTime and UNIX_EPOCH to compute seconds since epoch
         format as "YYYY-MM-DDTHH:MM:SSZ" — implement a minimal formatter using integer arithmetic
         (no chrono dependency required)
    
    6. project = Project { id, name: cmd.name.trim().to_string(), db_path, created_at }
    
    7. self.repository.save(&project).await?
    
    8. return Ok(project)
  ```

- [ ] **6.4** Implement `list_projects` method:
  ```
  async fn list_projects(&self) -> Result<Vec<Project>, ProjectError>:
    return self.repository.find_all().await
  ```

- [ ] **6.5** Implement `open_project` method:
  ```
  async fn open_project(&self, id: &ProjectId) -> Result<ProjectDb, ProjectError>:
    
    1. project = self.repository.find_by_id(id).await?
       if None → return Err(ProjectError::NotFound(id.clone()))
    
    2. db = ProjectDb::open(&project.db_path).await
         map Err → ProjectError::Io(err.to_string())
    
    3. return Ok(db)
  ```

- [ ] **6.6** Write unit tests in a `#[cfg(test)]` module at the bottom of `service.rs`:

  **Mock repository pseudocode:**
  ```
  struct MockProjectRepository:
    projects: std::sync::Mutex<HashMap<String, Project>>
  
  impl MockProjectRepository:
    function new() -> Self:
      MockProjectRepository { projects: Mutex::new(HashMap::new()) }
  
  #[async_trait]
  impl ProjectRepository for MockProjectRepository:
    async fn save(&self, project):
      mut map = self.projects.lock().unwrap()
      key = project.id.to_string()
      if map.contains_key(&key) → return Err(AlreadyExists(project.id.clone()))
      map.insert(key, project.clone())
      Ok(())
    
    async fn find_by_id(&self, id):
      map = self.projects.lock().unwrap()
      Ok(map.get(&id.to_string()).cloned())
    
    async fn find_all(&self):
      map = self.projects.lock().unwrap()
      Ok(map.values().cloned().collect())
  ```
  `std::sync::Mutex` is `Send + Sync` — satisfies the `ProjectRepository` trait bound.

  **Test cases (each `#[tokio::test]`):**
  ```
  test "create_project returns project with given name":
    repo = MockProjectRepository::new()
    service = ProjectService::new(repo, temp_dir())
    result = service.create_project(CreateProjectCommand { name: "My Project" }).await
    assert Ok(p) where p.name == "My Project" and p.id is valid UUID and p.db_path ends with ".db"
    (Note: this test creates a real .db file in temp_dir — acceptable per requirements)
  
  test "create_project with blank name returns InvalidName":
    repo = MockProjectRepository::new()
    service = ProjectService::new(repo, temp_dir())
    result = service.create_project(CreateProjectCommand { name: "   " }).await
    assert Err(ProjectError::InvalidName(_))
  
  test "create_project calls repository save (project appears in list)":
    repo = MockProjectRepository::new()
    service = ProjectService::new(repo, temp_dir())
    service.create_project(CreateProjectCommand { name: "X" }).await.unwrap()
    all = service.list_projects().await.unwrap()
    assert all.len() == 1 and all[0].name == "X"
  
  test "list_projects delegates to repository":
    repo = MockProjectRepository::new()
    -- pre-populate: insert 2 Project values directly into repo.projects
    service = ProjectService::new(repo, temp_dir())
    all = service.list_projects().await.unwrap()
    assert all.len() == 2
  ```
  Use `std::env::temp_dir()` for the `app_data_dir` in service tests to ensure real `ProjectDb::create` calls have a writable path.

---

## Phase 7 — `src/modules/project/mod.rs` (New File)

- [ ] **7.1** Create file `src/modules/project/mod.rs`:
  ```
  declare: pub mod domain
  declare: pub mod repository
  declare: pub mod service
  
  pub use domain::{Project, ProjectId, CreateProjectCommand, ProjectError}
  pub use repository::{ProjectRepository, SqliteProjectRepository}
  pub use service::ProjectService
  ```

---

## Phase 8 — `src/modules/mod.rs` Update

- [ ] **8.1** Add `pub mod project;` to `src/modules/mod.rs`, replacing the example comment:
  ```
  [existing doc comment block unchanged]
  
  pub mod project;
  ```

---

## Phase 9 — `src/app.rs` Update

**Goal:** Replace `db: Option<SqlitePool>` with `core_db: Option<CoreDb>`, update boot to resolve the platform app-data dir and call `CoreDb::open`, update all downstream references.

- [ ] **9.1** Update imports:
  ```
  add: use crate::db::CoreDb;
  add: use std::path::PathBuf;   (if not already imported)
  remove: use sqlx::SqlitePool;  (no longer referenced directly)
  ```

- [ ] **9.2** Update `App` struct:
  ```
  struct App:
    core_db: Option<CoreDb>     -- replaces db: Option<SqlitePool>
    init_error: Option<String>  -- unchanged
  ```

- [ ] **9.3** Update `App::default()`:
  ```
  Self { core_db: None, init_error: None }
  ```

- [ ] **9.4** Update `Message` enum:
  ```
  DbReady(CoreDb)    -- was DbReady(SqlitePool)
  DbFailed(String)   -- unchanged
  ```
  `CoreDb` derives `Clone` (Phase 3.2), satisfying iced's `#[derive(Clone)]` on `Message`.

- [ ] **9.5** Update `new()`: change the `Ok(pool) → Message::DbReady(pool)` arm to `Ok(core_db) → Message::DbReady(core_db)`.

- [ ] **9.6** Rewrite `init_db()` async helper:
  ```
  async fn init_db() -> Result<CoreDb>:
    base = dirs::data_dir()
      if None → return Err(anyhow!("Cannot determine app data directory"))
    app_data_dir = base.join("melange")
    tracing::info "Initializing core database at {app_data_dir:?}"
    core_db = CoreDb::open(&app_data_dir).await?
    tracing::info "Core database ready"
    return Ok(core_db)
  ```

- [ ] **9.7** Update `update()`:
  ```
  Message::DbReady(core_db):
    tracing::info "Core database pool received"
    state.core_db = Some(core_db)
    Task::none()
  Message::DbFailed(err): unchanged
  ```

- [ ] **9.8** Update `view()`: change `state.db.is_none()` → `state.core_db.is_none()`.

- [ ] **9.9** Update the module-level doc comment: replace "`App` owns the `SqlitePool`" with "`App` owns the `CoreDb`".

---

## Phase 10 — Verification

- [ ] **10.1** Run `cargo build` — must compile cleanly. This validates:
  - `sqlx::migrate!("migrations/core")` finds the `migrations/core/` directory
  - `sqlx::migrate!("migrations/project")` finds the `migrations/project/` directory
  - All type changes in `app.rs` are consistent
  - No unused import warnings

- [ ] **10.2** Run `cargo test` — all tests must pass:
  - Repository integration tests (4 tests, in-memory SQLite, `#[tokio::test]`)
  - Service unit tests (4 tests, mock repository, `#[tokio::test]`)

- [ ] **10.3** Confirm no offline cache update is required — since this plan uses `sqlx::query(...)` dynamic API (not `query!` macros), the `.sqlx/` cache does not need updating.

---

## Dependency Graph

```
Phase 1 (Cargo.toml)      → must complete before any Rust compilation
Phase 2 (Migrations)       → must complete before Phase 3 (migrate! macro checks paths at compile time)
Phase 3 (db/mod.rs)        → must complete before Phases 5, 6 (CoreDb, ProjectDb types needed)
Phase 4 (domain.rs)        → must complete before Phases 5, 6 (Project, ProjectId etc. needed)
Phase 5 (repository.rs)    → must complete before Phase 6 (ProjectRepository trait needed)
Phase 6 (service.rs)       → must complete before Phase 7
Phase 7 (project/mod.rs)   → must complete before Phase 8
Phase 8 (modules/mod.rs)   → must complete before Phase 9
Phase 9 (app.rs)           → final wiring
Phase 10 (verification)    → runs after all phases complete
```

---

## Edge Cases & Caveats

1. **`CoreDb` must implement `Clone`** — iced's `Message` enum derives `Clone`, so `DbReady(CoreDb)` requires `CoreDb: Clone`. `SqlitePool` is `Clone` (Arc internally). ✓

2. **`sqlx::migrate!` path is compile-time** — Must be a string literal relative to the workspace root. Never a variable. ✓

3. **`INSERT OR FAIL` vs `INSERT OR REPLACE`** — Use `INSERT OR FAIL` so duplicate primary keys produce a detectable error → mapped to `ProjectError::AlreadyExists`. Detect by checking if the sqlx error message/code contains "UNIQUE constraint". ✓

4. **`ProjectError` without `thiserror`** — Implement `std::fmt::Display` and `std::error::Error` manually. Do not add `thiserror`. ✓

5. **Timestamp without `chrono`** — Use `std::time::SystemTime::now().duration_since(UNIX_EPOCH)` to get epoch seconds, then implement a minimal formatter. This avoids adding a `chrono` or `time` dependency. ✓

6. **`MockProjectRepository` must be `Send + Sync`** — Use `std::sync::Mutex<HashMap<...>>`, not `RefCell`. ✓

7. **`ProjectDb` handle dropped after `create_project`** — The service creates the DB file (running project migrations), then drops the pool. Callers retrieve an active `ProjectDb` via `open_project`. This is intentional. ✓
