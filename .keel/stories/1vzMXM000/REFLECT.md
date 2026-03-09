---
created_at: 2026-03-08T19:18:17
---

# Reflection - Publish Streamed Guest Workflow Surface

## Knowledge

- [1vzMXM000](../../knowledge/1vzMXM000.md) Workflow-surface stories need proof that matches the published wording

## Observations

- The strongest proof for this slice came from keeping the docs and the tests coupled. The CLI help keyword guard plus the three verify scripts made it straightforward to see whether the published workflows still matched reality.
- The hosted forward behavior needed extra wording discipline. The capability is now live, but hosted detached lifecycle management is not, so the docs had to describe the boundary explicitly instead of inheriting the local forward wording.
- `keel story record` still mis-associates proof links when multiple acceptance criteria share the same SRS prefix. I had to correct the story README manually again before closing the slice.
