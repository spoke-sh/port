---
created_at: 2026-03-08T10:15:26
---

# Knowledge - 1vzHPo000

> Automated synthesis of story reflections.

## Story Knowledge

## Story: Model Pvm Node Capability Contract (1vzHRt000)

### 1w04a0000: Prefer Per-Ac Verify Annotations Over Shared Proof Blocks

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | A story has multiple acceptance criteria and `keel story record` is used to capture rerunnable command proofs. |
| **Insight** | Shared summary-level verify comments can cause proof metadata to drift or cross-wire between acceptance criteria, while one inline verify annotation per AC stays stable. |
| **Suggested Action** | For multi-AC stories, use repo-rooted `verify-ac-*.sh` scripts and only the per-AC inline verify comment form before recording evidence. |
| **Applies To** | `.keel/stories/*/README.md`, `.keel/stories/*/verify-ac-*.sh` |
| **Applied** | yes |



---

## Story: Select Local Pvm Runtime Inputs (1vzHSA000)

### 1w04f0000: Split Lane-Specific Binary Selection Into A Pure Helper

| Field | Value |
|-------|-------|
| **Category** | code |
| **Context** | A launch path needs to choose different VMM binaries for different protection modes, but spawning the VMM is too expensive and environment-sensitive for most unit tests. |
| **Insight** | Pulling lane-specific binary selection into a pure helper makes the protection-mode contract testable without depending on a live Firecracker process or host PATH mutations. |
| **Suggested Action** | When adding more substrate or protection-mode launch inputs, isolate the selection logic in a pure helper before wiring it into the process-spawn path. |
| **Applies To** | `crates/port-runtime/src/lib.rs`, launch-path selection helpers |
| **Applied** | yes |



---

## Synthesis

### 1DWcfzQD4: Prefer Per-Ac Verify Annotations Over Shared Proof Blocks

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | A story has multiple acceptance criteria and `keel story record` is used to capture rerunnable command proofs. |
| **Insight** | Shared summary-level verify comments can cause proof metadata to drift or cross-wire between acceptance criteria, while one inline verify annotation per AC stays stable. |
| **Suggested Action** | For multi-AC stories, use repo-rooted `verify-ac-*.sh` scripts and only the per-AC inline verify comment form before recording evidence. |
| **Applies To** | `.keel/stories/*/README.md`, `.keel/stories/*/verify-ac-*.sh` |
| **Linked Knowledge IDs** | 1w04a0000 |
| **Score** | 0.72 |
| **Confidence** | 0.90 |
| **Applied** | yes |

### 1rZTLCji3: Split Lane-Specific Binary Selection Into A Pure Helper

| Field | Value |
|-------|-------|
| **Category** | code |
| **Context** | A launch path needs to choose different VMM binaries for different protection modes, but spawning the VMM is too expensive and environment-sensitive for most unit tests. |
| **Insight** | Pulling lane-specific binary selection into a pure helper makes the protection-mode contract testable without depending on a live Firecracker process or host PATH mutations. |
| **Suggested Action** | When adding more substrate or protection-mode launch inputs, isolate the selection logic in a pure helper before wiring it into the process-spawn path. |
| **Applies To** | `crates/port-runtime/src/lib.rs`, launch-path selection helpers |
| **Linked Knowledge IDs** | 1w04f0000 |
| **Score** | 0.75 |
| **Confidence** | 0.93 |
| **Applied** | yes |

