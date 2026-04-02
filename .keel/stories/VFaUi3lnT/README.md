---
# system-managed
id: VFaUi3lnT
status: needs-human-verification
created_at: 2026-04-01T17:09:22
updated_at: 2026-04-01T17:17:57
# authored
title: Publish Docusaurus User Docs For Port
type: feat
operator-signal:
started_at: 2026-04-01T17:10:02
submitted_at: 2026-04-01T17:17:57
---

# Publish Docusaurus User Docs For Port

## Summary

Create a public Docusaurus site under `website/` for Port, wire repo-level docs
workflows, and publish narrative MDX tracks for local adoption, cloud-oriented
path-to-production guidance, and Linux/macOS/Windows host expectations without
overstating current Port runtime support.

## Acceptance Criteria

- [x] [SRS-01/AC-01] Port ships a Docusaurus site under `website/` with repo-supported `just docs-*` workflows and a branded landing page. <!-- verify: manual, SRS-01:start:end -->
- [x] [SRS-02/AC-01] The site publishes narrative MDX tracks for local adoption, a path-to-production overview, and provider-specific AWS, GCP, and Azure guidance that matches the current Port support boundaries. <!-- verify: manual, SRS-02:start:end -->
- [x] [SRS-03/AC-01] The site publishes host-platform guidance for Linux, macOS, and Windows and links readers back to the canonical root contracts for deeper reference. <!-- verify: manual, SRS-03:start:end -->
