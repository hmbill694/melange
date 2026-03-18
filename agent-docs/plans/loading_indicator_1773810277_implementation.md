# Implementation Plan: Loading Indicator for DB Migrations

**Slug:** `loading_indicator_1773810277`

---

## Architecture Decision

**Chosen approach (Hybrid Task + Conditional Subscription):**

- **Minimum-duration guarantee** — handled via a one-shot `Task` chain using `tokio::time::sleep`. This is polling-free and precisely timed.
- **Spinner animation** — driven by a conditional `Subscription` (`iced::time::every(100ms)`) that is only active while `loading_state` is `Loading`. Automatically stops when `LoadingDone` arrives.

This is idiomatic iced 0.14 and avoids any perpetual polling.

---

## Files to Create / Modify

| Action | Path |
|--------|------|
| CREATE | `src/kernel/mod.rs` |
| CREATE | `src/kernel/loading/mod.rs` |
| CREATE | `src/kernel/loading/domain.rs` |
| CREATE | `src/kernel/loading/ui.rs` |
| MODIFY | `src/app.rs` |
| MODIFY | `src/main.rs` |

---

## Ordered Implementation Checklist

> **Tests-first rule:** Each test step must be completed (compiling, asserting correctly) before the implementation step(s) it validates.

---

### Phase 1 — Scaffold the `kernel` namespace

- [ ] **Step 1.1 — Create `src/kernel/mod.rs`**

  Purpose: Establish the `kernel` namespace as the home for cross-cutting concerns, following the pattern described in `src/modules/mod.rs`.

  Content description:
  - Module-level doc comment explaining that `kernel` contains shared primitives used across bounded contexts. Modules must not import directly from `src/modules/`, and `kernel` types are the only sanctioned cross-module communication path.
  - Single declaration: `pub mod loading`

---

### Phase 2 — Domain logic (tests first)

- [ ] **Step 2.1 — Write unit tests in `src/kernel/loading/domain.rs` (tests-first)**

  Create the file with only the `#[cfg(test)]` block at this stage (the types they reference don't compile yet — that's intentional; tests specify the required interface).

  **Test 1: `test_elapsed_false_before_300ms`**
  - Arrange: let `start` be some `Instant`, let `now` be `start + 100ms`
  - Act: call `min_duration_elapsed(start, now)`
  - Assert: result is `false`
  - Why: 100ms < 300ms, so the minimum duration has not yet passed

  **Test 2: `test_elapsed_true_at_300ms`**
  - Arrange: let `start` be some `Instant`, let `now` be `start + 300ms`
  - Act: call `min_duration_elapsed(start, now)`
  - Assert: result is `true`
  - Why: exactly 300ms — boundary condition must be inclusive (≥ not >)

  **Test 3: `test_elapsed_true_after_300ms`**
  - Arrange: let `start` be some `Instant`, let `now` be `start + 500ms`
  - Act: call `min_duration_elapsed(start, now)`
  - Assert: result is `true`
  - Why: 500ms > 300ms, well past the minimum duration

  **Test 4: `test_loading_state_starts_idle`**
  - Arrange: call `LoadingState::default()`
  - Assert: the result equals `LoadingState::Idle`
  - Why: the app initialises in `Idle` before `new()` transitions it to `Loading`

  Implementation note for test setup: use `Instant::now()` in the test body, then add a `Duration` to produce `now`. No mocking required; these are deterministic arithmetic tests.

- [ ] **Step 2.2 — Implement `src/kernel/loading/domain.rs`**

  Purpose: Pure domain logic for loading state — no iced imports, no async, no I/O.

  **`MIN_LOADING_DURATION` constant**
  - Type: `std::time::Duration`
  - Value: 300 milliseconds
  - Visibility: `pub`
  - Why: single source of truth for the minimum display time

  **`LoadingState` enum**
  - Variants:
    - `Idle` — initial state, not yet started; this is the `Default`
    - `Loading { started_at: std::time::Instant }` — active, records when loading began
    - `Done` — minimum duration satisfied; triggers the view to show main UI
  - Derive: `Debug`, `Clone`, `PartialEq`
  - Implement `Default` → returns `Idle`
  - Why a struct variant for `Loading`: the `started_at` timestamp must be stored so that when `DbReady` arrives, we can compute exactly how much of the 300ms remains

  **`min_duration_elapsed(started_at: Instant, now: Instant) -> bool`**
  - Logic: `now.duration_since(started_at) >= MIN_LOADING_DURATION`
  - Return type: `bool`
  - Pure: no side effects, no I/O
  - Why a standalone function (not a method): keeps domain logic testable in isolation

  After this step, all 4 domain tests must pass with `cargo test`.

