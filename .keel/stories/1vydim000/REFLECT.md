---
created_at: 2026-03-06T14:59:11
---

# Reflection - Implement Local Firecracker Launch

## Knowledge

- [1vye8L000](../../knowledge/1vye8L000.md) Firecracker 1.14 Uses `smt` In `machine-config`

## Observations

- Pulling real demo artifacts from the official Firecracker ecosystem was enough
  to prove the CLI launch path before the in-repo artifact pipeline exists.
- The launch story needed both success-path and failure-path proof. The failure
  proof was easiest to make deterministic by pointing at the checked-in example
  config, which still references placeholder artifact paths.
- `keel story record` counts SRS phase markers in its AC index. Once
  `verify:SRS-XX:start:end` markers are present, later proof recordings need to
  account for those extra slots or a proof will attach to the wrong target.
