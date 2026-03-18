# Requirements: Database Foundation

## Overview
Establish the SQLite database layer for Melange using a dual-database strategy:
- A **core database** (`melange.db`) for application-wide data (projects, settings, future workflows)
- A **project database** (one per project, `<uuid>.db`) for project-scoped data

All design follows Domain-Driven Design, Modulith isolation, and a test-first (TDD) approach.

---

## 1. Core Database

- On app startup, resolve the platform app-data directory:
  - macOS: `~/Library/Application Support/melange/melange.db`
  - Linux: `~/.local/share/melange/melange.db`
  - Windows: `%APPDATA%\melange\melange.db`
- Create the directory if it does not exist
- Open (or create) `melange.db` at that path
- Run embedded migrations from `migrations/core/` using `sqlx::migrate!("migrations/core")`
- The core DB contains at minimum a `projects` table:
  - `id` — UUID (stored as TEXT), primary key
  - `name` — TEXT, not null
  - `db_path` — TEXT, not null — absolute path to the project's `.db` file
  - `created_at` — TEXT (ISO 8601 timestamp), not null

---

## 2. Project Database

- Each project database lives at: `<app-data>/melange/projects/<uuid>.db`
- On project creation:
  1. Derive the `db_path` from the project UUID
  2. Create/open the SQLite file at that path
  3. Run embedded migrations from `migrations/project/` using `sqlx::migrate!("migrations/project")`
  4. Return a live `SqlitePool` for the project
- The project pool is kept separate from the core pool

---

## 3. Migration Strategy

- Two embedded migration directories:
  - `migrations/core/` — applied to the core DB
  - `migrations/project/` — applied to each new project DB
- Both directories use sqlx's standard idempotent migration tracking (`_sqlx_migrations` table)
- The existing `migrations/20260317000000_init.sql` is relocated to `migrations/core/`
- An equivalent no-op initial migration is created at `migrations/project/` for the project DB

---

## 4. Module & DDD Architecture

### `src/db/`
- Expose two typed wrappers:
  - `CoreDb` — wraps a `SqlitePool` for the core database
  - `ProjectDb` — wraps a `SqlitePool` for a project database
- A `CoreDb::open(app_data_dir: &Path) -> Result<CoreDb>` async function
- A `ProjectDb::create(db_path: &Path) -> Result<ProjectDb>` async function (creates + migrates)
- A `ProjectDb::open(db_path: &Path) -> Result<ProjectDb>` async function (opens existing + migrates)
- Both types derive `Clone` (SqlitePool is clone-cheap)

### `src/modules/project/` — Project Bounded Context
- `mod.rs` — re-exports and wires sub-modules
- `domain.rs` — pure Rust types, no I/O:
  - `ProjectId` — newtype wrapping `uuid::Uuid`
  - `Project` — entity with fields: `id: ProjectId`, `name: String`, `db_path: PathBuf`
  - `CreateProjectCommand` — value object: `name: String`
  - `ProjectError` — domain error enum
- `repository.rs`:
  - `ProjectRepository` trait (async_trait) with methods:
    - `save(&self, project: &Project) -> Result<(), ProjectError>`
    - `find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, ProjectError>`
    - `find_all(&self) -> Result<Vec<Project>, ProjectError>`
  - `SqliteProjectRepository` — implements the trait against a `CoreDb`
- `service.rs`:
  - `ProjectService<R: ProjectRepository>` — generic over the repository
  - Methods:
    - `create_project(&self, cmd: CreateProjectCommand) -> Result<Project, ProjectError>`
      - Generates a UUID, derives `db_path`, creates `ProjectDb`, persists via repository
    - `list_projects(&self) -> Result<Vec<Project>, ProjectError>`
    - `open_project(&self, id: &ProjectId) -> Result<ProjectDb, ProjectError>`

### `src/app.rs` (updated)
- Replace the existing inline `create_pool("sqlite://melange.db")` call
- Boot sequence:
  1. Resolve app-data directory
  2. Call `CoreDb::open(app_data_dir)` → `CoreDb`
  3. Store `CoreDb` in `App` state (replacing `Option<SqlitePool>`)
  4. Instantiate `SqliteProjectRepository` and `ProjectService` from the core DB
- `App` state changes: `db: Option<SqlitePool>` → `core_db: Option<CoreDb>`

---

## 5. Testability Requirements

- All repository traits use `async_trait` so they are object-safe and mockable
- `SqliteProjectRepository` integration tests:
  - Each test creates an in-memory pool: `SqlitePool::connect("sqlite::memory:").await`
  - Applies `migrations/core/` migrations to the in-memory pool before each test
  - Tests: save a project → find by id, find all, save duplicate id (expect error)
- `ProjectService` unit tests:
  - Define a `MockProjectRepository` in the test module implementing `ProjectRepository`
  - Tests exercise service logic without any DB I/O
  - Tests: create project returns correct entity, create project calls repository save, list delegates to repository
- Tests live in `#[cfg(test)]` modules at the bottom of each source file
- A `uuid` crate dependency must be added to `Cargo.toml` with the `v4` feature

---

## 6. Dependency Changes

Add to `Cargo.toml`:
- `uuid = { version = "1", features = ["v4"] }`
- `async-trait = "0.1"`
- `dirs = "5"` (for platform app-data directory resolution)

---

## 7. File Structure After Implementation

```
migrations/
  core/
    20260317000000_init.sql          (relocated, no-op)
    20260317000001_create_projects.sql
  project/
    20260317000000_init.sql          (new, no-op placeholder)

src/
  db/
    mod.rs    (CoreDb, ProjectDb)
  modules/
    mod.rs
    project/
      mod.rs
      domain.rs
      repository.rs
      service.rs
  app.rs      (updated boot sequence)
  main.rs     (unchanged)
```
