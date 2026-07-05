/// Classified metric level.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Level {
    /// Metric is below the warning threshold.
    Ok,
    /// Metric reached the warning threshold but not the danger threshold.
    Warning,
    /// Metric reached the danger threshold or could not be classified safely.
    Danger,
}

/// Errors returned when creating or validating a threshold.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThresholdError {
    /// Both threshold bounds must be finite numbers.
    NonFiniteBound,
    /// The warning bound must be strictly lower than the danger bound.
    InvalidOrder,
}

/// Warning and danger bounds for a numeric metric.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Threshold {
    pub warning: f64,
    pub danger: f64,
}

impl Threshold {
    /// Creates a validated threshold.
    pub fn new(warning: f64, danger: f64) -> Result<Self, ThresholdError> {
        let threshold = Self { warning, danger };
        threshold.validate()?;
        Ok(threshold)
    }

    /// Validates the threshold bounds.
    pub fn validate(&self) -> Result<(), ThresholdError> {
        if !self.warning.is_finite() || !self.danger.is_finite() {
            return Err(ThresholdError::NonFiniteBound);
        }

        if self.warning >= self.danger {
            return Err(ThresholdError::InvalidOrder);
        }

        Ok(())
    }

    /// Classifies a value against this threshold.
    ///
    /// `NaN` is classified as [`Level::Danger`] because it means the metric
    /// cannot be trusted. Positive infinity is also danger, while negative
    /// infinity is below any finite warning threshold and therefore ok.
    ///
    /// # Panics
    ///
    /// Panics if the threshold bounds are invalid. Use [`Threshold::new`] to
    /// validate bounds at construction time.
    pub fn classify(&self, value: f64) -> Level {
        self.validate()
            .expect("threshold bounds must be finite and warning < danger");

        if value.is_nan() || value >= self.danger {
            Level::Danger
        } else if value >= self.warning {
            Level::Warning
        } else {
            Level::Ok
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn threshold() -> Threshold {
        Threshold::new(10.0, 20.0).expect("valid threshold")
    }

    #[test]
    fn validates_warning_less_than_danger() {
        assert_eq!(
            Threshold::new(10.0, 20.0),
            Ok(Threshold {
                warning: 10.0,
                danger: 20.0
            })
        );
        assert_eq!(
            Threshold::new(10.0, 10.0),
            Err(ThresholdError::InvalidOrder)
        );
        assert_eq!(
            Threshold::new(20.0, 10.0),
            Err(ThresholdError::InvalidOrder)
        );
    }

    #[test]
    fn rejects_non_finite_bounds() {
        assert_eq!(
            Threshold::new(f64::NAN, 20.0),
            Err(ThresholdError::NonFiniteBound)
        );
        assert_eq!(
            Threshold::new(10.0, f64::INFINITY),
            Err(ThresholdError::NonFiniteBound)
        );
        assert_eq!(
            Threshold::new(f64::NEG_INFINITY, 20.0),
            Err(ThresholdError::NonFiniteBound)
        );
    }

    #[test]
    fn classifies_boundary_values() {
        let threshold = threshold();

        assert_eq!(threshold.classify(9.99), Level::Ok);
        assert_eq!(threshold.classify(10.0), Level::Warning);
        assert_eq!(threshold.classify(15.0), Level::Warning);
        assert_eq!(threshold.classify(20.0), Level::Danger);
        assert_eq!(threshold.classify(20.01), Level::Danger);
    }

    #[test]
    fn classifies_nan_and_infinity_explicitly() {
        let threshold = threshold();

        assert_eq!(threshold.classify(f64::NAN), Level::Danger);
        assert_eq!(threshold.classify(f64::INFINITY), Level::Danger);
        assert_eq!(threshold.classify(f64::NEG_INFINITY), Level::Ok);
    }

    #[test]
    #[should_panic(expected = "threshold bounds must be finite and warning < danger")]
    fn classify_panics_for_invalid_literal_threshold() {
        let threshold = Threshold {
            warning: 20.0,
            danger: 10.0,
        };

        let _ = threshold.classify(15.0);
    }
}
