---
created_at: 2026-03-08T16:00:00
---

# Reflection - Wire Avf Guest Transport And Console Capture

## Knowledge

## Observations

- The cleanest AVF integration point in the current runtime is not a separate
  guest protocol. It is the same canonical runtime socket and log-path contract
  Port already uses elsewhere, with the launcher/helper owning the substrate
  translation to AVF virtio sockets and serial output.
- AVF forward verification needs bounded connection handling in tests. Reusing
  the production listener loop directly caused a hung proof because the loop is
  intentionally open-ended for real forwarding sessions.
- Canonical runtime metadata matters for operator parity. Writing the AVF guest
  socket and console log into launch metadata made `machine status`, `monitor`,
  and future workflow docs much easier to keep coherent.
