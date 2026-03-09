---
source_type: Story
source: stories/1vzQJB000/REFLECT.md
scope: 1vzETR000/1vzQEj000
source_story_id: 1vzQJB000
created_at: 2026-03-08T19:46:33
---

### 1vzQLp000: Bogus Client Runtime Roots Are A Strong No-Fallback Hosted CLI Proof

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | Verifying that hosted CLI commands no longer read repo-local runtime state after moving to the live control-plane and node-agent path |
| **Insight** | If the client config points the hosted node runtime root at a bogus path while the server-side config keeps the real runtime root, any successful hosted command proves the CLI is using remote transport rather than local state inspection. |
| **Suggested Action** | Keep using split server/client hosted configs with a bogus client runtime root in CLI integration tests for hosted transport stories. |
| **Applies To** | `crates/port-cli/tests/*`, hosted machine and guest transport tests |
| **Linked Knowledge IDs** |  |
| **Observed At** | 2026-03-08T19:46:33+00:00 |
| **Score** | 0.86 |
| **Confidence** | 0.97 |
| **Applied** | yes |
