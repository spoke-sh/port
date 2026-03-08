---
created_at: 2026-03-07T20:15:00
---

# Reflection - Gate Linux Only Dev Shell Inputs

## Knowledge

## Observations

- Cross-platform Nix verification is cheap enough to run from Linux by
  evaluating the Darwin dev-shell derivation directly. That is a good way to
  catch unsupported package assumptions without waiting for a real macOS host.
- Port's default shell needed a clearer separation between repo tooling and
  Linux-only runtime tooling. Making that split explicit in `flake.nix` is more
  maintainable than scattering `isLinux` checks around individual entries.
- The operator story is better when the shell tells the truth on macOS:
  entering a valid repo shell is useful, but it should not imply local
  Firecracker launch is available there today.
