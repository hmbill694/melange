# Melange — Codebase Map

Each entry follows the format: `<file-path> — <one-line summary>`

## Project Root
Cargo.toml — Rust crate manifest; declares melange v0.1.0 with iced 0.14, sqlx 0.8, tokio, anyhow, and tracing dependencies
Cargo.lock — Auto-generated dependency lock file

## Source Files
src/main.rs — Entry point; initialises tracing subscriber and launches the iced GUI via `iced::application(app::new, app::update, app::view).title("Melange").subscription(app::subscription).run()`
src/app.rs — Top-level iced application: defines `App` state (incl. `loading_state: LoadingState`, `tick_count: u32`), root `Message` enum (incl. `Tick`, `LoadingDone`), and `new`/`update`/`view`/`subscription` free functions; the sole compositor of domain module messages
src/kernel/mod.rs — Shared kernel namespace; declares `pub mod loading` for cross-cutting UI primitives; kernel modules must not import from `src/modules/`
src/kernel/loading/mod.rs — Re-exports `LoadingState`, `MIN_LOADING_DURATION`, `min_duration_elapsed`, and `loading_indicator` as the flat public API for the loading primitive
src/kernel/loading/domain.rs — Pure domain logic: `LoadingState` enum (Idle/Loading{started_at}/Done), `MIN_LOADING_DURATION` constant (300ms), `min_duration_elapsed` pure function; 4 unit tests covering boundary conditions and default state
src/kernel/loading/ui.rs — `loading_indicator<'a, Message>(label, tick_count) -> Element` reusable iced view component; braille spinner (8-frame) driven by tick_count mod 8, generic over Message
src/db/mod.rs — Database connection module; defines `CoreDb` and `ProjectDb` typed pool wrappers with `open`/`create`/`from_pool` constructors and embedded migration runners for each DB type
src/modules/mod.rs — Namespace for all DDD bounded-context modules; declares `pub mod project` and enforces modulith isolation rules via doc comments
src/modules/project/mod.rs — Re-exports all public types from the project bounded context (Project, ProjectId, CreateProjectCommand, ProjectError, ProjectRepository, SqliteProjectRepository, ProjectService)
src/modules/project/domain.rs — Pure domain types for the project context: `ProjectId` newtype (UUID), `Project` entity, `CreateProjectCommand` value object, `ProjectError` enum with manual Display/Error impls
src/modules/project/repository.rs — `ProjectRepository` async trait + `SqliteProjectRepository` impl backed by `CoreDb`; includes 4 in-memory integration tests covering save, find-by-id, find-all, and duplicate-key error
src/modules/project/service.rs — `ProjectService<R>` generic over `ProjectRepository`; implements `create_project`, `list_projects`, `open_project`; includes `MockProjectRepository` and 4 unit tests

## Migrations
migrations/core/20260317000000_init.sql — Relocated no-op initial migration for the core database (formerly at migrations/20260317000000_init.sql)
migrations/core/20260317000001_create_projects.sql — Creates the `projects` table in the core DB (id TEXT PK, name, db_path, created_at)
migrations/project/20260317000000_init.sql — No-op placeholder migration for the per-project database pipeline

## Tooling
.sqlx/.gitkeep — Tracks the sqlx offline query-cache directory in git; populated by `cargo sqlx prepare` when `query!` macros are added
