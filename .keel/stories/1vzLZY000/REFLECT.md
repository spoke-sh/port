---
created_at: 2026-03-08T16:20:00
---

# Reflection - Publish Macos Avf Operator Workflow

## Knowledge

## Observations

- An executable substrate lane is not operator-ready until the checked-in sample
  model names a concrete machine for it. Adding `demo-avf` turned the AVF story
  from abstract contract prose into a command path that help text, docs, and
  CLI proofs could all reference consistently.
- CLI help assertions should verify durable phrases, not exact line layout.
  Clap wraps long examples, so the tests need to key off stable tokens such as
  `demo-avf`, `PORT_AVF_LAUNCHER`, and the boundary text rather than a single
  unwrapped command line.
- For cross-platform operator docs, the best proof is a paired surface:
  successful discovery commands like `port --help` and `port doctor`, plus an
  explicit failure-path proof that shows the non-macOS AVF boundary remains
  intentional and legible.
