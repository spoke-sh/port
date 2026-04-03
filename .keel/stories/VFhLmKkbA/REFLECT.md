---
created_at: 2026-04-02T21:36:39
---

# Reflection - Ship AWS PVM Nix Host Kit Surface

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### VFhQWXTqJ: Title
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

- Port's existing AWS PVM contract already lived in `examples/port.toml` and the
  runtime model, so the safest Nix design was to import and assert against that
  data rather than restating the host-kit identity in a second handwritten
  source of truth.
- Exporting both a NixOS module and a companion package was the cleanest
  downstream seam because `infra` currently needs an absolute module path while
  operators still need a packaged `firecracker-pvm` and manifest surface.
- The repo still does not carry an in-tree patched PVM kernel or Firecracker
  derivation, so the module had to make those concrete builds explicit override
  points instead of pretending Port now owns downstream AMI orchestration end
  to end.
- `keel story accept` can still stamp `submitted_at` and `completed_at` with
  the same second, which trips `keel doctor`; accepted stories need a quick
  frontmatter sanity check until that tooling bug is fixed.
