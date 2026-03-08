---
created_at: 2026-03-07T20:00:00
---

# Reflection - Define Hosted Auth And API Contract

## Knowledge

## Observations

- Treating remote provider hosts as explicit hosted control-plane targets is a
  better contract than keeping SSH-shaped placeholders in the model. It makes
  the future Port product surface visible without pretending remote launch is
  already implemented.
- The first hosted auth slice needed to show up in three places together:
  shared model types, CLI/runtime guidance, and operator docs. Leaving any one
  of those out would have made the contract either invisible or misleading.
- `port doctor` is a good discovery surface for hosted intent because it can
  surface endpoint, audience, and token-source contracts without mutating any
  runtime state.
