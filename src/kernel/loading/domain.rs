use std::time::{Duration, Instant};

pub const MIN_LOADING_DURATION: Duration = Duration::from_millis(300);

#[derive(Debug, Clone, PartialEq)]
pub enum LoadingState {
    Idle,
    Loading { started_at: Instant },
    Done,
}

impl Default for LoadingState {
    fn default() -> Self {
        LoadingState::Idle
    }
}

/// Returns true if at least MIN_LOADING_DURATION has elapsed since `started_at`.
pub fn min_duration_elapsed(started_at: Instant, now: Instant) -> bool {
    now.duration_since(started_at) >= MIN_LOADING_DURATION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elapsed_false_before_300ms() {
        let start = Instant::now();
        let now = start + Duration::from_millis(100);
        assert!(!min_duration_elapsed(start, now));
    }

    #[test]
    fn test_elapsed_true_at_300ms() {
        let start = Instant::now();
        let now = start + Duration::from_millis(300);
        assert!(min_duration_elapsed(start, now));
    }

    #[test]
    fn test_elapsed_true_after_300ms() {
        let start = Instant::now();
        let now = start + Duration::from_millis(500);
        assert!(min_duration_elapsed(start, now));
    }

    #[test]
    fn test_loading_state_starts_idle() {
        assert_eq!(LoadingState::default(), LoadingState::Idle);
    }
}
