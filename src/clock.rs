use core::time::Duration;

const NANOS_PER_SECOND: u128 = 1_000_000_000;

/// Source of monotonic time for telemetry primitives.
pub trait Clock {
    /// Returns the current instant.
    fn now(&self) -> Instant;
}

/// Monotonic timestamp represented as nanoseconds from an arbitrary origin.
///
/// This type is intentionally not a re-export of `std::time::Instant`, so it
/// can be constructed deterministically in tests and used in `no_std` builds.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Instant {
    nanos: u128,
}

impl Instant {
    /// The zero instant for deterministic tests and synthetic clocks.
    pub const ZERO: Self = Self::from_nanos(0);

    /// Creates an instant from nanoseconds since an arbitrary origin.
    pub const fn from_nanos(nanos: u128) -> Self {
        Self { nanos }
    }

    /// Returns nanoseconds since the instant origin.
    pub const fn as_nanos(self) -> u128 {
        self.nanos
    }

    /// Returns the duration from `earlier` to `self`.
    ///
    /// If `earlier` is after `self`, the result saturates to zero.
    pub fn duration_since(self, earlier: Instant) -> Duration {
        duration_from_nanos(self.nanos.saturating_sub(earlier.nanos))
    }
}

#[cfg(feature = "std")]
/// Monotonic process-local clock backed by `std::time::Instant`.
#[derive(Clone, Copy, Debug, Default)]
pub struct StdClock;

#[cfg(feature = "std")]
impl Clock for StdClock {
    fn now(&self) -> Instant {
        use std::sync::OnceLock;

        static START: OnceLock<std::time::Instant> = OnceLock::new();
        let start = START.get_or_init(std::time::Instant::now);
        Instant::from_nanos(start.elapsed().as_nanos())
    }
}

fn duration_from_nanos(nanos: u128) -> Duration {
    let secs = nanos / NANOS_PER_SECOND;
    let subsec_nanos = nanos % NANOS_PER_SECOND;

    if secs > u64::MAX as u128 {
        Duration::new(u64::MAX, 999_999_999)
    } else {
        Duration::new(secs as u64, subsec_nanos as u32)
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

    #[test]
    fn duration_since_equal_timestamps_is_zero() {
        let instant = Instant::from_nanos(42);

        assert_eq!(instant.duration_since(instant), Duration::ZERO);
    }

    #[test]
    fn duration_since_positive_interval() {
        let earlier = Instant::from_nanos(1_000);
        let later = Instant::from_nanos(2_500_000_001);

        assert_eq!(later.duration_since(earlier), Duration::new(2, 499_999_001));
    }

    #[test]
    fn duration_since_saturates_when_earlier_is_later() {
        let earlier = Instant::from_nanos(10);
        let later = Instant::from_nanos(5);

        assert_eq!(later.duration_since(earlier), Duration::ZERO);
    }

    #[test]
    fn fake_clock_returns_deterministic_instant() {
        let clock = FakeClock {
            now: Instant::from_nanos(123),
        };

        assert_eq!(clock.now(), Instant::from_nanos(123));
    }

    #[cfg(feature = "std")]
    #[test]
    fn std_clock_is_monotonic() {
        let clock = StdClock;

        assert!(clock.now() <= clock.now());
    }
}
