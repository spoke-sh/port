---
id: 1vz4gb000
title: Define Hosted Auth And API Contract
type: feat
status: backlog
created_at: 2026-03-07T19:20:09
updated_at: 2026-03-07T19:22:36
scope: 1vz4Yn000/1vz4cU000
---

# Define Hosted Auth And API Contract

## Summary

Define the first hosted control-plane endpoint and token-auth contract so Port
can model authenticated hosted targets without inventing a second operator
surface.

## Acceptance Criteria

- [ ] [SRS-01/AC-01] Port publishes implementation-ready hosted endpoint and token-auth contracts in the shared model, including how hosted API identity maps onto the canonical CLI target surface.
- [ ] [SRS-01/AC-02] README, hosted docs, and CLI help describe the hosted auth contract and clearly distinguish the modeled control-plane path from shipped local behavior.
