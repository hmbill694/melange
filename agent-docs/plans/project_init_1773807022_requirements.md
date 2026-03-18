# Melange — Project Initialization Requirements

## Overview
Initialize a Rust native desktop application named **melange** using the following stack and architectural principles. This document defines the foundational scaffold that all future features must conform to.

---

## Tech Stack

| Concern       | Technology                          |
|---------------|-------------------------------------|
| Language      | Rust (stable toolchain)             |
| GUI Framework | `iced` (latest compatible version)  |
| Database      | SQLite via `sqlx` (async, offline)  |
| Architecture  | Domain-Driven Design (DDD), Moduliths |

---

## Cargo.toml Requirements

- Single crate (not a workspace for now, can be extended later)
- Crate name: `melange`
- Required dependencies:
  - `iced` with features: `["tokio"]` (async runtime)
  - `sqlx` with features: `["runtime-tokio", "sqlite", "migrate", "macros"]`
  - `tokio` with features: `["full"]`
  - `anyhow` for ergonomic error handling
  - `tracing` + `tracing-subscriber` for structured logging
- No dev dependencies needed at scaffold stage

---

## Directory Structure

```
melange/
├── Cargo.toml
├── Cargo.lock
├── .sqlx/                        # sqlx offline query cache (empty for now)
├── migrations/
│   └── 20260317000000_init.sql   # initial (empty) migration — proves the pipeline works
├── src/
│   ├── main.rs                   # entry point: initialises tracing, DB pool, runs iced app
│   ├── app.rs                    # top-level iced Application struct (App), Message enum, update/view
│   ├── db/
│   │   └── mod.rs                # create_pool() → sqlx::SqlitePool; runs migrations at startup
│   └── modules/
│       └── .gitkeep              # placeholder — no domain modules exist yet
```

---

## Architectural Conventions (must be documented in code comments)

### 1. Modulith Layout
Each bounded context lives under `src/modules/<domain>/` and contains:
- `mod.rs` — re-exports and wires sub-modules
- `domain.rs` — pure Rust structs/enums: entities, value objects, domain events (no I/O)
- `repository.rs` — `sqlx` queries; defines a `Repository` trait and a concrete `SqliteRepository`
- `ui.rs` — iced `view()` function and domain-scoped `Message` enum variants

### 2. Module Isolation
- Modules **must not** import from sibling modules directly
- Cross-module communication happens only via domain events defined in `src/kernel/` (to be created when the first cross-cutting concern appears)
- The top-level `app.rs` is the only place that composes module messages into `app::Message`

### 3. Database
- Connection pool is created once in `main.rs` and passed into the iced application state
- `sqlx::migrate!()` is called at startup before the GUI is shown
- All queries use the `sqlx::query!` / `sqlx::query_as!` macros (compile-time checked)
- Offline mode: `.sqlx/` cache is committed to the repo so CI doesn't need a live DB

### 4. Error Handling
- All fallible functions return `anyhow::Result<T>` unless they are domain-pure functions
- Errors propagate to the `main` function; fatal startup errors are logged and cause a clean exit

### 5. Logging
- `tracing_subscriber` is initialized at the top of `main()` before anything else
- All modules use `tracing::{info, warn, error, debug}` macros (never `println!`)

---

## Deliverables

1. `Cargo.toml` with all dependencies pinned/specified
2. `src/main.rs` — bootstraps tracing, DB pool, and launches the iced app
3. `src/app.rs` — minimal but runnable iced `Application` (empty window with a title)
4. `src/db/mod.rs` — `create_pool(db_path: &str) -> anyhow::Result<SqlitePool>`
5. `migrations/20260317000000_init.sql` — empty migration (just a comment)
6. `src/modules/.gitkeep` — placeholder
7. The application must **compile and run** showing an empty iced window titled "Melange"
8. `cargo build` must succeed with zero errors (warnings acceptable at scaffold stage)

---

## Out of Scope (for this task)
- Any actual domain modules or business logic
- Authentication, routing, or navigation
- Theming or styling beyond defaults
- CI/CD configuration
