# Implementation Plan: Opencode Startup Check

**Slug:** `opencode_startup_check_1773811148`

---

## Summary of Approach

- New `src/kernel/opencode/` module following the exact same three-file pattern as `src/kernel/loading/`
- The PATH check uses `std::process::Command` (no new deps) inside a `tokio::task::spawn_blocking` async wrapper
- Opening the install URL: the "not installed" screen displays the URL as plain selectable text for the user to copy/paste (no browser automation needed)
- The blocked screen is a hard block: `view()` checks opencode status first, before loading or ready UI
- Two new `Message` variants, one new `App` field, one new parallel `Task` in `new()`

---

## File: `src/kernel/opencode/domain.rs`

**Purpose:** Pure types and the async-compatible check function. No I/O side-effects in types.

```
/// Represents the result of the startup opencode binary check.
enum OpencodeStatus {
    Found,
    NotFound,
}
derives: Debug, Clone, PartialEq

/// Async function that checks whether `opencode` is on PATH.
/// Runs the check in a blocking thread to avoid blocking the async executor.
///
/// Strategy: spawn a blocking closure via tokio::task::spawn_blocking that calls
///   std::process::Command::new("opencode").arg("--version").output()
/// If output() returns Ok(_) → Found (binary exists and ran)
/// If output() returns Err(e) where e.kind() == ErrorKind::NotFound → NotFound
/// Any other Err (e.g. permission denied) → treat as Found (binary exists, just failed)
///
/// Returns: OpencodeStatus (never fails — errors are mapped to a status)
async function check_opencode_on_path() -> OpencodeStatus:
    result = spawn_blocking(|| Command::new("opencode").arg("--version").output())
             .await (unwrap the JoinError)
    match result:
        Ok(_output) → OpencodeStatus::Found
        Err(io_error) where kind == ErrorKind::NotFound → OpencodeStatus::NotFound
        Err(_other) → OpencodeStatus::Found  // present but misbehaving
```

**Unit tests (3):**
1. `test_status_found_variant` — construct `OpencodeStatus::Found`, assert it is the `Found` variant
2. `test_status_not_found_variant` — construct `OpencodeStatus::NotFound`, assert it is the `NotFound` variant
3. `test_opencode_status_derives` — verify `Debug` + `Clone` impls compile (just use `format!("{:?}", ...)`)

---

## File: `src/kernel/opencode/ui.rs`

**Purpose:** Reusable iced view component for the "opencode not installed" blocked screen. Generic over `Message` (same pattern as `loading_indicator`).

```
constant: OPENCODE_INSTALL_URL = "https://opencode.ai"

/// Renders a centered, full-screen "opencode not installed" block screen.
/// Generic over Message so it can be embedded in any module's view without coupling.
///
/// Signature:
///   pub fn opencode_not_found_screen<'a, Message>() -> Element<'a, Message>
///   where Message: 'a
///
/// Layout:
///   container(
///     column![
///       text("opencode is required").size(28),
///       text("Melange requires opencode to be installed and available on your PATH."),
///       text("To install, visit:"),
///       text(OPENCODE_INSTALL_URL).size(16),
///     ]
///     .spacing(12)
///     .align_x(Center)
///   )
///   .center(Fill)
///   .into()
```

- No button, no `on_press`, no Message variants needed in this file
- The URL is rendered as plain `text(...)` so the user can select and copy it
- The function is generic over `Message` (same as `loading_indicator`) for zero coupling to `app.rs`

---

## File: `src/kernel/opencode/mod.rs`

**Purpose:** Re-exports the flat public API, mirroring `src/kernel/loading/mod.rs`.

```
//! Opencode startup check primitive.
//!
//! Provides `OpencodeStatus` for check logic and `opencode_not_found_screen` for the iced view.

pub mod domain;
pub mod ui;

pub use domain::{check_opencode_on_path, OpencodeStatus};
pub use ui::opencode_not_found_screen;
```

---

## Modify: `src/kernel/mod.rs`

Add `pub mod opencode;` declaration alongside the existing `pub mod loading;`.

---

## Modify: `src/app.rs`

### 1. New import

Add to the use block:
```
use crate::kernel::opencode::{check_opencode_on_path, opencode_not_found_screen, OpencodeStatus};
```

