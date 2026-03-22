//! Application-level UI module.
//!
//! This module consolidates all UI components and logic specific to the
//! main application shell, including state management, view composition,
//! and update handling.

// Re-export state types
pub use state::HomeScreenState;

// Re-export view functions
pub use view::view_app;

// Re-export update functions and context types
pub use update::{handle_update, UpdateContext, HomeScreenUpdateContext};

// Declare submodules
mod state;
mod view;
mod update;