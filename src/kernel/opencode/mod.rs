//! Opencode startup check domain logic.
//!
//! Provides `OpencodeStatus` for check logic.
//!
//! Note: UI components have been moved to src/ui/opencode.rs

pub mod domain;

pub use domain::{check_opencode_on_path, OpencodeStatus};
