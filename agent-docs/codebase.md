# Melange — Codebase Map

Each entry follows the format: `<file-path> — <one-line summary>`

## Project Root
Cargo.toml — Rust crate manifest; declares melange v0.1.0 with iced 0.14, sqlx 0.8, tokio, anyhow, and tracing dependencies
Cargo.lock — Auto-generated dependency lock file

## Source Files
src/main.rs — Entry point; initialises tracing subscriber and launches the iced GUI via `iced::application(app::new, app::update, app::view).title("Melange").subscription(app::subscription).run()`
src/app.rs — Top-level iced application: defines `App` state (incl. `loading_state`, `tick_count`, `opencode_status`, `home_screen_state: HomeScreenState`, `window_width: f32`), root `Message` enum (incl. `Tick`, `LoadingDone`, `OpencodeReady`, `OpencodeNotFound`, `Project(ProjectMessage)`, `WindowResized(f32)`), and `new`/`update`/`view`/`subscription` free functions; fires opencode check + DB init tasks at startup, then fires project-load task on `DbReady`; subscribes to window resize events; `view()` composes `app_bar()` above a `screen_content` determined by the 4-priority chain (opencode-not-found → db-error → loading → home screen)
src/kernel/mod.rs — Shared kernel namespace; declares `pub mod app_bar`, `pub mod loading`, and `pub mod opencode` for cross-cutting domain logic; kernel modules must not import from `src/modules/`
src/kernel/app_bar/mod.rs — Re-exports domain logic for the app bar primitive
src/kernel/loading/mod.rs — Re-exports `LoadingState`, `MIN_LOADING_DURATION`, `min_duration_elapsed` as the flat public API for the loading primitive
src/kernel/loading/domain.rs — Pure domain logic: `LoadingState` enum (Idle/Loading{started_at}/Done), `MIN_LOADING_DURATION` constant (300ms), `min_duration_elapsed` pure function; 4 unit tests covering boundary conditions and default state
src/kernel/opencode/mod.rs — Re-exports `OpencodeStatus` and `check_opencode_on_path` as the flat public API for the opencode startup check primitive
src/kernel/opencode/domain.rs — Pure domain types: `OpencodeStatus` enum (Found/NotFound), async `check_opencode_on_path()` using `tokio::task::spawn_blocking` + `std::process::Command`; 3 unit tests
src/db/mod.rs — Database connection module; defines `CoreDb` and `ProjectDb` typed pool wrappers with `open`/`create`/`from_pool` constructors and embedded migration runners for each DB type
src/modules/mod.rs — Namespace for all DDD bounded-context modules; declares `pub mod project` and enforces modulith isolation rules via doc comments
src/modules/project/mod.rs — Re-exports all public types from the project bounded context (Project, ProjectId, CreateProjectCommand, ProjectError, ProjectRepository, SqliteProjectRepository, ProjectService, ProjectMessage); declares domain/repository/service/message submodules
src/modules/project/domain.rs — Pure domain types for the project context: `ProjectId` newtype (UUID), `Project` entity (incl. `description: Option<String>` and `file_path: PathBuf`), `CreateProjectCommand` value object, `ProjectError` enum with manual Display/Error impls
src/modules/project/repository.rs — `ProjectRepository` async trait + `SqliteProjectRepository` impl backed by `CoreDb`; SQL queries include `description` and `file_path` columns; 4 in-memory integration tests
src/modules/project/service.rs — `ProjectService<R>` generic over `ProjectRepository`; implements `create_project` (passes `description` and `file_path` from command), `list_projects`, `open_project`; includes `MockProjectRepository` and 4 unit tests
src/modules/project/message.rs — `ProjectMessage` enum with `SearchChanged(String)`, `ProjectsLoaded(Vec<Project>)`, `LoadFailed(String)`, navigation messages (`NavigateToCreateProject`, `NavigateToHome`), form update messages (`CreateProjectNameChanged(String)`, `CreateProjectPathChanged(String)`), and submission messages (`CreateProjectSubmitted`, `CreateProjectSucceeded(Project)`, `CreateProjectFailed(String)`); composed into root `Message::Project` in `app.rs`
src/ui/mod.rs — UI module namespace; declares `pub mod app_bar`, `pub mod loading`, `pub mod opencode`, and `pub mod project` for all UI components
src/ui/app_bar.rs — `app_bar<'a, Message>() -> Element<'a, Message>` reusable iced view component; renders a full-width container with "Melange" centered; generic over Message; no props
src/ui/loading.rs — `loading_indicator<'a, Message>(label, tick_count) -> Element` reusable iced view component; braille spinner (8-frame) driven by tick_count mod 8, generic over Message
src/ui/opencode.rs — `opencode_not_found_screen<'a, Message>() -> Element` generic iced view; renders centered "opencode is required" blocked screen with install URL as selectable plain text
src/ui/project.rs — `home_screen(projects, search_query, window_width, on_create_project)` view function with Create Project button next to search bar; filters by name/description; renders 3-column card grid when width ≥ 900px, single-column list otherwise; `create_project_screen(state, on_name_changed, on_path_changed, on_submit, on_cancel, on_back)` view function for project creation form with header, form inputs, and action buttons
src/ui/app/mod.rs — Re-exports for the app UI module; exposes `HomeScreenState`, `view_app`, `handle_update`, `UpdateContext`, and `HomeScreenUpdateContext`
src/ui/app/state.rs — Application UI state: `HomeScreenState` struct with `projects: Vec<Project>`, `search_query: String`, `current_screen: CurrentScreen` (Home/CreateProject enum), and `create_project_state: CreateProjectState` (holds form values, is_submitting flag, error_message); `CurrentScreen` and `CreateProjectState` types for screen navigation and form management
src/ui/app/view.rs — `view_app()` function composing the main application view; implements 4-priority screen chain (opencode-not-found → db-error → loading → home/create-project screen); routes between Home and CreateProject screens based on `current_screen` state
src/ui/app/update.rs — `handle_update()` function with `UpdateContext` and `HomeScreenUpdateContext` structs; handles all root Message variants and delegates to appropriate handlers
src/app/service.rs — Application-level service functions; `init_db()` async function for core database initialization with proper logging

## Migrations
migrations/core/20260317000000_init.sql — Relocated no-op initial migration for the core database (formerly at migrations/20260317000000_init.sql)
migrations/core/20260317000001_create_projects.sql — Creates the `projects` table in the core DB (id TEXT PK, name, db_path, created_at)
migrations/project/20260317000000_init.sql — No-op placeholder migration for the per-project database pipeline
migrations/core/20260317000002_add_project_fields.sql — Adds `description TEXT` (nullable) and `file_path TEXT NOT NULL DEFAULT ''` columns to the `projects` table

## Tooling
.sqlx/.gitkeep — Tracks the sqlx offline query-cache directory in git; populated by `cargo sqlx prepare` when `query!` macros are added
