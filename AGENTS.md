# AGENTS.md — live-core agent workflow

Every code or documentation change starts from a GitHub Issue and ends in a PR.

## Required Flow

1. Pick an open issue with label `agent` and without label `blocked`.
2. Assign yourself and comment that you are starting.
3. Create a branch `issue-N-short-slug`.
4. Work only within the issue scope.
5. Run:

```bash
cargo test --locked
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

6. Commit with a conventional message and reference the issue.
7. Push and open a draft PR.
8. Mark ready only after checks pass.

## Safety

- Do not commit `.env`, API keys, prompts, transcripts, task titles, private
  paths, passwords, or keystores.
- Do not add dependencies without explicit issue scope.
- Do not mix vimit runtime code into live-core.
- Keep modules small and pure: no network I/O in core primitives.

## Bootstrap Order

Follow `docs/LIVE_CORE_BOOTSTRAP.ru.md`:

1. `clock`
2. `rolling_buffer`
3. `threshold`

Do not start Rig, jj/git-cliff, Bevy/3D, iOS, async poller, Observer,
state_machine, offline_tracker, or v2 smoothing work during bootstrap.
