# Melange — Project Initialization Implementation Plan

## Overview

This plan initializes a Rust desktop application named **melange** using iced 0.14.0 (functional API), sqlx 0.8.6 (async SQLite), tokio, anyhow, and tracing. The application uses iced 0.14.0's `iced::application(boot, update, view)` builder pattern. DB migrations run asynchronously via `Task::perform` on boot; the window opens immediately with a loading state.

---

## Dependency Versions

```toml
[package]
name = "melange"
version = "0.1.0"
edition = "2024"

[dependencies]
iced = { version = "0.14.0", features = ["tokio"] }
sqlx = { version = "0.8.6", features = ["runtime-tokio", "sqlite", "migrate", "macros"] }
tokio = { version = "1", features = ["full"] }
anyhow = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

---

## Files to Create (in order)

### 1. `Cargo.toml`
### 2. `migrations/20260317000000_init.sql`
### 3. `.sqlx/.gitkeep`
### 4. `src/modules/mod.rs`
### 5. `src/db/mod.rs`
### 6. `src/app.rs`
### 7. `src/main.rs`

---

## Detailed File Descriptions

---

### File 1: `Cargo.toml`

**Purpose:** Declare the crate, all dependencies with features, and lock the Rust edition.

**Contents (pseudocode description):**

```
[package]
  name = "melange"
  version = "0.1.0"
  edition = "2024"          ← required for iced 0.14.0

[dependencies]
  iced = "0.14.0" with features ["tokio"]
      → iced's tokio feature wires iced_futures to use tokio as its executor
      → this is mandatory for Task::perform to work with async sqlx code

  sqlx = "0.8.6" with features:
      ["runtime-tokio", "sqlite", "migrate", "macros"]
      → runtime-tokio: use tokio for connection pool async
      → sqlite: enable the SQLite driver
      → migrate: enable sqlx::migrate!() macro and migration runner
      → macros: enable query! / query_as! compile-time checked macros

  tokio = "1" with features ["full"]
      → needed for block_on, spawn, etc. within tasks

  anyhow = "1"
      → ergonomic error propagation with ? operator

  tracing = "0.1"
      → structured logging macros (info!, warn!, error!, debug!)

  tracing-subscriber = "0.3" with features ["env-filter"]
      → initialize the global tracing subscriber with RUST_LOG support
```

**Tricky note:** iced 0.14.0 requires `edition = "2024"` — using `edition = "2021"` will cause compile errors with some of iced's internal `impl Trait` usage.

---

### File 2: `migrations/20260317000000_init.sql`

**Purpose:** Prove the sqlx migration pipeline works end-to-end. The file is intentionally empty of schema changes — it's a no-op migration that validates the `migrate!()` macro and `MigrateDatabase::migrate()` call succeed without error.

**Contents:**
```sql
-- Initial migration — scaffold only. No tables created yet.
-- All domain tables will be added in subsequent migrations.
```

**Why this matters:** sqlx's `migrate!()` macro scans the `migrations/` directory at compile time and embeds them. If the directory doesn't exist or the SQL files are malformed, compilation fails. The `.sql` file must be syntactically valid SQL even if it contains only comments.

---

### File 3: `.sqlx/.gitkeep`

**Purpose:** Create the `.sqlx/` directory (sqlx offline query cache). This directory must exist in the repo so CI doesn't fail. Currently empty because no `query!` macros are used yet.

**Contents:** Empty file.

**Why this matters:** When `SQLX_OFFLINE=true` is set (e.g., in CI), sqlx reads cached query metadata from `.sqlx/`. Without the directory, the offline flag causes build failures.

---

### File 4: `src/modules/mod.rs`

**Purpose:** Declares the `modules` namespace so `mod modules;` in `main.rs` compiles. Contains only an architectural doc comment — no code. (A `.gitkeep` alone won't satisfy rustc.)

**Contents (pseudocode):**
```
//! Domain modules (bounded contexts / moduliths).
//!
//! ## Modulith Layout
//! Each bounded context lives under `src/modules/<domain>/` and contains:
//!   - `mod.rs`        — re-exports and wires sub-modules
//!   - `domain.rs`     — pure Rust structs/enums: entities, value objects, domain events (no I/O)
//!   - `repository.rs` — sqlx queries; defines a Repository trait and a SqliteRepository impl
//!   - `ui.rs`         — iced view() function and domain-scoped Message enum variants
//!
//! ## Module Isolation Rules
//!   - Modules MUST NOT import from sibling modules directly.
//!   - Cross-module communication happens only via domain events defined in `src/kernel/`
//!     (to be created when the first cross-cutting concern appears).
//!   - The top-level `app.rs` is the ONLY place that composes module messages into app::Message.

