# Ship Cargo-Dist Release And Upgrade Path - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Ship a Port release contract that publishes cargo-dist archives and installers for Port's supported targets, and expose a `port upgrade` workflow that can consume those release installers or install a locally built revision by tag or git SHA. | board: VFdgQWhbn |

## Constraints

- Mirror the Keel and Sift cargo-dist release shape closely enough that Port uses the same tag-driven GitHub release flow, installer generation, and release validation vocabulary.
- Preserve Port's actual support matrix. Do not advertise a release target that the current codebase cannot build or operate correctly.
- Keep `port doctor` as the first post-install verification step for shipped binaries.
- Preserve Port's packaged runtime asset resolution so install paths continue to find the bundled docs, scripts, and examples needed by CLI workflows.

## Halting Rules

- DO NOT halt while the epic attached to this mission still has unplanned or unfinished work required to ship the release-and-upgrade contract.
- HALT when epic `VFdgQWhbn` is complete, release and installer documentation match the shipped behavior, and the `port upgrade` path is verified with automated evidence.
- YIELD to the human only if the supported release target matrix must change beyond the current Linux and macOS contract, or if an external hosting decision blocks installer publication.
