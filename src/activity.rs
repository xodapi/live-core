use core::time::Duration;

use crate::{Instant, Level, RollingBuffer, Threshold, ThresholdError};

/// Live activity state derived from aggregate token-rate telemetry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityState {
    /// No usable sample has been observed yet.
    Unknown,
    /// The latest zero-rate sample is older than the configured idle timeout.
    Idle,
    /// Recent telemetry indicates work or the idle grace window is still open.
    Active,
    /// Current rate reached the configured danger threshold.
    Hot,
}

/// Errors returned by activity telemetry constructors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityError {
    /// Token rate must be finite.
    NonFiniteRate,
    /// Token rate must not be negative.
    NegativeRate,
    /// Hot threshold must be valid.
    InvalidThreshold(ThresholdError),
}

/// Sanitized aggregate token-rate sample.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TokenRateSample {
    observed_at: Instant,
    tokens_per_second: f64,
}

impl TokenRateSample {
    /// Creates a validated aggregate token-rate sample.
    pub fn new(observed_at: Instant, tokens_per_second: f64) -> Result<Self, ActivityError> {
        if !tokens_per_second.is_finite() {
            return Err(ActivityError::NonFiniteRate);
        }

        if tokens_per_second < 0.0 {
            return Err(ActivityError::NegativeRate);
        }

        Ok(Self {
            observed_at,
            tokens_per_second,
        })
    }

    /// Returns the sample timestamp.
    pub const fn observed_at(self) -> Instant {
        self.observed_at
    }

    /// Returns the aggregate token rate.
    pub const fn tokens_per_second(self) -> f64 {
        self.tokens_per_second
    }
}

/// Stateless classifier for aggregate activity samples.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActivityClassifier {
    idle_timeout: Duration,
    hot_threshold: Threshold,
}

impl ActivityClassifier {
    /// Creates a classifier with an idle timeout and validated hot threshold.
    pub fn new(idle_timeout: Duration, hot_threshold: Threshold) -> Result<Self, ActivityError> {
        hot_threshold
            .validate()
            .map_err(ActivityError::InvalidThreshold)?;

        Ok(Self {
            idle_timeout,
            hot_threshold,
        })
    }

    /// Returns the configured idle timeout.
    pub const fn idle_timeout(&self) -> Duration {
        self.idle_timeout
    }

    /// Returns the configured hot threshold.
    pub const fn hot_threshold(&self) -> Threshold {
        self.hot_threshold
    }

    /// Classifies the latest sample at `now`.
    pub fn classify(&self, now: Instant, sample: Option<TokenRateSample>) -> ActivityState {
        let Some(sample) = sample else {
            return ActivityState::Unknown;
        };

        if self.hot_threshold.classify(sample.tokens_per_second) == Level::Danger {
            return ActivityState::Hot;
        }

        if sample.tokens_per_second == 0.0
            && now.duration_since(sample.observed_at) >= self.idle_timeout
        {
            return ActivityState::Idle;
        }

        ActivityState::Active
    }
}

/// Small stateful helper that records samples and exposes current activity.
#[derive(Clone, Debug, PartialEq)]
pub struct ActivityTracker<const N: usize> {
    classifier: ActivityClassifier,
    history: RollingBuffer<f64, N>,
    latest: Option<TokenRateSample>,
}

impl<const N: usize> ActivityTracker<N> {
    /// Creates an empty tracker.
    pub fn new(classifier: ActivityClassifier) -> Self {
        Self {
            classifier,
            history: RollingBuffer::new(),
            latest: None,
        }
    }

    /// Records a validated sample.
    pub fn record(&mut self, sample: TokenRateSample) {
        self.history.push(sample.tokens_per_second);
        self.latest = Some(sample);
    }

    /// Returns the current state at `now`.
    pub fn state(&self, now: Instant) -> ActivityState {
        self.classifier.classify(now, self.latest)
    }

    /// Returns the latest sample.
    pub const fn latest(&self) -> Option<TokenRateSample> {
        self.latest
    }

    /// Returns the recorded rate history.
    pub const fn history(&self) -> &RollingBuffer<f64, N> {
        &self.history
    }

    /// Returns the mean recorded aggregate token rate.
    pub fn mean_rate(&self) -> Option<f64> {
        self.history.mean()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn classifier() -> ActivityClassifier {
        ActivityClassifier::new(
            Duration::from_secs(30),
            Threshold::new(50.0, 100.0).expect("valid threshold"),
        )
        .expect("valid classifier")
    }

    fn sample(at_nanos: u128, rate: f64) -> TokenRateSample {
        TokenRateSample::new(Instant::from_nanos(at_nanos), rate).expect("valid sample")
    }

    #[test]
    fn missing_sample_is_unknown() {
        let classifier = classifier();

        assert_eq!(
            classifier.classify(Instant::from_nanos(0), None),
            ActivityState::Unknown
        );
    }

    #[test]
    fn zero_rate_older_than_timeout_is_idle() {
        let classifier = classifier();
        let now = Instant::from_nanos(Duration::from_secs(31).as_nanos());

        assert_eq!(
            classifier.classify(now, Some(sample(0, 0.0))),
            ActivityState::Idle
        );
    }

    #[test]
    fn zero_rate_inside_timeout_remains_active() {
        let classifier = classifier();
        let now = Instant::from_nanos(Duration::from_secs(5).as_nanos());

        assert_eq!(
            classifier.classify(now, Some(sample(0, 0.0))),
            ActivityState::Active
        );
    }

    #[test]
    fn positive_rate_is_active() {
        let classifier = classifier();

        assert_eq!(
            classifier.classify(Instant::from_nanos(10), Some(sample(0, 12.5))),
            ActivityState::Active
        );
    }

    #[test]
    fn danger_rate_is_hot() {
        let classifier = classifier();

        assert_eq!(
            classifier.classify(Instant::from_nanos(10), Some(sample(0, 100.0))),
            ActivityState::Hot
        );
    }

    #[test]
    fn rejects_invalid_samples() {
        assert_eq!(
            TokenRateSample::new(Instant::ZERO, f64::NAN),
            Err(ActivityError::NonFiniteRate)
        );
        assert_eq!(
            TokenRateSample::new(Instant::ZERO, f64::INFINITY),
            Err(ActivityError::NonFiniteRate)
        );
        assert_eq!(
            TokenRateSample::new(Instant::ZERO, -1.0),
            Err(ActivityError::NegativeRate)
        );
    }

    #[test]
    fn rejects_invalid_hot_threshold() {
        let invalid = Threshold {
            warning: 10.0,
            danger: 10.0,
        };

        assert_eq!(
            ActivityClassifier::new(Duration::from_secs(30), invalid),
            Err(ActivityError::InvalidThreshold(
                ThresholdError::InvalidOrder
            ))
        );
    }

    #[test]
    fn tracker_records_history_and_state() {
        let classifier = classifier();
        let mut tracker = ActivityTracker::<3>::new(classifier);

        tracker.record(sample(0, 10.0));
        tracker.record(sample(1, 20.0));
        tracker.record(sample(2, 30.0));
        tracker.record(sample(3, 40.0));

        assert_eq!(tracker.state(Instant::from_nanos(4)), ActivityState::Active);
        assert_eq!(tracker.latest(), Some(sample(3, 40.0)));
        assert_eq!(tracker.mean_rate(), Some(30.0));
        assert_eq!(
            tracker
                .history()
                .iter()
                .copied()
                .collect::<std::vec::Vec<_>>(),
            [20.0, 30.0, 40.0]
        );
    }
}
