use core::time::Duration;

use crate::{Clock, Instant};

/// Idle/offline state derived from the last observed activity timestamp.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdleState {
    /// No activity has been observed yet.
    NeverSeen,
    /// Activity was observed within the configured timeout.
    Active,
    /// The configured idle timeout elapsed since the last observed activity.
    Idle,
}

/// Small deterministic idle tracker.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IdleTracker {
    idle_timeout: Duration,
    last_seen: Option<Instant>,
}

impl IdleTracker {
    /// Creates an idle tracker with no observed activity.
    pub const fn new(idle_timeout: Duration) -> Self {
        Self {
            idle_timeout,
            last_seen: None,
        }
    }

    /// Returns the configured idle timeout.
    pub const fn idle_timeout(&self) -> Duration {
        self.idle_timeout
    }

    /// Returns the last observed activity timestamp.
    pub const fn last_seen(&self) -> Option<Instant> {
        self.last_seen
    }

    /// Records activity at a specific instant.
    pub fn observe(&mut self, at: Instant) {
        self.last_seen = Some(at);
    }

    /// Records activity using a clock.
    pub fn observe_now<C: Clock>(&mut self, clock: &C) {
        self.observe(clock.now());
    }

    /// Clears the tracker back to `NeverSeen`.
    pub fn reset(&mut self) {
        self.last_seen = None;
    }

    /// Returns the current idle state at `now`.
    pub fn state(&self, now: Instant) -> IdleState {
        let Some(last_seen) = self.last_seen else {
            return IdleState::NeverSeen;
        };

        if now.duration_since(last_seen) >= self.idle_timeout {
            IdleState::Idle
        } else {
            IdleState::Active
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy)]
    struct FakeClock {
        now: Instant,
    }

    impl Clock for FakeClock {
        fn now(&self) -> Instant {
            self.now
        }
    }

    fn tracker() -> IdleTracker {
        IdleTracker::new(Duration::from_secs(30))
    }

    #[test]
    fn starts_as_never_seen() {
        let tracker = tracker();

        assert_eq!(tracker.state(Instant::ZERO), IdleState::NeverSeen);
        assert_eq!(tracker.last_seen(), None);
    }

    #[test]
    fn observed_activity_is_active_before_timeout() {
        let mut tracker = tracker();

        tracker.observe(Instant::ZERO);

        assert_eq!(
            tracker.state(Instant::from_nanos(Duration::from_secs(29).as_nanos())),
            IdleState::Active
        );
    }

    #[test]
    fn timeout_boundary_is_idle() {
        let mut tracker = tracker();

        tracker.observe(Instant::ZERO);

        assert_eq!(
            tracker.state(Instant::from_nanos(Duration::from_secs(30).as_nanos())),
            IdleState::Idle
        );
    }

    #[test]
    fn observe_now_uses_clock() {
        let clock = FakeClock {
            now: Instant::from_nanos(42),
        };
        let mut tracker = tracker();

        tracker.observe_now(&clock);

        assert_eq!(tracker.last_seen(), Some(Instant::from_nanos(42)));
    }

    #[test]
    fn reset_returns_to_never_seen() {
        let mut tracker = tracker();

        tracker.observe(Instant::ZERO);
        tracker.reset();

        assert_eq!(tracker.state(Instant::ZERO), IdleState::NeverSeen);
    }
}
