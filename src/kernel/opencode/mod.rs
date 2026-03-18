//! Opencode startup check primitive.
//!
//! Provides `OpencodeStatus` for check logic and `opencode_not_found_screen` for the iced view.

pub mod domain;
pub mod ui;

pub use domain::{check_opencode_on_path, OpencodeStatus};
pub use ui::opencode_not_found_screen;
