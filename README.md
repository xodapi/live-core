# live-core

Core primitives for live autonomous-agent telemetry: clocks, buffers,
thresholds, and state transitions.

The first implementation track is complete:

1. `clock`: `Clock` + `Instant`
2. `rolling_buffer`: fixed-size metric history
3. `threshold`: pure level classification
4. `activity`: aggregate token-rate activity state

See [docs/LIVE_CORE_FULL_SPEC.ru.md](docs/LIVE_CORE_FULL_SPEC.ru.md) for the
current specification and
[docs/LIVE_CORE_SECURITY_AUDIT.ru.md](docs/LIVE_CORE_SECURITY_AUDIT.ru.md) for
security boundaries.

The crate is intentionally small, dependency-light, and prepared for `no_std`
work where practical.
