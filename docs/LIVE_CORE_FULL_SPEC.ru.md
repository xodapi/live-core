# live-core — full specification

**Status:** canonical v1 specification after bootstrap.

`live-core` is a small Rust library for live autonomous-agent telemetry
primitives. It is intentionally UI-free, network-free, dependency-light, and
prepared for `no_std` where practical.

The crate currently provides:

- `clock`: deterministic time abstraction.
- `rolling_buffer`: fixed-size metric history.
- `threshold`: pure level classification.
- `activity`: aggregate token-rate activity classification.

---

## 1. Design goals

- Keep core telemetry logic testable without wall-clock sleeps.
- Share the same primitives across CLI, TUI, desktop GUI, Android, and future
  agent supervisors.
- Avoid storing sensitive payloads. Core modules work with numeric rates,
  timestamps, durations, and validated thresholds.
- Keep the first version small enough for multi-agent work without conflicts.
- Preserve a clean `no_std` path for embedded or constrained runtimes.

---

## 2. Non-goals

`live-core` v1 does not include:

- API clients or network polling.
- UI rendering, Android glue, desktop notifications, or vibration handling.
- Provider-specific token schemas.
- Prompt, transcript, task-title, or payload storage.
- Rig, jj/git-cliff, Bevy/3D, iOS, or other research tracks.
- percentile, median, EMA/ExponentialSmoother, serde, or smoothing plugins.

These can be revisited only through focused future issues.

---

## 3. Current modules

### 3.1 `clock`

Purpose: separate time from `std::time::Instant` so tests and downstream
modules can use deterministic timestamps.

Public surface:

- `Clock`: trait with `now() -> Instant`.
- `Instant`: nanoseconds from an arbitrary monotonic origin.
- `StdClock`: process-local monotonic clock behind the `std` feature.

Important behavior:

- `Instant` is a newtype, not a public re-export of `std::time::Instant`.
- `Instant::ZERO`, `from_nanos`, and `as_nanos` support deterministic tests.
- `duration_since` saturates to zero when the earlier instant is after self.
- `StdClock` is not required in `no_std` builds.

### 3.2 `rolling_buffer`

Purpose: keep a fixed-size history of recent metric values without heap
allocation.

Public surface:

- `RollingBuffer<T, const N: usize>`.
- `push(value)`.
- `len()`, `is_empty()`, `capacity()`.
- `iter()` from oldest to newest.
- `mean()` for `RollingBuffer<f64, N>`.

Important behavior:

- The buffer uses fixed storage and does not require heap allocation.
- Overflow evicts the oldest values.
- `N = 0` is a documented no-op buffer: pushed values are dropped, length
  stays zero, and iteration is empty.
- Empty buffers return `None` for `mean()`.

### 3.3 `threshold`

Purpose: classify numeric metrics into stable levels without UI or API
coupling.

Public surface:

- `Level::{Ok, Warning, Danger}`.
- `Threshold { warning, danger }`.
- `ThresholdError::{NonFiniteBound, InvalidOrder}`.
- `Threshold::new`, `validate`, and `classify`.

Important behavior:

- Bounds must be finite.
- `warning < danger` is validated.
- `NaN` values classify as `Danger`.
- Positive infinity classifies as `Danger`.
- Negative infinity classifies as `Ok` for finite thresholds.

---

## 4. Activity module

The activity module turns sanitized aggregate token-rate samples into live
states without UI, Android, network, or provider coupling.

Public concepts:

- `ActivityState::{Unknown, Idle, Active, Hot}`.
- `TokenRateSample`.
- `ActivityClassifier`.
- `ActivityTracker<N>`.

Important behavior:

- Missing sample means `Unknown`.
- Zero-rate sample older than idle timeout means `Idle`.
- Recent positive rate means `Active`.
- Rate above the danger threshold means `Hot`.
- `ActivityTracker<N>` records aggregate rates in `RollingBuffer<f64, N>`.
- Tests use synthetic `Instant` values, not sleeping.

This module must not implement strict input/output token-rate split. That
belongs to a future producer-contract issue once a real telemetry producer
exists.

---

## 5. Integration boundaries

`live-core` may be consumed by `vimit`, Android, desktop GUI, TUI, or agent
supervisors, but it must not depend on them.

Allowed inputs:

- numeric metric values;
- aggregate token rates;
- timestamps and durations;
- validated thresholds;
- fixed-size sample histories.

Forbidden inputs:

- API keys or account tokens;
- prompt text;
- conversation transcripts;
- private task names or task titles;
- raw provider responses;
- local filesystem paths;
- UI widgets, Android handles, or notification objects.

---

## 6. Validation baseline

Every change should pass:

```bash
cargo test --locked
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo check --no-default-features --locked
```

Documentation-only changes should also scan for obvious secret/path patterns
when they mention security or integration boundaries.
