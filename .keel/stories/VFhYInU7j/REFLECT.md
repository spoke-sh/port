---
created_at: 2026-04-02T22:16:04
---

# Reflection - Fix AWS PVM Host Kit Kernel Default

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### VFhaRlCyo: Title
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

- The real downstream failure was best isolated by reading `infra` again rather
  than assuming it still used the older module-path env seam. It now imports
  `port.nixosModules.aws-pvm-host` directly from the flake input, so the
  correct local validation path is `--override-input port ...`.
- The broken behavior came from trying to encode the canonical host-kit kernel
  release into a stock `linux_6_12` derivation via `modDirVersion`. That made
  the package look right in metadata but guaranteed a build-time failure once a
  real system image tried to realize the kernel.
- Raw `path:/...` flake overrides against the Port checkout can still fail if
  repository runtime artifacts exist. A clean tracked-file mirror of the
  working tree was a reliable way to validate `infra` against unpublished Port
  changes without waiting for a commit first.
- `keel story accept` is still stamping `submitted_at` and `completed_at` with
  the same second in this repo, so accepted stories need the same quick
  timestamp sanity check before the final sealing commit.
