---
created_at: 2026-05-03T17:34:18
---

# Reflection - Use K3s 1.35.4 From Nixpkgs

## Knowledge

<!--
Link existing knowledge files when the insight already exists:
- [123abcDEF](../../knowledge/123abcDEF.md) Existing knowledge title

Capture only novel/actionable knowledge that is likely useful in future work as
an inline candidate block. Unique entries are promoted into `.keel/knowledge/<id>.md`
on submit/accept.

If there is no reusable insight for this story, leave the Knowledge section empty.
Format:
### VIbhnvc9w: Title
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

The current `nixos-unstable` lock and the latest `nixos-unstable` tip still
resolve `pkgs.k3s.version` to `1.35.2+k3s1`, while the merged nixpkgs PR
515339 resolves the package to `1.35.4+k3s1`. To avoid a K3s-only nixpkgs
input, the primary nixpkgs input now points at the PR merge revision.

`nix develop --command k3s --version` realizes the updated K3s closure and is
the direct proof that the dev shell exposes the requested binary.
