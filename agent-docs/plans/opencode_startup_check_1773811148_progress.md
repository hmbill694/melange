# Progress: opencode_startup_check_1773811148

- [x] Step 1: Create `src/kernel/opencode/domain.rs` — `OpencodeStatus` enum + `check_opencode_on_path()` async fn + 3 unit tests
- [x] Step 2: Create `src/kernel/opencode/ui.rs` — `opencode_not_found_screen()` generic view fn
- [x] Step 3: Create `src/kernel/opencode/mod.rs` — module declarations + re-exports
- [x] Step 4: Modify `src/kernel/mod.rs` — add `pub mod opencode;`
- [x] Step 5: Modify `src/app.rs` — imports: add opencode kernel import
- [x] Step 6: Modify `src/app.rs` — `App` struct: add `opencode_status: Option<OpencodeStatus>` field + update default/init
- [x] Step 7: Modify `src/app.rs` — `Message` enum: add `OpencodeReady` and `OpencodeNotFound` variants
- [x] Step 8: Modify `src/app.rs` — `new()`: create `opencode_task`, wrap both tasks in `Task::batch`
- [x] Step 9: Modify `src/app.rs` — `update()`: add handlers for both new message variants
- [x] Step 10: Modify `src/app.rs` — `view()`: restructure into four-priority chain
