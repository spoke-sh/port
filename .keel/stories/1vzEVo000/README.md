---
id: 1vzEVo000
title: Route Hosted CLI And SDK Through Live Transport
type: feat
status: in-progress
created_at: 2026-03-08T06:49:40
updated_at: 2026-03-08T07:13:13
scope: 1vzETR000/1vzETX000
started_at: 2026-03-08T07:13:13
---

# Route Hosted CLI And SDK Through Live Transport

## Summary

Route hosted CLI and SDK operations through the live control-plane transport
instead of the current in-process hosted runtime-root shortcut.

## Acceptance Criteria

<!-- verify: manual, SRS-03:start:end, proof: ac-1.log, ac-2.log -->
- [ ] [SRS-03/AC-01] Hosted `port machine ...` and `port guest ...` commands execute through the live hosted HTTP path whenever a machine resolves to `hosted-control-plane` mode. <!-- [SRS-03/AC-01] verify: cargo test -q -p port-cli, proof: ac-2.log -->
- [ ] [SRS-04/AC-02] `port-sdk`, CLI help, and operator output align with the live hosted routes and distinguish shipped transport from still-planned follow-on behavior. <!-- [SRS-04/AC-02] verify: cargo test -q -p port-sdk && cargo test -q -p port-cli, proof: ac-2.log -->
