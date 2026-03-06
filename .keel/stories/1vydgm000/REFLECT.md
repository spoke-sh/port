---
created_at: 2026-03-06T15:42:35
---

# Reflection - Document Operator Workflows

## Knowledge

### 1vyeP0000: Anchor Platform Guidance On `port doctor`
| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | When documenting or exposing macOS and Windows operator workflows for a Linux-only runtime |
| **Insight** | The most stable support contract is to document the intended workflow and then use `port doctor` as the runtime gate instead of promising environment capabilities that vary across hosts, especially in WSL-backed setups |
| **Suggested Action** | Keep README, platform docs, and CLI help centered on the exact `port doctor` boundary when platform support depends on Linux host capabilities |
| **Applies To** | README, `docs/operators.md`, CLI help text, diagnostics |
| **Linked Knowledge IDs** | |
| **Observed At** | 2026-03-06T23:43:00Z |
| **Score** | 0.74 |
| **Confidence** | 0.91 |
| **Applied** | yes |

## Observations

- The docs were most coherent once the Linux local-launch workflow and the separate runtime-socket guest workflow were written as distinct supported paths instead of being blended together.
- Updating `port --help` and `port doctor` alongside the README avoided a mismatch where the prose would promise a platform story the CLI did not surface.
- The artifact story changed the truth of the guest-image path, so the operator-doc pass was the right place to catch and fix that drift before it became institutionalized in the README.