---

### Phase 3 — UI component

- [ ] **Step 3.1 — Create `src/kernel/loading/ui.rs`**

  Purpose: Reusable iced view component, generic over `Message`. No domain logic here — only layout.

  **Revised signature:** `loading_indicator<'a, Message>(label: &str, tick_count: u32) -> Element<'a, Message>`

  Imports needed: `iced::widget::{column, container, text}`, `iced::{Element, Fill, Alignment}`

  Layout description:
  - Outer: `container(...)` with `.center(Fill)` to fill the window and center content
  - Inner: a `column` with vertical centering alignment (`Alignment::Center`) and some spacing
  - Row 1: a `text(spinner_char)` where `spinner_char` is picked from a braille spinner array indexed by `tick_count % 8`
  - Row 2: `text(label)` — the label string passed in

  Spinner character array (8 frames): `['⣾', '⣽', '⣻', '⢿', '⡿', '⣟', '⣯', '⣷']`
  - Render spinner as `text(spinner_char).size(24)`

  Why generic over `Message`: this function has no knowledge of the application's message type, making it reusable in any future module without coupling.

  No unit tests for this step (iced view functions cannot be unit tested without a renderer).

---

### Phase 4 — Module re-exports

- [ ] **Step 4.1 — Create `src/kernel/loading/mod.rs`**

  Purpose: Clean public surface for the `loading` module.

  Content:
  - Module-level doc comment: "Cross-cutting loading indicator primitive. Provides `LoadingState` for timing logic and `loading_indicator` for the iced view."
  - `pub mod domain;`
  - `pub mod ui;`
  - Re-export flat surface:
    - `pub use domain::{LoadingState, MIN_LOADING_DURATION, min_duration_elapsed}`
    - `pub use ui::loading_indicator`

---

### Phase 5 — Wire `kernel` into `main.rs`

- [ ] **Step 5.1 — Add `mod kernel;` declaration to `src/main.rs`**

  Add `mod kernel;` alongside the existing `mod app;`, `mod db;`, `mod modules;` declarations.

  Why here: Rust module declarations must appear in a file that is an ancestor of the module's source. `main.rs` is the crate root.

---

### Phase 6 — Update `src/app.rs`

- [ ] **Step 6.1 — Add new imports to `src/app.rs`**

  New imports needed:
  - `use std::time::Instant` — for `started_at` timestamp
  - `use iced::time::Duration` — for the subscription interval
  - `use iced::Subscription` — for the subscription return type
  - `use crate::kernel::loading::{LoadingState, MIN_LOADING_DURATION, min_duration_elapsed, loading_indicator}` — kernel types and view fn

- [ ] **Step 6.2 — Extend `App` struct with two new fields**

  Add to the `App` struct:
  - `loading_state: LoadingState` — tracks the three-phase loading lifecycle
  - `tick_count: u32` — increments on every `Tick` for spinner animation frame selection

  Update `App::default()`:
  - `loading_state` → `LoadingState::Idle`
  - `tick_count` → `0`

- [ ] **Step 6.3 — Add new `Message` variants**

  Add to the `Message` enum:
  - `Message::Tick` — fired by the 100ms subscription; advances the spinner animation frame
  - `Message::LoadingDone` — fired by the one-shot Task after both DB is ready AND 300ms has elapsed; transitions `loading_state` to `Done`

  Keep existing `DbReady(CoreDb)` and `DbFailed(String)` variants unchanged.

- [ ] **Step 6.4 — Update `new()` boot function**

  Purpose: initialise `loading_state` to `Loading` at startup so the spinner is shown from the first frame.

  Logic:
  ```
  function new():
    state = App {
      loading_state: LoadingState::Loading { started_at: Instant::now() },
      tick_count: 0,
      core_db: None,
      init_error: None,
    }

    db_task = Task::perform(init_db(), |result| match result {
      Ok(core_db) => Message::DbReady(core_db),
      Err(e) => Message::DbFailed(e.to_string()),
    })

    return (state, db_task)
  ```

  Note: `loading_state` is set to `Loading` in `new()` (not `Idle`). `App::default()` still returns `Idle` but `new()` overrides it.

- [ ] **Step 6.5 — Update `update()` — handle `Message::DbReady`**

  Purpose: Store the DB, then chain a one-shot Task that waits for the remainder of the 300ms before firing `Message::LoadingDone`.

  Logic:
  ```
  Message::DbReady(core_db):
    state.core_db = Some(core_db)
    log info "Core database pool received"

    if state.loading_state is Loading { started_at }:
      return Task::perform(
        async move {
          let elapsed = started_at.elapsed()
          if elapsed < MIN_LOADING_DURATION {
            tokio::time::sleep(MIN_LOADING_DURATION - elapsed).await
          }
          // If already elapsed, return immediately
        },
        |_| Message::LoadingDone
      )
    else:
      return Task::none()
  ```

  Why `tokio::time::sleep` in an async block: simpler than `Task` chaining; `tokio` is already a dependency with `features = ["full"]`.

