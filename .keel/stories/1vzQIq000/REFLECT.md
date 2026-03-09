---
created_at: 2026-03-08T19:31:35
---

# Reflection - Define Hosted Detached Forward Contract

## Knowledge

- [1vzQKh000](../../knowledge/1vzQKh000.md) Keel Story Record Proof Mapping Can Drift Across Same-SRS ACs

## Observations

The contract slice stayed narrow and that was the right call. Adding explicit
detached-forward routes plus `forward_name` to the shared hosted route context
was enough to unblock the later runtime stories without dragging the control
plane implementation into this commit.

The main surprise was that the upgraded `keel` verifier stayed correct while
`keel story record` still rewrote AC1's proof comment to `ac-2.log` and left
both AC checkboxes unchecked. Manual README inspection remains mandatory before
submit when multiple ACs map to the same SRS requirement.