// No code here yet — modules will be declared as they are added.
// Example (do not add until the module exists):
// pub mod example_domain;
```

---

### File 5: `src/db/mod.rs`

**Purpose:** Encapsulate all database connection concerns. Exposes a single public function `create_pool`.

**Module declaration:** This file lives at `src/db/mod.rs`, which means `src/main.rs` must declare `mod db;`.

**Pseudocode:**

```
//! Database connection management.
//!
//! ## Design Notes
//! - Pool is created ONCE at startup and stored in `App` state.
//! - `SqlitePool` is `Clone + Send + Sync` — clone it cheaply when passing to repositories.
//! - All queries use `sqlx::query!` / `sqlx::query_as!` macros (compile-time checked).
//! - Offline mode: `.sqlx/` cache is committed to the repo so CI does not need a live DB.
//!   Run `cargo sqlx prepare` after adding or modifying any `query!` / `query_as!` calls.
//!   Set `DATABASE_URL=sqlite://melange.db` when running `cargo sqlx prepare`.

use sqlx::SqlitePool
use anyhow::Result

// PUBLIC FUNCTION
async function create_pool(db_url: &str) -> Result<SqlitePool>:
    // 1. Create or open the SQLite file at db_url
    //    db_url format: "sqlite://./melange.db" or "sqlite://:memory:"
    //    Use SqlitePool::connect(db_url) → returns SqlitePool on success
    //    Propagate any connection error with ? (anyhow::Result)

    // 2. Run embedded migrations
    //    sqlx::migrate!() expands to a static Migrator that embeds all
    //    .sql files found in the "migrations/" directory at compile time.
    //    Call: sqlx::migrate!().run(&pool).await?
    //    This is idempotent — already-applied migrations are skipped.
    //    Log: tracing::info!("Database migrations applied successfully")

    // 3. Return the pool
    //    return Ok(pool)

// EDGE CASES:
//   - If the database file path doesn't exist, SQLite creates it automatically.
//   - If migrations have already run, sqlx's _sqlx_migrations table prevents re-running.
//   - Connection errors (permissions, invalid path) propagate as anyhow::Error.
//   - The db_url MUST start with "sqlite://" — sqlx requires the scheme prefix.
```

**Data shape:**
- Input: `db_url: &str` — e.g., `"sqlite://melange.db"` (relative to working directory)
- Output: `anyhow::Result<sqlx::SqlitePool>` — the pool is `Clone` and `Send + Sync`

---

### File 6: `src/app.rs`

**Purpose:** Defines the top-level iced application state (`App`), the `Message` enum, and the `new`, `update`, and `view` functions that are passed to `iced::application(...)`.

**⚠️ Critical API note for iced 0.14.0:**
The old `Application` trait (implement `new`, `update`, `view`, `title`) is **gone**. Instead:
```
iced::application(boot_fn, update_fn, view_fn)
    .title("Melange")
    .run()
```
- `boot_fn` returns `(State, Task<Message>)` — a free function, not a method
- `update_fn` signature: `fn update(state: &mut App, message: Message) -> Task<Message>`
- `view_fn` signature: `fn view(state: &App) -> Element<'_, Message>`

**Pseudocode:**

