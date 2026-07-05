# live-core — security and privacy audit

**Status:** canonical v1 audit after bootstrap.

`live-core` is designed as a pure telemetry primitive crate. Its strongest
security property is narrow scope: it should classify numeric signals without
seeing secrets, prompts, transcripts, UI handles, or network responses.

---

## 1. Data classification

Allowed data:

- aggregate numeric rates, such as tokens per second;
- timestamps and durations;
- threshold numbers;
- small fixed-size numeric histories;
- enum states derived from numeric metrics.

Forbidden data:

- API keys, account tokens, refresh tokens, or signing secrets;
- raw prompts, completions, transcripts, summaries, or task titles;
- provider JSON payloads beyond already-sanitized numeric fields;
- local private filesystem paths;
- Android, desktop, terminal, or browser UI handles;
- network endpoints, headers, or request bodies.

---

## 2. Current module review

### 2.1 `clock`

Risk: accidental wall-clock dependence or nondeterministic tests.

Controls:

- Core trait accepts synthetic clocks.
- `Instant` is a numeric newtype.
- `StdClock` is gated behind the `std` feature.
- Tests can construct `Instant` directly.

Residual risk: downstream consumers may still pass wall-clock-derived values.
That is acceptable because the crate does not store or transmit them.

### 2.2 `rolling_buffer`

Risk: unexpected allocation or retaining sensitive values.

Controls:

- The v1 type is generic, fixed-size, and no-allocation.
- The intended use is numeric metrics only.
- The full spec forbids payload, prompt, transcript, and task-title storage.
- `N = 0` is documented as a no-op instead of a panic path.

Residual risk: generic `T` technically allows a downstream caller to store a
sensitive type. Repository-level guidance must keep public examples and
integrations numeric-only.

### 2.3 `threshold`

Risk: invalid thresholds or undefined floating-point behavior.

Controls:

- `Threshold::new` validates finite bounds and strict order.
- `validate` is public for literal structs.
- `NaN`, positive infinity, and negative infinity behavior is documented and
tested.
- No UI, provider, or account context enters classification.

Residual risk: public fields allow invalid literal construction. `classify`
validates and panics on invalid bounds, so this fails closed during testing.

### 2.4 `activity`

Risk: accidentally treating raw telemetry as core input or keeping an agent
active forever from stale data.

Controls:

- `TokenRateSample` accepts only finite, non-negative aggregate rates.
- Missing samples classify as `Unknown`.
- Zero-rate samples classify as `Idle` after the configured timeout.
- Hot state uses validated `Threshold`.
- `ActivityTracker` stores only numeric rates and the latest sanitized sample.

Residual risk: producer freshness policy is still owned by the integration
layer. Future idle/offline tracker work should make stale-positive handling
explicit without adding provider coupling.

---

## 3. Dependency and runtime posture

Current posture:

- no runtime dependencies;
- no network I/O;
- no filesystem I/O;
- no logging;
- no background threads;
- `std` feature enabled by default only for `StdClock`;
- `no_std` check is part of CI.

Future dependencies require an explicit issue and must justify why the same
behavior cannot be implemented with core/std primitives.

---

## 4. Integration rules

Downstream integrations should sanitize before calling `live-core`.

Allowed integration pattern:

1. Provider/collector code reads external data.
2. Integration layer extracts numeric rates and timestamps.
3. `live-core` receives only sanitized numbers and durations.
4. UI layer renders derived states.

Forbidden integration pattern:

1. Provider/collector code passes raw response payloads to `live-core`.
2. `live-core` parses provider-specific JSON.
3. `live-core` logs, stores, or forwards prompt/task data.

---

## 5. Threat checklist

| Threat | Current status | Control |
| --- | --- | --- |
| Secret leakage | Low | Core accepts no keys or tokens |
| Prompt/transcript retention | Low | Forbidden by spec; no storage APIs |
| Private path leakage | Low | Docs and examples must avoid local paths |
| Network exfiltration | Low | No network code |
| UI privilege coupling | Low | No Android/desktop/TUI types |
| Infinite memory growth | Low | Fixed-size buffers |
| Time-based flaky tests | Low | Synthetic `Instant` support |
| Invalid threshold behavior | Medium | `new`, `validate`, and panic-on-classify |

---

## 6. PR review checklist

Before merging a `live-core` PR, verify:

- no new network, filesystem, logging, or background runtime behavior;
- no examples with API keys, private paths, prompts, transcripts, or task
  titles;
- no UI or Android-specific types in core modules;
- no new dependency without explicit issue scope;
- `cargo test --locked` passes;
- `cargo clippy --all-targets -- -D warnings` passes;
- `cargo fmt --check` passes;
- `cargo check --no-default-features --locked` passes.

---

## 7. Future work boundaries

The following remain future work, not active implementation:

- EMA/ExponentialSmoother;
- percentile/median;
- serde support;
- Rig or other agent frameworks;
- jj/git-cliff workflow automation;
- Bevy/3D visualization;
- iOS integration;
- provider-specific telemetry producers.

Each item needs a focused issue and a concrete product reason before code
starts.