### 2. New `App` field

Add to the `App` struct:
```
/// Holds the result of the startup opencode binary check.
/// `None` means the check has not yet completed.
opencode_status: Option<OpencodeStatus>,
```

Update `App::default()` to set `opencode_status: None`.

### 3. New `Message` variants

Add to the `Message` enum:
```
/// Sent when the startup opencode check confirms the binary is on PATH.
OpencodeReady,

/// Sent when the startup opencode check finds the binary is NOT on PATH.
OpencodeNotFound,
```

### 4. `new()` — fire both tasks concurrently

Modify `new()` to produce two tasks batched via `Task::batch`:

```
function new() -> (App, Task<Message>):
    initial_state = App {
        loading_state: LoadingState::Loading { started_at: Instant::now() },
        tick_count: 0,
        core_db: None,
        init_error: None,
        opencode_status: None,
    }

    db_task = Task::perform(
        async { init_db().await },
        |result| match result {
            Ok(db) → Message::DbReady(db),
            Err(e) → Message::DbFailed(e.to_string()),
        }
    )

    opencode_task = Task::perform(
        async { check_opencode_on_path().await },
        |status| match status {
            OpencodeStatus::Found    → Message::OpencodeReady,
            OpencodeStatus::NotFound → Message::OpencodeNotFound,
        }
    )

    return (initial_state, Task::batch([db_task, opencode_task]))
```

### 5. `update()` — handle new variants

Add two new arms to the `match message`:

```
Message::OpencodeReady:
    log info: "opencode binary found on PATH — proceeding normally"
    state.opencode_status = Some(OpencodeStatus::Found)
    return Task::none()

Message::OpencodeNotFound:
    log warning: "opencode binary NOT found on PATH — showing blocked screen"
    state.opencode_status = Some(OpencodeStatus::NotFound)
    return Task::none()
```

### 6. `view()` — routing priority

Replace the existing `view()` with a four-priority chain:

```
function view(state: &App) -> Element<'_, Message>:

    // Priority 1: opencode not found → hard block (highest priority)
    if state.opencode_status == Some(OpencodeStatus::NotFound):
        return opencode_not_found_screen()

    // Priority 2: existing init_error check (DB failure)
    if state.init_error is Some(ref err):
        return container(text("Failed to initialize database: {err}")).center(Fill)

    // Priority 3: still loading OR opencode check not yet resolved
    if state.loading_state != LoadingState::Done
       OR state.opencode_status is None:
        return loading_indicator("Initialising…", state.tick_count)

    // Priority 4: ready — main UI
    return container(column![text("Melange").size(32), text("Ready.")].spacing(10))
               .center(Fill)
```

**Key design note on Priority 3:** The loading spinner is shown while `opencode_status` is still `None` (check not yet complete). Once the check resolves to `Found`, the normal loading→done flow continues. If it resolves to `NotFound`, Priority 1 takes over immediately.

---

## Implementation Checklist

- [ ] **Step 1** — Create `src/kernel/opencode/domain.rs`: `OpencodeStatus` enum + `check_opencode_on_path()` async fn + 3 unit tests
- [ ] **Step 2** — Create `src/kernel/opencode/ui.rs`: `opencode_not_found_screen()` generic view fn
- [ ] **Step 3** — Create `src/kernel/opencode/mod.rs`: module declarations + re-exports
- [ ] **Step 4** — Modify `src/kernel/mod.rs`: add `pub mod opencode;`
- [ ] **Step 5** — Modify `src/app.rs` — imports: add opencode kernel import
- [ ] **Step 6** — Modify `src/app.rs` — `App` struct: add `opencode_status: Option<OpencodeStatus>` field + update default/init
- [ ] **Step 7** — Modify `src/app.rs` — `Message` enum: add `OpencodeReady` and `OpencodeNotFound` variants
- [ ] **Step 8** — Modify `src/app.rs` — `new()`: create `opencode_task`, wrap both tasks in `Task::batch`
- [ ] **Step 9** — Modify `src/app.rs` — `update()`: add handlers for both new message variants
- [ ] **Step 10** — Modify `src/app.rs` — `view()`: restructure into four-priority chain