```
//! Top-level iced application state, messages, and handlers.
//!
//! ## Architectural Rules (enforced here)
//! - This is the ONLY file that composes domain module Messages into the root Message enum.
//! - No business logic lives in this file — delegate everything to modules.
//! - `App` owns the `SqlitePool`; repositories receive a cloned reference.
//! - Module messages are wrapped as variants: e.g., Message::Foo(modules::foo::Message).

use iced::{Element, Task}
use iced::widget::{column, text, container}
use iced::Fill
use sqlx::SqlitePool
use anyhow::Result

// ─── STATE ───────────────────────────────────────────────────────────────────

// PUBLIC STRUCT (must be 'static — no non-'static references)
struct App:
    // Holds the database pool. None until the async init task completes.
    db: Option<SqlitePool>

    // Holds a startup error message if DB initialization fails.
    // Displayed instead of the main UI when Some.
    init_error: Option<String>

// Implement Default for App:
//   db = None
//   init_error = None

// ─── MESSAGES ────────────────────────────────────────────────────────────────

// PUBLIC ENUM — derive Debug, Clone
enum Message:
    // Sent when async DB init succeeds. Carries the ready pool.
    DbReady(SqlitePool)

    // Sent when async DB init fails. Carries the error as a String.
    DbFailed(String)

    // Future variants added here as new modules are introduced.

// ─── BOOT FUNCTION ───────────────────────────────────────────────────────────

// Called by iced at startup. Returns initial state + a Task that resolves the DB.
PUBLIC function new() -> (App, Task<Message>):
    let initial_state = App::default()

    // Task::perform takes:
    //   1. An async BLOCK (not a closure) returning a value
    //   2. A mapping fn: value → Message
    let task = Task::perform(
        async { init_db().await },
        |result| match result:
            Ok(pool)  → Message::DbReady(pool)
            Err(e)    → Message::DbFailed(e.to_string())
    )

    return (initial_state, task)

// Private async helper — called only by new()
async function init_db() -> Result<SqlitePool>:
    tracing::info!("Initializing database...")
    let pool = crate::db::create_pool("sqlite://melange.db").await?
    tracing::info!("Database ready")
    return Ok(pool)

// ─── UPDATE FUNCTION ─────────────────────────────────────────────────────────

PUBLIC function update(state: &mut App, message: Message) -> Task<Message>:
    match message:
        Message::DbReady(pool):
            tracing::info!("Database pool received by application state")
            state.db = Some(pool)
            Task::none()

        Message::DbFailed(err):
            tracing::error!("Database initialization failed: {}", err)
            state.init_error = Some(err)
            Task::none()

// ─── VIEW FUNCTION ───────────────────────────────────────────────────────────

PUBLIC function view(state: &App) -> Element<'_, Message>:
    if state.init_error is Some(ref err):
        // Centered error message
        container(
            text(format!("Failed to initialize database: {}", err))
        )
        .center(Fill)
        .into()

    else if state.db is None:
        // Loading state
        container(text("Initializing..."))
            .center(Fill)
            .into()

    else:
        // Ready state — main UI placeholder
        container(
            column![
                text("Melange").size(32),
                text("Ready.")
            ]
            .spacing(10)
        )
        .center(Fill)
        .into()
```

**Tricky notes:**
- `SqlitePool` is `Clone + Send + Sync + 'static` — safe to store in `Message` and clone for repositories
- `Message` must derive `Clone` because `SqlitePool: Clone` — this works fine
- `Task::perform`'s first arg is an `async { }` block (a `Future`), NOT a closure
- `.center(Fill)` is the iced 0.14.0 API for centering — use it instead of `.center_x(Fill).center_y(Fill)`

---

### File 7: `src/main.rs`

**Purpose:** Entry point. Initializes tracing, then launches the iced GUI. No business logic.

**Pseudocode:**

