---
created_at: 2026-03-08T09:54:15
---

# Knowledge - 1vzGo0000

> Automated synthesis of story reflections.

## Story Knowledge

## Story: Materialize PVM Artifact Variants (1vzGrf000)

### 1w03w0000: Infer Artifact Variants From Canonical Output Paths

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Artifact scripts are invoked with a single output path, but the build and validation logic still needs to know architecture and protection mode without growing a second command contract. |
| **Insight** | Deriving selector intent from the canonical artifact path keeps model selection, cache/store layout, and script behavior aligned. That lets new variants like `x86_64/firecracker/pvm` land without widening the script API. |
| **Suggested Action** | When adding future artifact selectors, keep the path layout canonical and let the scripts derive selector intent from it before introducing new script arguments or hidden environment variables. |
| **Applies To** | `scripts/artifacts/*.sh`, `crates/port-runtime/src/lib.rs`, artifact selector evolution |
| **Applied** | yes |



---

## Story: Add PVM Doctor Host Kit Checks (1vzGrd000)

### 1w03vv000: Add Probe Seams Before Expanding Host Diagnostics

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | Host diagnostics depend on live OS facts like platform, architecture, `/proc/cmdline`, or binary lookup, but the story needs deterministic tests for several incompatible states. |
| **Insight** | A small probe struct is enough to turn environment-dependent diagnostics into a testable seam. That is lower-cost and more maintainable than trying to mock shell commands or `/proc` access ad hoc in each test. |
| **Suggested Action** | When extending `doctor` with more host or platform checks, first introduce or reuse a single fact-gathering struct and keep the decision logic pure over that struct. |
| **Applies To** | `crates/port-runtime/src/lib.rs`, future doctor and platform readiness checks |
| **Applied** | yes |



---

## Story: Model X86 64 PVM Host Kit Contract (1vzGrP000)

### 1w03v0000: Prefer Serializable Contract Fields Over Orphan Helper Types

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | A lane or capability contract exists in code as a helper type, but downstream configuration, examples, and diagnostics still cannot see or round-trip it. |
| **Insight** | A standalone contract type is not enough for operator-facing work. If the contract matters to CLI discovery or future runtime checks, it should live on the serializable model surface so examples, validation, and later stories all reuse the same source of truth. |
| **Suggested Action** | When adding future substrate or host-kit contracts, first attach them to the config/model structs and round-trip them through the checked-in example before building doctor or runtime logic on top. |
| **Applies To** | `crates/port-model/src/lib.rs`, `examples/*.toml`, future substrate contracts |
| **Applied** | yes |



---

## Story: Publish PVM Operator Proof Workflow (1vzGrQ000)

### 1w03x0000: Order Operator Proofs So The Log Tells The Story

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | A repository-local proof script demonstrates several commands in sequence and the first failing-looking lines can dominate the acceptance review even when later commands show the intended end state. |
| **Insight** | Proof scripts should be ordered so the log explains the workflow clearly from top to bottom. Rebuilding prerequisite artifacts before a diagnostic step can make the evidence materially easier to review without changing the underlying behavior. |
| **Suggested Action** | When writing workflow proofs, read the first screenful of the resulting log and reorder the commands until that excerpt communicates the intended operator outcome. |
| **Applies To** | `.keel/stories/*/verify-ac-*.sh`, operator workflow evidence |
| **Applied** | yes |



---

## Synthesis

### EWs7hlr7N: Infer Artifact Variants From Canonical Output Paths

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Artifact scripts are invoked with a single output path, but the build and validation logic still needs to know architecture and protection mode without growing a second command contract. |
| **Insight** | Deriving selector intent from the canonical artifact path keeps model selection, cache/store layout, and script behavior aligned. That lets new variants like `x86_64/firecracker/pvm` land without widening the script API. |
| **Suggested Action** | When adding future artifact selectors, keep the path layout canonical and let the scripts derive selector intent from it before introducing new script arguments or hidden environment variables. |
| **Applies To** | `scripts/artifacts/*.sh`, `crates/port-runtime/src/lib.rs`, artifact selector evolution |
| **Linked Knowledge IDs** | 1w03w0000 |
| **Score** | 0.80 |
| **Confidence** | 0.92 |
| **Applied** | yes |

### XX0gyHFzU: Add Probe Seams Before Expanding Host Diagnostics

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | Host diagnostics depend on live OS facts like platform, architecture, `/proc/cmdline`, or binary lookup, but the story needs deterministic tests for several incompatible states. |
| **Insight** | A small probe struct is enough to turn environment-dependent diagnostics into a testable seam. That is lower-cost and more maintainable than trying to mock shell commands or `/proc` access ad hoc in each test. |
| **Suggested Action** | When extending `doctor` with more host or platform checks, first introduce or reuse a single fact-gathering struct and keep the decision logic pure over that struct. |
| **Applies To** | `crates/port-runtime/src/lib.rs`, future doctor and platform readiness checks |
| **Linked Knowledge IDs** | 1w03vv000 |
| **Score** | 0.82 |
| **Confidence** | 0.94 |
| **Applied** | yes |

### gifJTelIw: Prefer Serializable Contract Fields Over Orphan Helper Types

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | A lane or capability contract exists in code as a helper type, but downstream configuration, examples, and diagnostics still cannot see or round-trip it. |
| **Insight** | A standalone contract type is not enough for operator-facing work. If the contract matters to CLI discovery or future runtime checks, it should live on the serializable model surface so examples, validation, and later stories all reuse the same source of truth. |
| **Suggested Action** | When adding future substrate or host-kit contracts, first attach them to the config/model structs and round-trip them through the checked-in example before building doctor or runtime logic on top. |
| **Applies To** | `crates/port-model/src/lib.rs`, `examples/*.toml`, future substrate contracts |
| **Linked Knowledge IDs** | 1w03v0000 |
| **Score** | 0.83 |
| **Confidence** | 0.93 |
| **Applied** | yes |

### V1vMY6Ve5: Order Operator Proofs So The Log Tells The Story

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | A repository-local proof script demonstrates several commands in sequence and the first failing-looking lines can dominate the acceptance review even when later commands show the intended end state. |
| **Insight** | Proof scripts should be ordered so the log explains the workflow clearly from top to bottom. Rebuilding prerequisite artifacts before a diagnostic step can make the evidence materially easier to review without changing the underlying behavior. |
| **Suggested Action** | When writing workflow proofs, read the first screenful of the resulting log and reorder the commands until that excerpt communicates the intended operator outcome. |
| **Applies To** | `.keel/stories/*/verify-ac-*.sh`, operator workflow evidence |
| **Linked Knowledge IDs** | 1w03x0000 |
| **Score** | 0.74 |
| **Confidence** | 0.90 |
| **Applied** | yes |

