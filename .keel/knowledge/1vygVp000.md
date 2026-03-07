---
source_type: Story
source: stories/1vyfvx000/REFLECT.md
scope: 1vydg7000/1vyfve000
source_story_id: 1vyfvx000
created_at: 2026-03-06T17:31:25
---

### 1vygVp000: Preserve buffered bytes when switching from framed control to raw streams

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Control protocols that use a framed handshake and then hand the same duplex socket over to raw byte proxying |
| **Insight** | A `BufReader` can prefetch guest data that arrives immediately after the final framed response. Dropping the reader with `into_inner()` without draining `buffer()` silently loses those bytes. |
| **Suggested Action** | When handing a Port transport from framed JSON into raw copy/forward mode, either avoid buffered reads at the handoff point or wrap the underlying stream with a prefix reader that drains the buffered bytes first. |
| **Applies To** | `crates/port-runtime/src/lib.rs`, future guest transport/proxy code |
| **Linked Knowledge IDs** |  |
| **Observed At** | 2026-03-07T01:31:25+00:00 |
| **Score** | 0.91 |
| **Confidence** | 0.94 |
| **Applied** | yes |
