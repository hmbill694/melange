//! Application-level UI module.
//!
//! This module consolidates all UI components and logic specific to the
//! main application shell, including state management, view composition,
//! and update handling.

// Re-export state types
// Note: CurrentScreen and CreateProjectState are re-exported for external module usage
#[allow(unused_imports)]
pub use state::{HomeScreenState, CurrentScreen, CreateProjectState};

// Re-export view functions
pub use view::view_app;

// Re-export update functions and context types
pub use update::{handle_update, UpdateContext, HomeScreenUpdateContext};

// Declare submodules
pub mod state;
mod view;
mod update;