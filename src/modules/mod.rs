//! Domain modules (bounded contexts / moduliths).
//!
//! ## Modulith Layout
//! Each bounded context lives under `src/modules/<domain>/` and contains:
//!   - `mod.rs`        — re-exports and wires sub-modules
//!   - `domain.rs`     — pure Rust structs/enums: entities, value objects, domain events (no I/O)
//!   - `repository.rs` — sqlx queries; defines a Repository trait and a SqliteRepository impl
//!   - `ui.rs`         — iced view() function and domain-scoped Message enum variants
//!
//! ## Module Isolation Rules
//!   - Modules MUST NOT import from sibling modules directly.
//!   - Cross-module communication happens only via domain events defined in `src/kernel/`
//!     (to be created when the first cross-cutting concern appears).
//!   - The top-level `app.rs` is the ONLY place that composes module messages into app::Message.

pub mod project;
