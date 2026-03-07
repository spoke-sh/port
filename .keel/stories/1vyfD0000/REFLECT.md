---
created_at: 2026-03-06T16:10:46
---

# Reflection - Fix Help Example Guidance

## Knowledge

- [1vyfFY000](../../knowledge/1vyfFY000.md) Say When Help Examples Depend On Repo Context

## Observations

- The examples themselves were not syntactically wrong; the breakage came from implicit assumptions about running from the repository root and having `firecracker` plus the artifact-build tools on `PATH`.
- `port doctor` was already the correct preflight gate, so the most effective fix was to move that gate into the help text rather than inventing a new command or compatibility shim.
- Re-running the full help-published workflow in `nix develop` was important because it proved the examples were honest after the wording change instead of only looking better in static text.
