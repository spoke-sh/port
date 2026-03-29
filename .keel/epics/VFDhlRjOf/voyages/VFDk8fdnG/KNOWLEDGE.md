---
created_at: 2026-03-28T22:04:05
---

# Knowledge - VFDk8fdnG

> Automated synthesis of story reflections.

## Story Knowledge

## Story: Publish Cluster Operator Contract And Infra Handoff Proof (VFDk8ggoV)

### VFG3hLr2M: Proof scripts must honor Cargo target indirection

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | Renderer-backed proof scripts that build binaries inside `nix develop` |
| **Insight** | The dev shell can redirect Cargo outputs through `CARGO_TARGET_DIR`, so proof scripts that hardcode `./target/debug/...` can execute stale binaries even after a successful build. |
| **Suggested Action** | Resolve built binary paths from `$CARGO_TARGET_DIR` with a fallback to the repo `target` directory, or use `cargo run` when the executable path must follow the active shell contract. |
| **Applies To** | `scripts/render-*.sh` |
| **Applied** | `scripts/render-local-cluster-proof.sh` now resolves `port` and `port-guest-agent` from the active Cargo target root. |



---

## Story: Implement Cluster Lifecycle Health And Kubeconfig Surfaces (VFDk8gRoD)

### VFDmLq9xQ: Firecracker test doubles must preserve launch argv

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | Runtime and CLI tests that rely on local Firecracker machine-status detection |
| **Insight** | Port classifies local Firecracker processes by inspecting live argv for both `firecracker` and `--id <machine>`, so a fake helper that `exec`s into another binary can make a healthy test process look stale. |
| **Suggested Action** | Keep fake Firecracker helpers running under a command line that still includes the `firecracker` script path and launch args, or update the test double explicitly when machine-status matching changes. |
| **Applies To** | crates/port-runtime/src/lib.rs; crates/port-cli/tests/machine_commands.rs |
| **Applied** | yes |



---

## Story: Stage Offline K3s Artifacts And Guest Profile (VFDk8gGoC)

### VFDUWw5P4: Local guest-agent execs need guest-root-relative paths

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | Repo-local tests that drive cluster or guest workflows through the fake local `port-guest-agent` socket rather than a real VM. |
| **Insight** | The fake local guest-agent resolves copy paths against the guest root, but exec commands are not chrooted. To keep repo-local tests aligned with real-guest semantics, run execs with `cwd = "/"` and use guest-root-relative paths like `opt/...` instead of host-absolute `/opt/...` paths. |
| **Suggested Action** | When adding guest exec proofs or runtime helpers for local harnesses, strip the leading slash from guest paths for the shell command and set the exec cwd to guest `/`. |
| **Applies To** | `crates/port-runtime/src/lib.rs`, `crates/port-cli/tests/*`, local guest-agent harnesses |
| **Applied** | yes |



---

## Synthesis

### QlGPRqWWm: Proof scripts must honor Cargo target indirection

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | Renderer-backed proof scripts that build binaries inside `nix develop` |
| **Insight** | The dev shell can redirect Cargo outputs through `CARGO_TARGET_DIR`, so proof scripts that hardcode `./target/debug/...` can execute stale binaries even after a successful build. |
| **Suggested Action** | Resolve built binary paths from `$CARGO_TARGET_DIR` with a fallback to the repo `target` directory, or use `cargo run` when the executable path must follow the active shell contract. |
| **Applies To** | `scripts/render-*.sh` |
| **Linked Knowledge IDs** | VFG3hLr2M |
| **Score** | 0.84 |
| **Confidence** | 0.98 |
| **Applied** | `scripts/render-local-cluster-proof.sh` now resolves `port` and `port-guest-agent` from the active Cargo target root. |

### vlOH3iS4u: Firecracker test doubles must preserve launch argv

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | Runtime and CLI tests that rely on local Firecracker machine-status detection |
| **Insight** | Port classifies local Firecracker processes by inspecting live argv for both `firecracker` and `--id <machine>`, so a fake helper that `exec`s into another binary can make a healthy test process look stale. |
| **Suggested Action** | Keep fake Firecracker helpers running under a command line that still includes the `firecracker` script path and launch args, or update the test double explicitly when machine-status matching changes. |
| **Applies To** | crates/port-runtime/src/lib.rs; crates/port-cli/tests/machine_commands.rs |
| **Linked Knowledge IDs** | VFDmLq9xQ |
| **Score** | 0.74 |
| **Confidence** | 0.92 |
| **Applied** | yes |

### 14u0BuT2u: Local guest-agent execs need guest-root-relative paths

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | Repo-local tests that drive cluster or guest workflows through the fake local `port-guest-agent` socket rather than a real VM. |
| **Insight** | The fake local guest-agent resolves copy paths against the guest root, but exec commands are not chrooted. To keep repo-local tests aligned with real-guest semantics, run execs with `cwd = "/"` and use guest-root-relative paths like `opt/...` instead of host-absolute `/opt/...` paths. |
| **Suggested Action** | When adding guest exec proofs or runtime helpers for local harnesses, strip the leading slash from guest paths for the shell command and set the exec cwd to guest `/`. |
| **Applies To** | `crates/port-runtime/src/lib.rs`, `crates/port-cli/tests/*`, local guest-agent harnesses |
| **Linked Knowledge IDs** | VFDUWw5P4 |
| **Score** | 0.87 |
| **Confidence** | 0.96 |
| **Applied** | yes |

