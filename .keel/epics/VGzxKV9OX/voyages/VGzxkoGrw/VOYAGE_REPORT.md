# VOYAGE REPORT: Guest-Agent Heartbeat And Age Surface

## Voyage Metadata
- **ID:** VGzxkoGrw
- **Epic:** VGzxKV9OX
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Add Ping Frame And Guest-Agent Heartbeat Wire Contract
- **ID:** VGzxv3FOx
- **Status:** done

#### Summary
Introduce the minimal wire-level contract that lets the node-agent prove a guest-agent is awake without piggybacking on `Exec` or any streamed operation. Add a `Ping` request frame and a `Pong` response (or equivalent envelope pair) to `port-agent-protocol`, and wire a handler in `port-guest-agent` that responds immediately on its existing read loop. This story owns only the protocol and the guest-side handler; the node-agent-side probe loop lives in a follow-on story.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] `port-agent-protocol` defines the `Ping`/`Pong` frame pair and round-trips through serde; `port-guest-agent`'s read loop matches `Ping` and writes `Pong` without touching running managed services or PTY sessions, with a documented, observable response budget. <!-- [SRS-01/AC-01] verify: cargo test -p port-agent-protocol -p port-guest-agent, proof: ac-1.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/VGzxv3FOx/EVIDENCE/ac-1.log)

### Drive Periodic Guest Heartbeat Probe From Node-Agent
- **ID:** VGzyLJtZw
- **Status:** done

#### Summary
Once the wire contract exists, the node-agent has to actually issue pings on a timer and record the result. Spawn a per-machine probe task on registration, issue `Ping` on the existing transport with a bounded interval, stamp `guest_agent_last_heartbeat` in a per-machine in-memory sidecar on successful `Pong`, and leave the prior timestamp untouched on any failure (timeout, decode error, transport close). The probe must run independently of in-flight guest operations so a long `Exec` or `Pty` does not mask a wedged guest-agent.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] The node-agent spawns a per-machine probe task on registration and cancels it on deregistration; successful pongs update a `RwLock<BTreeMap<String, Instant>>` sidecar keyed by machine name, and failed probes leave the prior timestamp untouched. <!-- [SRS-02/AC-01] verify: cargo test -p port-runtime -- guest_heartbeat_probe, proof: ac-2.log -->
- [x] [SRS-NFR-01/AC-01] The probe loop runs concurrently with long-running guest operations: a test keeps `Exec` or `Pty` open for longer than one probe interval and asserts the guest heartbeat still advances, proving the probe does not serialize behind in-flight ops. <!-- [SRS-NFR-01/AC-01] verify: cargo test -p port-runtime -- guest_heartbeat_probe_runs_concurrently, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/VGzyLJtZw/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VGzyLJtZw/EVIDENCE/ac-2.log)

### Surface Guest Refresh Age Seconds In Cluster Status
- **ID:** VGzyLTlgJ
- **Status:** done

#### Summary
With the probe loop stamping heartbeats, extend the hosted cluster status contract to surface `guest_refresh_age_seconds: Option<u64>` per machine. Compute the age from the monotonic `Instant` sidecar so wall-clock jumps on the node-agent host cannot produce negative or inflated values. Thread the field through the existing node-agent → control-plane status path the same way `refresh_age_seconds` already flows, and confirm the existing guest-operation suites (`exec`, `copy`, `pty`, `logs`, `forward`, hosted round-trips) still pass with the probe loop running.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] `port cluster status --format json` and the machine status contract expose `guest_refresh_age_seconds: Option<u64>` per machine, `None` before the first successful pong and populated thereafter; an integration test asserts the field appears after a probe and increases monotonically across reads. <!-- [SRS-03/AC-01] verify: cargo test -p port-runtime -- guest_heartbeat_age, proof: ac-2.log -->
- [x] [SRS-04/AC-01] The existing hosted guest-operation test suites (including the `exec`, `copy`, `pty`, `logs`, `forward`, and hosted control-plane round-trip tests) continue to pass with the probe loop active. <!-- [SRS-04/AC-01] verify: cargo test -p port-runtime --lib, proof: ac-2.log -->
- [x] [SRS-NFR-02/AC-01] The age computation uses a monotonic clock source (`Instant::now()` saturating-subtracted from the sidecar entry); `guest_heartbeat_age_uses_monotonic_clock_and_is_non_regressive` asserts non-regression across reads. <!-- [SRS-NFR-02/AC-01] verify: cargo test -p port-runtime -- guest_heartbeat_age_uses_monotonic_clock, proof: ac-3.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/VGzyLTlgJ/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VGzyLTlgJ/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VGzyLTlgJ/EVIDENCE/ac-3.log)


