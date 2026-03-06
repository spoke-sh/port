---
created_at: 2026-03-06T15:43:39
---

# Knowledge - 1vydgL000

> Automated synthesis of story reflections.

## Story Knowledge

## Story: Document Operator Workflows (1vydgm000)

### 1vyeP0000: Anchor Platform Guidance On `port doctor`

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | When documenting or exposing macOS and Windows operator workflows for a Linux-only runtime |
| **Insight** | The most stable support contract is to document the intended workflow and then use `port doctor` as the runtime gate instead of promising environment capabilities that vary across hosts, especially in WSL-backed setups |
| **Suggested Action** | Keep README, platform docs, and CLI help centered on the exact `port doctor` boundary when platform support depends on Linux host capabilities |
| **Applies To** | README, `docs/operators.md`, CLI help text, diagnostics |
| **Applied** | yes |



---

## Story: Deliver Guest Agent Capabilities (1vydip000)

### 1vyeM0000: Prefer In-Process Daemons For Workspace CLI Integration Tests

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | When a CLI crate needs an integration test against a daemon implemented in another workspace crate |
| **Insight** | Spawning the daemon crate in-process through a dev-dependency is more reliable than discovering a sibling workspace binary from the test harness |
| **Suggested Action** | Prefer `thread::spawn` plus the daemon library entrypoint for workspace-local CLI integration tests unless the binary packaging itself is under test |
| **Applies To** | `crates/*/tests/*.rs`, workspace daemons, CLI integration tests |
| **Applied** | yes |



---

## Story: Implement Local Firecracker Launch (1vydim000)

### 1vye8L000: Firecracker 1.14 Uses `smt` In `machine-config`

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Generating Firecracker JSON config files for `--config-file` launch on Firecracker 1.14.x |
| **Insight** | Current Firecracker 1.14 rejects the older `machine-config.ht_enabled` field and expects `machine-config.smt` instead. Using the older field fails fast during JSON parsing before the microVM starts. |
| **Suggested Action** | Match generated config fields to the live Firecracker binary in the dev shell and confirm with an executable launch proof before trusting older examples. |
| **Applies To** | `crates/port-runtime/*`, Firecracker config generation, local launch proofs |
| **Applied** | yes |



---

## Story: Build Artifact Pipelines And Docs (1vydit000)

### 1vyeN0000: Build Firecracker Rootfs Images Without Privileged Mounts

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When Port needs to build a minimal guest image inside the repository or CI without root-only mount steps |
| **Insight** | `mkfs.ext4 -d` plus `ldd`-discovered shared libraries is enough to assemble a bootable ext4 guest image carrying dynamic binaries like BusyBox and `port-guest-agent` |
| **Suggested Action** | Prefer staging-directory image assembly with `mkfs.ext4 -d`, `e2fsck`, and `debugfs` before introducing mount-based image mutation tooling |
| **Applies To** | `scripts/artifacts/*.sh`, guest image pipelines, future cloud image assembly |
| **Applied** | yes |



---

## Story: Bootstrap Port Workspace And CLI (1vydgl000)

### 1vye3K000: Verify Annotations Are Required For Story Evidence

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | Recording proof with `keel story record` against story acceptance criteria |
| **Insight** | `keel story record` ignores ACs unless each criterion includes an inline HTML comment that repeats the AC ID and declares the verification technique or command, for example ``. |
| **Suggested Action** | Add verify annotations while authoring or refining stories, before starting implementation, so evidence recording and submit gates do not stall later. |
| **Applies To** | `.keel/stories/*/README.md` |
| **Applied** | yes |



---

## Synthesis

### sf2SLS4BV: Anchor Platform Guidance On `port doctor`

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | When documenting or exposing macOS and Windows operator workflows for a Linux-only runtime |
| **Insight** | The most stable support contract is to document the intended workflow and then use `port doctor` as the runtime gate instead of promising environment capabilities that vary across hosts, especially in WSL-backed setups |
| **Suggested Action** | Keep README, platform docs, and CLI help centered on the exact `port doctor` boundary when platform support depends on Linux host capabilities |
| **Applies To** | README, `docs/operators.md`, CLI help text, diagnostics |
| **Linked Knowledge IDs** | 1vyeP0000 |
| **Score** | 0.74 |
| **Confidence** | 0.91 |
| **Applied** | yes |

### MJXXnMVfL: Prefer In-Process Daemons For Workspace CLI Integration Tests

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | When a CLI crate needs an integration test against a daemon implemented in another workspace crate |
| **Insight** | Spawning the daemon crate in-process through a dev-dependency is more reliable than discovering a sibling workspace binary from the test harness |
| **Suggested Action** | Prefer `thread::spawn` plus the daemon library entrypoint for workspace-local CLI integration tests unless the binary packaging itself is under test |
| **Applies To** | `crates/*/tests/*.rs`, workspace daemons, CLI integration tests |
| **Linked Knowledge IDs** | 1vyeM0000 |
| **Score** | 0.78 |
| **Confidence** | 0.90 |
| **Applied** | yes |

### LcOvXV6zu: Firecracker 1.14 Uses `smt` In `machine-config`

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Generating Firecracker JSON config files for `--config-file` launch on Firecracker 1.14.x |
| **Insight** | Current Firecracker 1.14 rejects the older `machine-config.ht_enabled` field and expects `machine-config.smt` instead. Using the older field fails fast during JSON parsing before the microVM starts. |
| **Suggested Action** | Match generated config fields to the live Firecracker binary in the dev shell and confirm with an executable launch proof before trusting older examples. |
| **Applies To** | `crates/port-runtime/*`, Firecracker config generation, local launch proofs |
| **Linked Knowledge IDs** | 1vye8L000 |
| **Score** | 0.88 |
| **Confidence** | 0.97 |
| **Applied** | yes |

### XljFO1r2K: Build Firecracker Rootfs Images Without Privileged Mounts

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When Port needs to build a minimal guest image inside the repository or CI without root-only mount steps |
| **Insight** | `mkfs.ext4 -d` plus `ldd`-discovered shared libraries is enough to assemble a bootable ext4 guest image carrying dynamic binaries like BusyBox and `port-guest-agent` |
| **Suggested Action** | Prefer staging-directory image assembly with `mkfs.ext4 -d`, `e2fsck`, and `debugfs` before introducing mount-based image mutation tooling |
| **Applies To** | `scripts/artifacts/*.sh`, guest image pipelines, future cloud image assembly |
| **Linked Knowledge IDs** | 1vyeN0000 |
| **Score** | 0.86 |
| **Confidence** | 0.93 |
| **Applied** | yes |

### jl0QxU981: Verify Annotations Are Required For Story Evidence

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | Recording proof with `keel story record` against story acceptance criteria |
| **Insight** | `keel story record` ignores ACs unless each criterion includes an inline HTML comment that repeats the AC ID and declares the verification technique or command, for example ``. |
| **Suggested Action** | Add verify annotations while authoring or refining stories, before starting implementation, so evidence recording and submit gates do not stall later. |
| **Applies To** | `.keel/stories/*/README.md` |
| **Linked Knowledge IDs** | 1vye3K000 |
| **Score** | 0.83 |
| **Confidence** | 0.92 |
| **Applied** | yes |

