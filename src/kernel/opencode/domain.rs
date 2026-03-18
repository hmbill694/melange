use std::io::ErrorKind;
use std::process::Command;

/// Represents the result of the startup opencode binary check.
#[derive(Debug, Clone, PartialEq)]
pub enum OpencodeStatus {
    /// The `opencode` binary was found on PATH (or at least attempted to run).
    Found,
    /// The `opencode` binary was NOT found on PATH.
    NotFound,
}

/// Async function that checks whether `opencode` is on PATH.
///
/// Runs the check in a blocking thread to avoid blocking the async executor.
///
/// Strategy: spawns a blocking closure via `tokio::task::spawn_blocking` that calls
/// `std::process::Command::new("opencode").arg("--version").output()`.
/// - If `output()` returns `Ok(_)` → `Found` (binary exists and ran)
/// - If `output()` returns `Err(e)` where `e.kind() == ErrorKind::NotFound` → `NotFound`
/// - Any other `Err` (e.g. permission denied) → treat as `Found` (binary exists, just failed)
///
/// Returns `OpencodeStatus` (never fails — errors are mapped to a status).
pub async fn check_opencode_on_path() -> OpencodeStatus {
    let result = tokio::task::spawn_blocking(|| {
        Command::new("opencode").arg("--version").output()
    })
    .await
    .expect("spawn_blocking task panicked");

    match result {
        Ok(_output) => OpencodeStatus::Found,
        Err(io_error) if io_error.kind() == ErrorKind::NotFound => OpencodeStatus::NotFound,
        Err(_other) => OpencodeStatus::Found, // present but misbehaving
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_found_variant() {
        let status = OpencodeStatus::Found;
        assert_eq!(status, OpencodeStatus::Found);
    }

    #[test]
    fn test_status_not_found_variant() {
        let status = OpencodeStatus::NotFound;
        assert_eq!(status, OpencodeStatus::NotFound);
    }

    #[test]
    fn test_opencode_status_derives() {
        let found = OpencodeStatus::Found;
        let cloned = found.clone();
        assert_eq!(format!("{:?}", cloned), "Found");

        let not_found = OpencodeStatus::NotFound;
        let cloned_nf = not_found.clone();
        assert_eq!(format!("{:?}", cloned_nf), "NotFound");
    }
}
