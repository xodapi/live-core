#![cfg_attr(not(feature = "std"), no_std)]

//! Bootstrap crate for live autonomous-agent telemetry primitives.
//!
//! The first real modules are planned as `clock`, `rolling_buffer`, and
//! `threshold`. This crate starts intentionally small so repository setup,
//! governance, and CI can land before module work begins.

pub mod activity;
pub mod clock;
pub mod rolling_buffer;
pub mod threshold;

pub use activity::{
    ActivityClassifier, ActivityError, ActivityState, ActivityTracker, TokenRateSample,
};
pub use clock::{Clock, Instant};
pub use rolling_buffer::{Iter, RollingBuffer};
pub use threshold::{Level, Threshold, ThresholdError};

#[cfg(feature = "std")]
pub use clock::StdClock;

/// Human-readable crate status for smoke tests and early integrations.
pub const BOOTSTRAP_STATUS: &str = "live-core bootstrap";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_bootstrap_status() {
        assert_eq!(BOOTSTRAP_STATUS, "live-core bootstrap");
    }
}
