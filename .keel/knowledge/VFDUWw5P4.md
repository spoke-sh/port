---
source_type: Story
source: stories/VFDk8gGoC/REFLECT.md
scope: VFDhlRjOf/VFDk8fdnG
source_story_id: VFDk8gGoC
created_at: 2026-03-28T21:32:05
---

### VFDUWw5P4: Local guest-agent execs need guest-root-relative paths

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | Repo-local tests that drive cluster or guest workflows through the fake local `port-guest-agent` socket rather than a real VM. |
| **Insight** | The fake local guest-agent resolves copy paths against the guest root, but exec commands are not chrooted. To keep repo-local tests aligned with real-guest semantics, run execs with `cwd = "/"` and use guest-root-relative paths like `opt/...` instead of host-absolute `/opt/...` paths. |
| **Suggested Action** | When adding guest exec proofs or runtime helpers for local harnesses, strip the leading slash from guest paths for the shell command and set the exec cwd to guest `/`. |
| **Applies To** | `crates/port-runtime/src/lib.rs`, `crates/port-cli/tests/*`, local guest-agent harnesses |
| **Linked Knowledge IDs** |  |
| **Observed At** | 2026-03-29T04:35:00+00:00 |
| **Score** | 0.87 |
| **Confidence** | 0.96 |
| **Applied** | yes |
