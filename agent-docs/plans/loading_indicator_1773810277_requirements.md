# Requirements: Loading Indicator for DB Migrations

**Slug:** `loading_indicator_1773810277`

## Overview
Implement a modular, reusable loading indicator that displays while the core database migration task runs on startup. The indicator must persist for a minimum of 300ms to prevent UI flicker, even if the async DB init completes faster than that.

## Context
- Framework: iced 0.14 (Rust GUI)
- Pattern: Modulith / Domain-Driven Design
- Current behaviour: `app::view()` renders a plain `text("Initializing...")` while `App.core_db` is `None`
- Desired behaviour: a polished, animated/styled loading indicator that stays visible for ≥300ms

## Constraints
- **Tests first**: unit tests must be written before or alongside the implementation code
- **Modulith rules**: no cross-module direct imports; the loading indicator is a cross-cutting UI primitive → place in `src/kernel/loading/`
- **DDD**: pure domain logic (timing math) lives in `domain.rs`, no I/O or iced imports there
- **No new Cargo dependencies** unless absolutely necessary

---

## Deliverables

### 1. New module: `src/kernel/` (shared kernel namespace)
- Create `src/kernel/mod.rs` declaring `pub mod loading`

### 2. `src/kernel/loading/mod.rs`
- Re-exports `LoadingState`, `min_duration_elapsed`, and the `loading_indicator` view function
- Documents the module's purpose

### 3. `src/kernel/loading/domain.rs` — Pure domain logic (no iced, no I/O)
- **`LoadingState` enum**:
  - `Idle` — not yet started
  - `Loading { started_at: std::time::Instant }` — in progress, records when loading began
  - `Done` — loading complete and minimum duration satisfied
- **`MIN_LOADING_DURATION: Duration`** — constant set to 300ms
- **`min_duration_elapsed(started_at: Instant, now: Instant) -> bool`** — pure function; returns true if `now - started_at >= MIN_LOADING_DURATION`
- **Unit tests** (written first, in `#[cfg(test)]` block within this file):
  1. `test_elapsed_false_before_300ms` — `min_duration_elapsed` returns false when only 100ms has passed
  2. `test_elapsed_true_at_300ms` — returns true when exactly 300ms has passed
  3. `test_elapsed_true_after_300ms` — returns true when 500ms has passed
  4. `test_loading_state_starts_idle` — default `LoadingState` is `Idle`

### 4. `src/kernel/loading/ui.rs` — Reusable iced view component
- **`loading_indicator<'a, Message>(label: &str) -> Element<'a, Message>`**
  - Returns a centered iced layout containing a spinner/animation placeholder and the given label text
  - Generic over `Message` so it is reusable in any module without coupling to a specific message type
  - For the spinner: use iced's `text` widget with a Unicode braille/spinner character or a simple animated dot sequence (driven by `Tick` count stored in `App`), keeping it dependency-free
  - The exact visual style is intentionally simple for now (can be enhanced later)

### 5. Updates to `src/app.rs`
- Add two new fields to `App`:
  - `loading_state: kernel::loading::LoadingState`
  - `tick_count: u32` — increments on each `Tick` for spinner animation
- Add two new `Message` variants:
  - `Message::Tick` — emitted on a recurring timer (every ~100ms via `iced::time::every(...)`)
  - `Message::DbReady(CoreDb)` — already exists; update handler to record DB-ready time and check if 300ms has elapsed
- **Boot (`new()`)**: set `loading_state = LoadingState::Loading { started_at: Instant::now() }` and return a combined task: the existing DB init task **merged** with a recurring tick subscription
- **Update handlers**:
  - `Message::Tick`: increment `tick_count`; if `loading_state` is `Loading` and DB is ready and 300ms elapsed → transition to `LoadingState::Done`
  - `Message::DbReady`: store `core_db`; if 300ms already elapsed → set `LoadingState::Done`, else stay `Loading` (the Tick handler will eventually flip it)
  - `Message::DbFailed`: existing behaviour unchanged; set `loading_state = Done` (or a new `Failed` variant if preferred — keep it simple, use `Done`)
- **View (`view()`)**: replace `text("Initializing...")` with `kernel::loading::loading_indicator("Initialising database…")` when `loading_state != Done`

### 6. Wire up `src/main.rs`
- Add `mod kernel;` declaration

---

## Test Plan (tests-first order)

| # | Location | Test name | What it asserts |
|---|----------|-----------|-----------------|
| 1 | `kernel/loading/domain.rs` | `test_elapsed_false_before_300ms` | `min_duration_elapsed` is false at t+100ms |
| 2 | `kernel/loading/domain.rs` | `test_elapsed_true_at_300ms` | `min_duration_elapsed` is true at t+300ms |
| 3 | `kernel/loading/domain.rs` | `test_elapsed_true_after_300ms` | `min_duration_elapsed` is true at t+500ms |
| 4 | `kernel/loading/domain.rs` | `test_loading_state_starts_idle` | `LoadingState::default()` == `Idle` |

All tests must pass with `cargo test` before the implementation is considered complete.

---

## Out of Scope
- Animated SVG/Lottie spinners (add later)
- Per-module loading states (this covers app-level startup only)
- Error state visual differentiation (existing error text path is unchanged)
