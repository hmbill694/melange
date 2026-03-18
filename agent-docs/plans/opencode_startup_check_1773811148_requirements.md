# Requirements: Opencode Startup Check

## Feature Slug
`opencode_startup_check_1773811148`

## Summary
On every app startup, the application must verify that the `opencode` binary is accessible on the system PATH. If it is not found, the user is presented with a blocking screen that explains opencode is required and provides a button to open the official install page in their browser. The main application UI must not be shown until opencode is confirmed present.

## Detailed Requirements

### 1. Startup Check
- On every app launch, run an async check to determine whether the `opencode` binary exists on the system PATH.
- The check must complete before the main UI is rendered.
- The check should run concurrently with the existing DB initialization task (both fire from `app::new()`).

### 2. Not Found — Blocked State
- If `opencode` is not found on PATH:
  - The main UI is **never shown**.
  - A dedicated "opencode not installed" screen is rendered.
  - The screen must include:
    - A clear heading explaining opencode is required.
    - A short explanatory message.
    - A button labeled something like "Install opencode" that opens the official install URL in the user's default browser.
  - No dismiss/skip/continue option is provided. The screen is a hard block.

### 3. Found — Normal Flow
- If `opencode` is found on PATH, startup proceeds normally with no additional UI or messages.
- The DB init + loading sequence continues as before.

### 4. Module Structure
- The new logic lives in `src/kernel/opencode/` following the existing modulith layout:
  - `domain.rs` — pure Rust types: `OpencodeStatus` enum (`Found`, `NotFound`), check logic (no I/O side-effects in types)
  - `ui.rs` — iced view function rendering the "not installed" screen
  - `mod.rs` — re-exports the public API
- Kernel modules must not import from `src/modules/`.

### 5. App Integration (`src/app.rs`)
- Add two new `Message` variants:
  - `OpencodeReady` — opencode was found; proceed normally
  - `OpencodeNotFound` — opencode is missing; switch to blocked state
- Add an `opencode_status: Option<OpencodeStatus>` field (or equivalent) to `App` state to track check result.
- The `new()` boot function fires a `Task::perform` for the opencode check alongside the existing DB task.
- The `update()` function handles both new message variants.
- The `view()` function renders the "not installed" screen when status is `NotFound`, taking priority over loading/ready UI.

### 6. Install URL
- The button opens: `https://opencode.ai` (or the official install/docs page — use `https://opencode.ai` as the target).
- Use `iced::open_url` or the appropriate iced API to open the URL in the default browser.

### 7. No New Dependencies
- The PATH check uses Rust's standard library only (`std::process::Command` with `which`/`where` or checking `PATH` directly), or the existing `which` crate if already present — no new Cargo dependencies required.
- Opening a URL uses the iced built-in mechanism (no additional shell-exec crate needed).

## Out of Scope
- Re-checking after the app is open (no polling).
- Auto-installing opencode from within the app.
- Version checking (just presence check).
