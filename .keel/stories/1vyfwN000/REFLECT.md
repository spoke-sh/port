---
created_at: 2026-03-06T17:10:51
---

# Reflection - Connect Exec Pty And Logs To Live VMs

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### 1vygBv000: Title
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

The transport code itself landed quickly once the host-side Firecracker tunnel
shape was pinned down. The unexpected failures were both in the guest boot
path: the rootfs needed `init=/init`, and the init script needed idempotent
mounts because `/dev` was already mounted when the kernel handed off control.

The most useful live proof was the combined launch plus `guest exec`/`pty`/`logs`
sequence against a rebuilt image. It proved the full chain end to end:
boot args, guest init, guest-agent vsock listener, Firecracker tunnel, runtime
transport selection, and CLI rendering.
