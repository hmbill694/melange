//! Cross-cutting loading indicator primitive.
//!
//! Provides `LoadingState` for timing logic and `loading_indicator` for the iced view.

pub mod domain;
pub mod ui;

pub use domain::{LoadingState, MIN_LOADING_DURATION};
pub use ui::loading_indicator;
