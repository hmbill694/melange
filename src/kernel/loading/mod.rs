//! Cross-cutting loading indicator primitive.
//!
//! Provides `LoadingState` for timing logic and `loading_indicator` for the iced view.

pub mod domain;
pub mod ui;

pub use domain::{min_duration_elapsed, LoadingState, MIN_LOADING_DURATION};
pub use ui::loading_indicator;
