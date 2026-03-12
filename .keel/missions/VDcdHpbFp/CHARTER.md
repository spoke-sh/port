# Ship Installable Linux And Mac Developer Experience - Charter

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Convert bearing VDcT0vaPb into executable board work for an installable Linux and macOS developer experience. | board: VDcT0vaPb |
| MG-02 | Keep the mission surfaces doctor-clean and make this installable developer-experience mission visible through the board command surfaces. | manual: run `just keel mission show VDcdHpbFp`, `just flow`, and `just keel doctor` |

## Constraints
- Keep one canonical `port` CLI and guest model across Linux and macOS, use VDcT0vaPb plus `RELEASE.md`, `README.md`, and `docs/avf.md` as the primary sources, and avoid broadening scope into new runtime substrates or separate platform-specific toolchains.
- Preserve the hard-cutover policy: do not add compatibility aliases or dual-surface operator paths just to ease packaging.

## Halting Rules

- DO NOT halt while any MG-* goal has unfinished board work
- HALT when the installable developer-experience work is decomposed into executable board items and only verification-oriented manual checks remain
- YIELD to human when prioritization depends on external signing, packaging-channel, or distribution decisions that are not inferable from the repository
