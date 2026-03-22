//! Application-level service functions.
//!
//! This module contains async service functions for application initialization
//! and coordination tasks that don't belong in the UI layer.

use crate::db::CoreDb;
use anyhow::{anyhow, Result};

use tracing;

/// Initialize the core database.
///
/// This async function:
/// 1. Determines the application data directory using dirs::data_dir()
/// 2. Creates the melange subdirectory if needed
/// 3. Opens/creates the core database
///
/// Returns: Result<CoreDb> - the initialized database handle or an error
pub async fn init_db() -> Result<CoreDb> {
    // Determine base data directory
    let base = dirs::data_dir().ok_or_else(|| anyhow!("Cannot determine app data directory"))?;
    
    let app_data_dir = base.join("melange");
    
    tracing::info!("Initializing core database at {:?}", app_data_dir);
    
    // Open the core database
    let core_db = CoreDb::open(&app_data_dir).await?;
    
    tracing::info!("Core database ready");
    
    Ok(core_db)
}