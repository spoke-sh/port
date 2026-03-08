---
created_at: 2026-03-07T18:11:24
---

# Knowledge - 1vz2ky000

> Automated synthesis of story reflections.

## Story Knowledge

## Story: Publish Hosted Node Agent Contract (1vz2oh000)

### 1vz3N8000: Hosted Port Should Broker The Guest Protocol, Not Replace It

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When Port extends from local runtime ownership into a hosted node-agent plus control-plane architecture |
| **Insight** | Port already has a usable guest protocol and CLI vocabulary. The hosted layer should add routing, ownership, and auth around that protocol instead of inventing a second hosted-only guest API. |
| **Suggested Action** | Keep hosted design and implementation work centered on tunneling the existing guest protocol through node agents and control-plane sessions, with the CLI remaining the same surface in local and hosted modes. |
| **Applies To** | docs/hosted.md, crates/port-runtime/**, crates/port-cli/**, future hosted-control crates |
| **Applied** | yes |



---

## Story: Define Artifact Mobility Commands And Contracts (1vz2on000)

### 1vz3rU000: Noninteractive Story Record Needs Editor Override

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | Recording `keel story record --cmd ...` proofs from the harness without an attached editor session |
| **Insight** | `keel story record` still opens a manual-evidence editor even for command proofs unless the editor exits immediately; setting `EDITOR=true` keeps the command proof path noninteractive. |
| **Suggested Action** | Use `EDITOR=true nix develop -c keel story record ... --cmd "<command>"` for automated proof capture and only fall back to a PTY editor when a manual note is genuinely needed. |
| **Applies To** | `.keel/stories/*`, proof recording workflow |
| **Applied** | yes |



---

## Story: Add Machine Inventory Status And Stop (1vz2oY000)

### 1vz3Mb000: Runtime Lifecycle Should Key Off Runtime State, Not Config

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When Port adds lifecycle or inspection commands that must keep working after launch, across config drift, or under a future hosted control plane |
| **Insight** | `launch` is model-backed, but `list`, `status`, and `stop` should key off runtime manifests and PID inspection instead of reloading the machine model. That keeps lifecycle commands usable after a VM already exists and matches the control-plane direction for hosted Port. |
| **Suggested Action** | Treat runtime-root inspection data as the source of truth for post-launch lifecycle commands, and only require the model for launch-time validation or artifact resolution. |
| **Applies To** | crates/port-runtime/**, crates/port-cli/**, docs/operators.md, README.md |
| **Applied** | yes |



---

## Story: Model Substrates And Protection Modes (1vz2oQ000)

### 1vz3A9000: Separate Architecture From Protection-Mode Support

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When Port expands from one Linux Firecracker lane into PVM, AVF, and additional substrate lanes |
| **Insight** | General architecture support and protected-virtualization support cannot share one boolean or one token. Port needs explicit substrate, protection-mode, and architecture fields so it can say “arm64 exists” without implying “arm64 PVM is shipped.” |
| **Suggested Action** | Keep machine compatibility validation and docs keyed on substrate plus protection mode plus resolved architecture, and reject unsupported combinations explicitly. |
| **Applies To** | crates/port-model/**, crates/port-runtime/**, docs/**, examples/port.toml |
| **Applied** | yes |



---

## Synthesis

### wnw89PsRA: Hosted Port Should Broker The Guest Protocol, Not Replace It

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When Port extends from local runtime ownership into a hosted node-agent plus control-plane architecture |
| **Insight** | Port already has a usable guest protocol and CLI vocabulary. The hosted layer should add routing, ownership, and auth around that protocol instead of inventing a second hosted-only guest API. |
| **Suggested Action** | Keep hosted design and implementation work centered on tunneling the existing guest protocol through node agents and control-plane sessions, with the CLI remaining the same surface in local and hosted modes. |
| **Applies To** | docs/hosted.md, crates/port-runtime/**, crates/port-cli/**, future hosted-control crates |
| **Linked Knowledge IDs** | 1vz3N8000 |
| **Score** | 0.89 |
| **Confidence** | 0.94 |
| **Applied** | yes |

### bjaDLpFy1: Noninteractive Story Record Needs Editor Override

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | Recording `keel story record --cmd ...` proofs from the harness without an attached editor session |
| **Insight** | `keel story record` still opens a manual-evidence editor even for command proofs unless the editor exits immediately; setting `EDITOR=true` keeps the command proof path noninteractive. |
| **Suggested Action** | Use `EDITOR=true nix develop -c keel story record ... --cmd "<command>"` for automated proof capture and only fall back to a PTY editor when a manual note is genuinely needed. |
| **Applies To** | `.keel/stories/*`, proof recording workflow |
| **Linked Knowledge IDs** | 1vz3rU000 |
| **Score** | 0.80 |
| **Confidence** | 0.92 |
| **Applied** | yes |

### ddefrtZNC: Runtime Lifecycle Should Key Off Runtime State, Not Config

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When Port adds lifecycle or inspection commands that must keep working after launch, across config drift, or under a future hosted control plane |
| **Insight** | `launch` is model-backed, but `list`, `status`, and `stop` should key off runtime manifests and PID inspection instead of reloading the machine model. That keeps lifecycle commands usable after a VM already exists and matches the control-plane direction for hosted Port. |
| **Suggested Action** | Treat runtime-root inspection data as the source of truth for post-launch lifecycle commands, and only require the model for launch-time validation or artifact resolution. |
| **Applies To** | crates/port-runtime/**, crates/port-cli/**, docs/operators.md, README.md |
| **Linked Knowledge IDs** | 1vz3Mb000 |
| **Score** | 0.91 |
| **Confidence** | 0.95 |
| **Applied** | yes |

### wD56Iypwn: Separate Architecture From Protection-Mode Support

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When Port expands from one Linux Firecracker lane into PVM, AVF, and additional substrate lanes |
| **Insight** | General architecture support and protected-virtualization support cannot share one boolean or one token. Port needs explicit substrate, protection-mode, and architecture fields so it can say “arm64 exists” without implying “arm64 PVM is shipped.” |
| **Suggested Action** | Keep machine compatibility validation and docs keyed on substrate plus protection mode plus resolved architecture, and reject unsupported combinations explicitly. |
| **Applies To** | crates/port-model/**, crates/port-runtime/**, docs/**, examples/port.toml |
| **Linked Knowledge IDs** | 1vz3A9000 |
| **Score** | 0.93 |
| **Confidence** | 0.95 |
| **Applied** | yes |

