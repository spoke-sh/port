---
created_at: 2026-03-08T20:59:11
---

# Knowledge - 1vzQEj000

> Automated synthesis of story reflections.

## Story Knowledge

## Story: Implement Hosted Detached Forward Inventory (1vzQJ6000)

### 1vzQL2000: Detached Forward Runtime Helpers Must Not Assume `current_exe` Is The Port CLI

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Launching detached forward daemons from shared runtime code that also runs under library tests and hosted node-agent servers |
| **Insight** | `std::env::current_exe()` can resolve to a Rust test harness instead of the `port` CLI binary, so detached child-process launch must prefer an explicit or workspace `port` binary path before falling back to the current executable. |
| **Suggested Action** | Keep detached helper launchers behind a resolver that checks `PORT_DETACHED_FORWARD_EXECUTABLE` and the workspace `target/debug/port` path before using `current_exe()`. |
| **Applies To** | `crates/port-runtime/src/lib.rs`, detached runtime helpers, hosted node-agent tests |
| **Applied** | yes |



---

## Story: Define Hosted Detached Forward Contract (1vzQIq000)

### 1vzQKh000: Keel Story Record Proof Mapping Can Drift Across Same-SRS ACs

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | Recording command proof for stories whose acceptance criteria share one SRS requirement prefix |
| **Insight** | `keel story record` can overwrite the inline `proof:` annotation for an earlier AC with the later AC's evidence file, while leaving the checkbox state unchanged. |
| **Suggested Action** | Inspect the story README after every multi-AC `story record` run and correct proof links or checkboxes before submit. |
| **Applies To** | `.keel/stories/*/README.md`, `keel story record` workflows |
| **Applied** | yes |



---

## Story: Route Hosted Detached Forward Lifecycle (1vzQJB000)

### 1vzQLp000: Bogus Client Runtime Roots Are A Strong No-Fallback Hosted CLI Proof

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | Verifying that hosted CLI commands no longer read repo-local runtime state after moving to the live control-plane and node-agent path |
| **Insight** | If the client config points the hosted node runtime root at a bogus path while the server-side config keeps the real runtime root, any successful hosted command proves the CLI is using remote transport rather than local state inspection. |
| **Suggested Action** | Keep using split server/client hosted configs with a bogus client runtime root in CLI integration tests for hosted transport stories. |
| **Applies To** | `crates/port-cli/tests/*`, hosted machine and guest transport tests |
| **Applied** | yes |



---

## Story: Publish Hosted Detached Forward Workflow (1vzQIy000)

### 1w04h0000: Keep Hosted Demo Socket Paths Short Under Nested Nix Shells

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | Running hosted demo or proof scripts through `keel verify` or `keel story record` inside `nix develop` |
| **Insight** | Nested Nix shells can set `TMPDIR` to a long path that pushes Unix socket files past `SUN_LEN`, so demo proofs that rely on Unix sockets must pick a short temp root and avoid repeated `cargo run` startup races |
| **Suggested Action** | Use a fixed short `/tmp` workdir prefix and prebuild binaries before backgrounding guest-agent, node-agent, or control-plane demo processes |
| **Applies To** | scripts/hosted-demo.sh, hosted CLI proof scripts, Unix-socket integration tests |
| **Applied** | yes |



---

## Synthesis

### td2mihn3g: Detached Forward Runtime Helpers Must Not Assume `current_exe` Is The Port CLI

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Launching detached forward daemons from shared runtime code that also runs under library tests and hosted node-agent servers |
| **Insight** | `std::env::current_exe()` can resolve to a Rust test harness instead of the `port` CLI binary, so detached child-process launch must prefer an explicit or workspace `port` binary path before falling back to the current executable. |
| **Suggested Action** | Keep detached helper launchers behind a resolver that checks `PORT_DETACHED_FORWARD_EXECUTABLE` and the workspace `target/debug/port` path before using `current_exe()`. |
| **Applies To** | `crates/port-runtime/src/lib.rs`, detached runtime helpers, hosted node-agent tests |
| **Linked Knowledge IDs** | 1vzQL2000 |
| **Score** | 0.89 |
| **Confidence** | 0.97 |
| **Applied** | yes |

### HWeiHhOWB: Keel Story Record Proof Mapping Can Drift Across Same-SRS ACs

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | Recording command proof for stories whose acceptance criteria share one SRS requirement prefix |
| **Insight** | `keel story record` can overwrite the inline `proof:` annotation for an earlier AC with the later AC's evidence file, while leaving the checkbox state unchanged. |
| **Suggested Action** | Inspect the story README after every multi-AC `story record` run and correct proof links or checkboxes before submit. |
| **Applies To** | `.keel/stories/*/README.md`, `keel story record` workflows |
| **Linked Knowledge IDs** | 1vzQKh000 |
| **Score** | 0.84 |
| **Confidence** | 0.96 |
| **Applied** | yes |

### eR9JRIYwU: Bogus Client Runtime Roots Are A Strong No-Fallback Hosted CLI Proof

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | Verifying that hosted CLI commands no longer read repo-local runtime state after moving to the live control-plane and node-agent path |
| **Insight** | If the client config points the hosted node runtime root at a bogus path while the server-side config keeps the real runtime root, any successful hosted command proves the CLI is using remote transport rather than local state inspection. |
| **Suggested Action** | Keep using split server/client hosted configs with a bogus client runtime root in CLI integration tests for hosted transport stories. |
| **Applies To** | `crates/port-cli/tests/*`, hosted machine and guest transport tests |
| **Linked Knowledge IDs** | 1vzQLp000 |
| **Score** | 0.86 |
| **Confidence** | 0.97 |
| **Applied** | yes |

### ezm8lhF9j: Keep Hosted Demo Socket Paths Short Under Nested Nix Shells

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | Running hosted demo or proof scripts through `keel verify` or `keel story record` inside `nix develop` |
| **Insight** | Nested Nix shells can set `TMPDIR` to a long path that pushes Unix socket files past `SUN_LEN`, so demo proofs that rely on Unix sockets must pick a short temp root and avoid repeated `cargo run` startup races |
| **Suggested Action** | Use a fixed short `/tmp` workdir prefix and prebuild binaries before backgrounding guest-agent, node-agent, or control-plane demo processes |
| **Applies To** | scripts/hosted-demo.sh, hosted CLI proof scripts, Unix-socket integration tests |
| **Linked Knowledge IDs** | 1w04h0000 |
| **Score** | 0.87 |
| **Confidence** | 0.96 |
| **Applied** | yes |

