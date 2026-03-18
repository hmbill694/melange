//! Database connection management.
//!
//! ## Design Notes
//! - `CoreDb` manages the single application-wide SQLite database that stores core data
//!   (e.g. the projects registry). `App` owns a `CoreDb` instance.
//! - `ProjectDb` manages a per-project SQLite database. Each project has its own file.
//! - Both structs wrap a `SqlitePool` (which is `Clone + Send + Sync` via Arc internally).
//! - All queries use the dynamic `sqlx::query(...)` API — no `query!` macros — so the
//!   `.sqlx/` offline cache does not need updating when migrations change.

use anyhow::{anyhow, Result};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::SqlitePool;
use std::path::Path;
use std::str::FromStr;

// ─── CoreDb ──────────────────────────────────────────────────────────────────

/// Typed wrapper around the core application SQLite pool.
///
/// Holds the `melange.db` connection pool and applies the `migrations/core/`
/// migration set on construction. Clone is cheap (Arc internally).
#[derive(Debug, Clone)]
pub struct CoreDb {
    pool: SqlitePool,
}

impl CoreDb {
    /// Open (or create) the core database at `<app_data_dir>/melange.db`,
    /// run embedded core migrations, and return the ready `CoreDb`.
    pub async fn open(app_data_dir: &Path) -> Result<CoreDb> {
        let db_path = app_data_dir.join("melange.db");

        // Ensure the parent directory exists (idempotent).
        std::fs::create_dir_all(app_data_dir)?;

        let url = format!("sqlite://{}", db_path.display());
        let opts = SqliteConnectOptions::from_str(&url)?.create_if_missing(true);
        let pool = SqlitePool::connect_with(opts).await?;

        // Run embedded migrations from migrations/core/.
        sqlx::migrate!("migrations/core").run(&pool).await?;

        tracing::info!("Core database ready at {:?}", db_path);
        Ok(CoreDb { pool })
    }

    /// Borrow the underlying connection pool.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Construct a `CoreDb` from an already-open pool.
    ///
    /// Intended for use in integration tests where the pool is backed by
    /// `sqlite::memory:` and migrations have already been applied.
    pub(crate) fn from_pool(pool: SqlitePool) -> CoreDb {
        CoreDb { pool }
    }
}

// ─── ProjectDb ───────────────────────────────────────────────────────────────

/// Typed wrapper around a per-project SQLite pool.
///
/// Each project has its own `.db` file. `ProjectDb` applies the
/// `migrations/project/` migration set on construction. Clone is cheap.
#[derive(Clone)]
pub struct ProjectDb {
    pool: SqlitePool,
}

impl ProjectDb {
    /// Create a new project database at `db_path`, run embedded project
    /// migrations, and return the ready `ProjectDb`.
    pub async fn create(db_path: &Path) -> Result<ProjectDb> {
        // Ensure parent directory exists.
        std::fs::create_dir_all(db_path.parent().unwrap_or(Path::new(".")))?;

        let url = format!("sqlite://{}", db_path.display());
        let opts = SqliteConnectOptions::from_str(&url)?.create_if_missing(true);
        let pool = SqlitePool::connect_with(opts).await?;

        sqlx::migrate!("migrations/project").run(&pool).await?;

        tracing::info!("Project database created at {:?}", db_path);
        Ok(ProjectDb { pool })
    }

    /// Open an existing project database at `db_path`, run embedded project
    /// migrations (idempotent), and return the ready `ProjectDb`.
    pub async fn open(db_path: &Path) -> Result<ProjectDb> {
        if !db_path.exists() {
            return Err(anyhow!("Project DB not found: {}", db_path.display()));
        }

        let url = format!("sqlite://{}", db_path.display());
        let pool = SqlitePool::connect(&url).await?;

        sqlx::migrate!("migrations/project").run(&pool).await?;

        tracing::info!("Project database opened at {:?}", db_path);
        Ok(ProjectDb { pool })
    }

    /// Borrow the underlying connection pool.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
