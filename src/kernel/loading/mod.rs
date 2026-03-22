//! Cross-cutting loading indicator domain logic.
//!
//! Provides `LoadingState` for timing logic.
//!
//! Note: UI components have been moved to src/ui/loading.rs

pub mod domain;

pub use domain::{LoadingState, MIN_LOADING_DURATION};
