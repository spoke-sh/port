# Improve Operator Signal And Documentation Experience - Charter

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Complete epic `VDaiFfFPe` so Port has one concise mission-verification entrypoint, a simplified `just` surface, and foundational documentation that is fast to audit. | board: VDaiFfFPe |
| MG-02 | Leave the repository in a doctor-clean state with mission progress legible through `just mission` and linked board artifacts. | manual: run `just mission` and `just keel doctor` |

## Constraints

- Keep `nix develop -c just keel doctor` clean after structural board changes.
- Use `port` as the canonical user-facing command in docs and help; do not publish `cargo run -p port-cli` in user-facing examples.
- Keep default help surfaces concise: root `just` help should emphasize common workflows, and `port --help` should keep only a small set of high-value examples.
- Preserve access to lower-level or demo recipes without surfacing them in the default top-level help.

## Halting Rules

- DO NOT halt while epic `VDaiFfFPe` still has actionable planning or delivery work that can be progressed from local context.
- HALT when `board: VDaiFfFPe` is satisfied and only manual verification remains.
- YIELD to human when progress depends on external release infrastructure, unavailable credentials, or subjective documentation direction that cannot be inferred from local context.
