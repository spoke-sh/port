---
created_at: 2026-03-09T00:20:11
---

# Knowledge - 1vzTR9000

> Automated synthesis of story reflections.

## Story Knowledge

## Story: Implement Node Agent Registration Refresh (1vzTTI000)

### 1vzU8M000: Hosted node-agent tests must bootstrap the control plane first

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | Any hosted test or proof that starts `port node-agent serve` after the eager registration contract landed |
| **Insight** | The node agent now fails before listening unless its configured control-plane endpoint is already reachable and the control-plane auth env var is present. Older hosted fixtures that started node agents before the control plane or omitted the token now fail for the right reason. |
| **Suggested Action** | In hosted fixtures, start the control plane first, set the control-plane token env var for the node agent, and isolate or clean `.port/hosted/<control-plane>` registration state between tests. |
| **Applies To** | `crates/port-cli/tests/*.rs`, hosted runtime integration helpers, future hosted proof scripts |
| **Applied** | yes |



---

## Synthesis

### PPcugqyz6: Hosted node-agent tests must bootstrap the control plane first

| Field | Value |
|-------|-------|
| **Category** | testing |
| **Context** | Any hosted test or proof that starts `port node-agent serve` after the eager registration contract landed |
| **Insight** | The node agent now fails before listening unless its configured control-plane endpoint is already reachable and the control-plane auth env var is present. Older hosted fixtures that started node agents before the control plane or omitted the token now fail for the right reason. |
| **Suggested Action** | In hosted fixtures, start the control plane first, set the control-plane token env var for the node agent, and isolate or clean `.port/hosted/<control-plane>` registration state between tests. |
| **Applies To** | `crates/port-cli/tests/*.rs`, hosted runtime integration helpers, future hosted proof scripts |
| **Linked Knowledge IDs** | 1vzU8M000 |
| **Score** | 0.93 |
| **Confidence** | 0.98 |
| **Applied** | yes |

