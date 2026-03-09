---
created_at: 2026-03-08T19:27:01
---

# Knowledge - 1vzMVY000

> Automated synthesis of story reflections.

## Story Knowledge

## Story: Implement Hosted Streamed Forward Transport (1vzMY2000)

### 1vzMY2000: Hosted forward ownership can hide behind local listener setup

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When a hosted guest capability has a working node-agent route but the CLI/runtime still reuses a local helper such as `prepare_guest_forward`. |
| **Insight** | The hosted control-plane path can be functionally live while the canonical CLI still bypasses it and silently falls back to local runtime assumptions. Forward ownership broke specifically because the CLI kept constructing a local session instead of entering the hosted `guest:forward` route. |
| **Suggested Action** | Add hosted-path tests that use a bogus client-side runtime root and require the control-plane/node-agent route to succeed or fail with hosted route context. |
| **Applies To** | `crates/port-runtime/src/lib.rs`, `crates/port-cli/src/lib.rs`, hosted guest capability tests |
| **Applied** | yes |



---

## Story: Publish Streamed Guest Workflow Surface (1vzMXM000)

### 1vzMXM000: Workflow-surface stories need proof that matches the published wording

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | When a story is mostly CLI/help/docs work instead of a deep runtime change. |
| **Insight** | Doc-only acceptance is still fragile unless the proof scripts check the exact published keywords and pair them with executable workflow tests. The fastest way to keep these stories honest was to combine `rg`-based surface checks with targeted CLI/runtime tests for the workflows named in the docs. |
| **Suggested Action** | For future workflow-surface stories, write verify scripts that inspect the text and replay the referenced commands before submit. |
| **Applies To** | `.keel/stories/*/verify-ac-*.sh`, CLI help text, README and docs updates |
| **Applied** | yes |



---

## Synthesis

### tBtL4hzE6: Hosted forward ownership can hide behind local listener setup

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | When a hosted guest capability has a working node-agent route but the CLI/runtime still reuses a local helper such as `prepare_guest_forward`. |
| **Insight** | The hosted control-plane path can be functionally live while the canonical CLI still bypasses it and silently falls back to local runtime assumptions. Forward ownership broke specifically because the CLI kept constructing a local session instead of entering the hosted `guest:forward` route. |
| **Suggested Action** | Add hosted-path tests that use a bogus client-side runtime root and require the control-plane/node-agent route to succeed or fail with hosted route context. |
| **Applies To** | `crates/port-runtime/src/lib.rs`, `crates/port-cli/src/lib.rs`, hosted guest capability tests |
| **Linked Knowledge IDs** | 1vzMY2000 |
| **Score** | 0.86 |
| **Confidence** | 0.96 |
| **Applied** | yes |

### g3TRMUVos: Workflow-surface stories need proof that matches the published wording

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | When a story is mostly CLI/help/docs work instead of a deep runtime change. |
| **Insight** | Doc-only acceptance is still fragile unless the proof scripts check the exact published keywords and pair them with executable workflow tests. The fastest way to keep these stories honest was to combine `rg`-based surface checks with targeted CLI/runtime tests for the workflows named in the docs. |
| **Suggested Action** | For future workflow-surface stories, write verify scripts that inspect the text and replay the referenced commands before submit. |
| **Applies To** | `.keel/stories/*/verify-ac-*.sh`, CLI help text, README and docs updates |
| **Linked Knowledge IDs** | 1vzMXM000 |
| **Score** | 0.74 |
| **Confidence** | 0.94 |
| **Applied** | yes |