```
//! Application entry point.
//!
//! ## Startup Sequence
//! 1. Initialize tracing subscriber (reads RUST_LOG; defaults to "melange=debug,sqlx=warn")
//! 2. Launch iced application — window opens immediately
//! 3. Async DB init fires via Task::perform in app::new()
//! 4. On DB ready: App state receives the pool and renders the main UI
//!
//! ## Adding New Domain Modules
//! Create `src/modules/<domain>/` following the modulith layout documented in
//! `src/modules/mod.rs`. Declare the submodule in `src/modules/mod.rs` only —
//! never import a module directly from main.rs or from a sibling module.

mod app;
mod db;
mod modules;

// main returns iced::Result (NOT anyhow::Result)
// iced::Result = Result<(), iced::Error>
// This lets iced cleanly report renderer/window failures.
function main() -> iced::Result:

    // Step 1: Initialize tracing BEFORE anything else.
    // Reads RUST_LOG env var; falls back to default filter string.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("melange=debug,sqlx=warn"))
        )
        .init()

    tracing::info!("Starting Melange")

    // Step 2: Launch the iced application.
    // Pattern: iced::application(boot, update, view).title(...).run()
    //
    // boot   = app::new    → (App, Task<Message>), called once at startup
    // update = app::update → (&mut App, Message) -> Task<Message>
    // view   = app::view   → (&App) -> Element<'_, Message>
    //
    // .title("Melange") sets the OS window title bar
    // .run() blocks until the window is closed; returns iced::Result
    iced::application(app::new, app::update, app::view)
        .title("Melange")
        .run()
```

---

## Architecture Documentation Requirements

Each file must include a module-level doc comment (`//!` block) documenting architectural conventions. The doc comments in the pseudocode above serve as the specification for these comments.

---

## Ordered Checklist

- [ ] **Step 1:** Create `Cargo.toml` with edition `"2024"` and all dependencies as specified.
- [ ] **Step 2:** Create `migrations/20260317000000_init.sql` with only a SQL comment (no DDL).
- [ ] **Step 3:** Create `.sqlx/.gitkeep` (empty file to track the `.sqlx/` directory in git).
- [ ] **Step 4:** Create `src/modules/mod.rs` with the architectural doc comment and no code.
- [ ] **Step 5:** Create `src/db/mod.rs` implementing `create_pool(db_url: &str) -> anyhow::Result<SqlitePool>`.
- [ ] **Step 6:** Create `src/app.rs` with `App`, `Message`, `new`, `update`, and `view` as described.
- [ ] **Step 7:** Create `src/main.rs` with tracing init and `iced::application(...).title("Melange").run()`.

---

## Tricky Parts Summary

| Issue | Resolution |
|---|---|
| iced 0.14.0 uses builder pattern, not `Application` trait | Use `iced::application(boot, update, view).title(...).run()` |
| `new()` boot fn returns `(App, Task<Message>)` for async init | Return a `Task::perform` that resolves the DB pool |
| `Task::perform` first arg is a `Future`, not a closure | Use `async { init_db().await }` block |
| sqlx `migrate!()` macro scans `migrations/` at compile time | Directory and at least one `.sql` file must exist |
| `mod modules;` in main.rs needs `src/modules/mod.rs` | Create `src/modules/mod.rs` (not just `.gitkeep`) |
| `.center_x/.center_y` are removed in iced 0.14 | Use `.center(Fill)` instead |
| `tracing_subscriber` must be first | First lines of `main()` before any `tracing::` call |
| `main()` must return `iced::Result` | Use `-> iced::Result`, not `anyhow::Result` |

---

## Verification Section

### What `cargo build` should produce:
- **Zero errors**
- Possible warnings: unused `mod modules` (acceptable at scaffold stage)
- sqlx compile-time migration scan succeeds (migrations dir + valid .sql exists)
- No `DATABASE_URL` env var needed — `query!` macros are not used yet
- Binary artifact: `target/debug/melange`

### What the user sees when running `cargo run`:
1. Tracing output in terminal:
   ```
   INFO melange: Starting Melange
   INFO melange::app: Initializing database...
   INFO melange::app: Database ready
   INFO melange::app: Database pool received by application state
   ```
2. A native desktop window opens titled **"Melange"**
3. Window briefly shows **"Initializing..."** while the async DB task runs
4. Window transitions to show **"Melange"** (size 32) and **"Ready."** centered
5. `melange.db` file is created in the project root on first run
6. Closing the window exits cleanly (exit code 0)

### Manual Smoke Test Checklist:
- [ ] `cargo build` exits with code 0
- [ ] `cargo run` opens a window titled "Melange"
- [ ] `melange.db` appears in the project root after first run
- [ ] No panics or error output in terminal
- [ ] Window closes cleanly on pressing the OS close button
