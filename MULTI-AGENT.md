# MULTI-AGENT.md — live-core coordination

Use separate branches or worktrees per issue.

During bootstrap:

- one agent may work on `clock`;
- one agent may work on `rolling_buffer`;
- one agent may work on `threshold`;
- only one agent should edit `Cargo.toml` or CI at a time.

If a task requires expanding scope, stop and create or request a child issue.
Do not refactor another module while finishing a focused module issue.
