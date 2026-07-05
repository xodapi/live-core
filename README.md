# live-core

Core primitives for live autonomous-agent telemetry: clocks, buffers,
thresholds, and state transitions.

This repository is in bootstrap state. The first implementation track is:

1. `clock`: `Clock` + `Instant`
2. `rolling_buffer`: fixed-size metric history
3. `threshold`: pure level classification

See [docs/LIVE_CORE_BOOTSTRAP.ru.md](docs/LIVE_CORE_BOOTSTRAP.ru.md) for the
current source of truth.

The crate is intentionally small, dependency-light, and prepared for `no_std`
work where practical.
