---
created_at: 2026-03-07T17:32:52
---

# Reflection - Model Substrates And Protection Modes

## Knowledge

- [1vz3A9000](../../knowledge/1vz3A9000.md) Separate Architecture From Protection-Mode Support

## Observations

- The new model landed cleanly once the scope stayed narrow: machine-level
  substrate/protection/architecture plus artifact compatibility metadata was
  enough to express the new support matrix without prematurely introducing a
  full runtime-driver abstraction.
- Runtime diagnostics were the right first enforcement point. Adding lane-aware
  checks to `port doctor` and launch-time contract validation gives operators
  actionable failures before later lifecycle or hosted work lands.
- Documentation drift was a real risk. README and `docs/cloud.md` still carried
  the old “PVM dropped” story, so this slice had to update the docs in the same
  change or the board would have contradicted the implementation direction.
- `keel story record --cmd` appears to leave long-lived recorder processes in
  this environment. Recording proof through generated log files plus annotated
  acceptance criteria was reliable and should be reused if the direct command
  path keeps hanging.
