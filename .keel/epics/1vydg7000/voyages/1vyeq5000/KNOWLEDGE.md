---
created_at: 2026-03-06T16:02:57
---

# Knowledge - 1vyeq5000

> Automated synthesis of story reflections.

## Story Knowledge

## Story: Publish Cloud Support Matrix (1vyerX000)

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

## Story: Model Cloud Linux Providers (1vyerj000)

### 1vyezc000: Parse-Test Canonical Example Configs

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | When `examples/port.toml` becomes a canonical CLI proof surface for new provider or platform lanes. |
| **Insight** | String-matching example config content is not enough once the example carries workflow-critical provider identity; a parse test catches drift between the checked-in example and the shared model. |
| **Suggested Action** | Add a `PortConfig::from_path` test for canonical example files whenever model shape changes. |
| **Applies To** | `examples/*.toml`, `crates/port-model/src/lib.rs` |
| **Applied** | yes |



---

## Story: Implement Remote Linux Diagnostics (1vyetE000)

### 1vyf2b000: Guard Remote Launches Before Local Preflight

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When a CLI exposes both local-only runtime checks and partially implemented remote/cloud host targets. |
| **Insight** | Remote-provider launch requests should be rejected before Linux-local preflight runs, otherwise missing local prerequisites can hide the real provider-specific support boundary. |
| **Suggested Action** | Resolve the target machine and host first, return provider-aware guidance for remote lanes, and run `/dev/kvm` or local-binary checks only for actual local-launch paths. |
| **Applies To** | `crates/port-runtime/src/lib.rs`, launch guards, future remote control lanes |
| **Applied** | yes |



---

## Synthesis

### Qo90g7RY8: Anchor Platform Guidance On `port doctor`

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

### cpKQY1e9Z: Parse-Test Canonical Example Configs

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | When `examples/port.toml` becomes a canonical CLI proof surface for new provider or platform lanes. |
| **Insight** | String-matching example config content is not enough once the example carries workflow-critical provider identity; a parse test catches drift between the checked-in example and the shared model. |
| **Suggested Action** | Add a `PortConfig::from_path` test for canonical example files whenever model shape changes. |
| **Applies To** | `examples/*.toml`, `crates/port-model/src/lib.rs` |
| **Linked Knowledge IDs** | 1vyezc000 |
| **Score** | 0.77 |
| **Confidence** | 0.94 |
| **Applied** | yes |

### ZFDQ66amd: Guard Remote Launches Before Local Preflight

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When a CLI exposes both local-only runtime checks and partially implemented remote/cloud host targets. |
| **Insight** | Remote-provider launch requests should be rejected before Linux-local preflight runs, otherwise missing local prerequisites can hide the real provider-specific support boundary. |
| **Suggested Action** | Resolve the target machine and host first, return provider-aware guidance for remote lanes, and run `/dev/kvm` or local-binary checks only for actual local-launch paths. |
| **Applies To** | `crates/port-runtime/src/lib.rs`, launch guards, future remote control lanes |
| **Linked Knowledge IDs** | 1vyf2b000 |
| **Score** | 0.84 |
| **Confidence** | 0.95 |
| **Applied** | yes |

