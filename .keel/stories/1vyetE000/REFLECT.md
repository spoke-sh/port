---
created_at: 2026-03-06T15:56:29
---

# Reflection - Implement Remote Linux Diagnostics

## Knowledge

### 1vyf2b000: Guard Remote Launches Before Local Preflight
| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When a CLI exposes both local-only runtime checks and partially implemented remote/cloud host targets. |
| **Insight** | Remote-provider launch requests should be rejected before Linux-local preflight runs, otherwise missing local prerequisites can hide the real provider-specific support boundary. |
| **Suggested Action** | Resolve the target machine and host first, return provider-aware guidance for remote lanes, and run `/dev/kvm` or local-binary checks only for actual local-launch paths. |
| **Applies To** | `crates/port-runtime/src/lib.rs`, launch guards, future remote control lanes |
| **Linked Knowledge IDs** | |
| **Observed At** | 2026-03-06T23:57:00Z |
| **Score** | 0.84 |
| **Confidence** | 0.95 |
| **Applied** | yes |

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vyf1x000: Title
| Field | Value |
|-------|-------|
| **Category** | code/testing/process/architecture |
| **Context** | describe when this applies |
| **Insight** | the fundamental discovery |
| **Suggested Action** | what to do next time |
| **Applies To** | file patterns or components |
| **Linked Knowledge IDs** | optional canonical IDs this insight builds on |
| **Observed At** | RFC3339 timestamp (e.g. 2026-02-22T12:00:00Z) |
| **Score** | 0.0-1.0 (impact significance) |
| **Confidence** | 0.0-1.0 (insight quality) |
| **Applied** | |
-->

## Observations

- Provider-aware checks fit cleanly into `port doctor` as non-blocking host checks, which keeps the local Linux preflight intact while still exposing the cloud support boundary at the canonical CLI surface.
- The launch path needed to resolve the target machine and host before running local preflight, otherwise a remote AWS machine could fail on local prerequisites instead of returning the intended provider-specific guidance.
- Using the built `/tmp/port-target/debug/port` binary for CLI proofs kept verification close to the actual product surface without relying on another nested `nix develop` inside `keel story record`.
