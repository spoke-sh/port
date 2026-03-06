---
created_at: 2026-03-06T15:42:35
---

# Reflection - Document Operator Workflows

## Knowledge

- [1vyeP0000](../../knowledge/1vyeP0000.md) Anchor Platform Guidance On `port doctor`

## Observations

- The docs were most coherent once the Linux local-launch workflow and the separate runtime-socket guest workflow were written as distinct supported paths instead of being blended together.
- Updating `port --help` and `port doctor` alongside the README avoided a mismatch where the prose would promise a platform story the CLI did not surface.
- The artifact story changed the truth of the guest-image path, so the operator-doc pass was the right place to catch and fix that drift before it became institutionalized in the README.