- [ ] **Step 6.6 — Update `update()` — handle `Message::DbFailed`**

  No change to error storage logic. Additionally:
  - Set `state.loading_state = LoadingState::Done`
  - Why: even on failure, the loading screen must stop; the error view renders instead

- [ ] **Step 6.7 — Update `update()` — handle `Message::Tick`**

  Logic:
  ```
  Message::Tick:
    state.tick_count = state.tick_count.wrapping_add(1)
    return Task::none()
  ```

  Why `wrapping_add`: prevents overflow panic; spinner modulo handles any value safely.

- [ ] **Step 6.8 — Update `update()` — handle `Message::LoadingDone`**

  Logic:
  ```
  Message::LoadingDone:
    state.loading_state = LoadingState::Done
    log info "Loading complete — minimum duration satisfied"
    return Task::none()
  ```

- [ ] **Step 6.9 — Add `subscription()` free function to `src/app.rs`**

  Signature: `pub fn subscription(state: &App) -> Subscription<Message>`

  Logic:
  ```
  function subscription(state: &App) -> Subscription<Message>:
    match state.loading_state:
      LoadingState::Loading { .. } =>
        return iced::time::every(Duration::from_millis(100))
               .map(|_instant| Message::Tick)
      _ =>
        return Subscription::none()
  ```

  Why conditional: returning `Subscription::none()` when done cancels the stream, avoiding ongoing work after loading completes.

- [ ] **Step 6.10 — Update `view()` to use `loading_indicator`**

  Replace the existing `text("Initializing...")` branch:

  Logic:
  ```
  function view(state: &App) -> Element<'_, Message>:
    if state.init_error is Some(err):
      return container(text(format!("Failed to initialize database: {}", err))).center(Fill)

    if state.loading_state != LoadingState::Done:
      return loading_indicator("Initialising database…", state.tick_count)

    // Ready state
    return container(column![text("Melange").size(32), text("Ready.")].spacing(10)).center(Fill)
  ```

  Note: The check is `!= Done` (not `core_db.is_none()`) — more precise. Even if the DB is ready, keep the loading indicator until `LoadingDone` has been received.

---

### Phase 7 — Final `main.rs` wiring

- [ ] **Step 7.1 — Add `.subscription(app::subscription)` to the iced builder in `src/main.rs`**

  Update the builder chain:
  ```
  Before:
    iced::application(app::new, app::update, app::view)
      .title("Melange")
      .run()

  After:
    iced::application(app::new, app::update, app::view)
      .title("Melange")
      .subscription(app::subscription)
      .run()
  ```

  Why: The subscription function must be registered with the iced runtime. Without this, `Tick` messages are never dispatched.

---

## Test Plan Summary (tests-first order)

| # | File | Test name | Validates |
|---|------|-----------|-----------|
| 1 | `src/kernel/loading/domain.rs` | `test_elapsed_false_before_300ms` | `min_duration_elapsed(start, start+100ms)` → `false` |
| 2 | `src/kernel/loading/domain.rs` | `test_elapsed_true_at_300ms` | `min_duration_elapsed(start, start+300ms)` → `true` (inclusive boundary) |
| 3 | `src/kernel/loading/domain.rs` | `test_elapsed_true_after_300ms` | `min_duration_elapsed(start, start+500ms)` → `true` |
| 4 | `src/kernel/loading/domain.rs` | `test_loading_state_starts_idle` | `LoadingState::default()` == `LoadingState::Idle` |

All 4 tests must pass via `cargo test` before the feature is considered complete.

---

## Key Invariants and Edge Cases

| Scenario | Expected behaviour |
|----------|-------------------|
| DB completes in 50ms | `LoadingDone` fires at exactly 250ms later (total 300ms from boot) |
| DB completes in 400ms | `LoadingDone` fires immediately after `DbReady` (0ms sleep, 400ms > 300ms) |
| DB fails | `DbFailed` sets `loading_state = Done`; error view shown immediately |
| `tick_count` overflows u32 | `wrapping_add` prevents panic; spinner continues from 0 |
| `loading_state` is `Idle` when `DbReady` arrives | Should not happen (boot sets `Loading`); `Task::none()` returned safely |

---

## Non-goals (out of scope per requirements)

- Animated SVG/Lottie spinners
- Per-module loading states
- Error state visual differentiation
- New Cargo dependencies
