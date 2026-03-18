//! Shared kernel — cross-cutting primitives used across all bounded contexts.
//!
//! ## Rules
//! - Kernel modules MUST NOT import from `src/modules/`.
//! - Domain modules communicate with each other only through types defined here.

pub mod loading;
