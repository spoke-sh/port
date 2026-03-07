---
created_at: 2026-03-06T17:31:25
---

# Reflection - Rework Copy And Forward For Live Guest Transport

## Knowledge

- [1vygVp000](../../knowledge/1vygVp000.md) Preserve buffered bytes when switching from framed control to raw streams

## Observations

- The runtime already had the right architectural split: `exec`/`pty`/`logs`
  stayed on the existing one-shot path, while `copy` and `forward` needed
  dedicated streaming flows. The user-facing failure came from the CLI not
  switching over to those runtime helpers.
- Live proof mattered. The automated tests caught protocol shape and handshake
  behavior, but the real VM run exposed a misleading guest-to-host success
  message and the undocumented requirement that the sample guest needs loopback
  brought up before `forward --target 127.0.0.1:...` can work.
- The forward transport had one subtle correctness bug even after the feature
  looked implemented: the first guest bytes after the `Accepted` frame could be
  dropped at the runtime handoff. Adding a targeted regression test was worth
  it because that defect would have been easy to reintroduce later.
