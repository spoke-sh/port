---
source_type: Story
source: stories/VFDk8ggoV/REFLECT.md
scope: VFDhlRjOf/VFDk8fdnG
source_story_id: VFDk8ggoV
created_at: 2026-03-29T08:33:00
---

### VFG3hLr2M: Proof scripts must honor Cargo target indirection

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | Renderer-backed proof scripts that build binaries inside `nix develop` |
| **Insight** | The dev shell can redirect Cargo outputs through `CARGO_TARGET_DIR`, so proof scripts that hardcode `./target/debug/...` can execute stale binaries even after a successful build. |
| **Suggested Action** | Resolve built binary paths from `$CARGO_TARGET_DIR` with a fallback to the repo `target` directory, or use `cargo run` when the executable path must follow the active shell contract. |
| **Applies To** | `scripts/render-*.sh` |
| **Linked Knowledge IDs** |  |
| **Observed At** | 2026-03-29T15:33:22+00:00 |
| **Score** | 0.84 |
| **Confidence** | 0.98 |
| **Applied** | `scripts/render-local-cluster-proof.sh` now resolves `port` and `port-guest-agent` from the active Cargo target root. |
