---
created_at: 2026-03-06T14:59:11
---

# Reflection - Implement Local Firecracker Launch

## Knowledge

### 1vye8L000: Firecracker 1.14 Uses `smt` In `machine-config`
| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Generating Firecracker JSON config files for `--config-file` launch on Firecracker 1.14.x |
| **Insight** | Current Firecracker 1.14 rejects the older `machine-config.ht_enabled` field and expects `machine-config.smt` instead. Using the older field fails fast during JSON parsing before the microVM starts. |
| **Suggested Action** | Match generated config fields to the live Firecracker binary in the dev shell and confirm with an executable launch proof before trusting older examples. |
| **Applies To** | `crates/port-runtime/*`, Firecracker config generation, local launch proofs |
| **Linked Knowledge IDs** | |
| **Observed At** | 2026-03-06T22:58:00Z |
| **Score** | 0.88 |
| **Confidence** | 0.97 |
| **Applied** | yes |

## Observations

- Pulling real demo artifacts from the official Firecracker ecosystem was enough
  to prove the CLI launch path before the in-repo artifact pipeline exists.
- The launch story needed both success-path and failure-path proof. The failure
  proof was easiest to make deterministic by pointing at the checked-in example
  config, which still references placeholder artifact paths.
- `keel story record` counts SRS phase markers in its AC index. Once
  `verify:SRS-XX:start:end` markers are present, later proof recordings need to
  account for those extra slots or a proof will attach to the wrong target.
