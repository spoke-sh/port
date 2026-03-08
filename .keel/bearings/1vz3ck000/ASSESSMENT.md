---
id: 1vz3ck000
---

# Assessment - PVM And Multi-Substrate Execution

## Recommendation

Keep Firecracker/PVM for `x86_64` as a strategic execution lane. Keep Apple
Virtualization Framework as a first-class macOS substrate lane. Drop arm64
Firecracker/PVM from near-term implementation scope and keep it research-only
until there is stronger evidence than "arm64 hardware exists elsewhere".

Port should immediately plan the next implementation work around substrate
drivers and host/runtime ownership, not around more support-matrix prose.

## Decision Matrix

| Topic | Recommendation | Why |
|-------|----------------|-----|
| Firecracker/PVM on `x86_64` | Keep | Strongest path to cloud cost control on hosts without nested virtualization; external evidence shows a real, if custom, lane |
| Firecracker/PVM on `aarch64` | Keep as research only | Current reviewed evidence does not justify a supportable Port runtime claim |
| Native arm execution on arm hardware | Keep | Valid cost/performance lane, but different from PVM |
| AVF on macOS | Keep as first-class planned implementation lane | Real operator need, real substrate support, and already proven by adjacent products |
| More provider-only planning | Drop | Provider tokens do not solve runtime ownership, transport, or host-kit requirements |

## Immediate Implications For Port

1. Port needs a substrate driver boundary in the runtime.
2. Port needs a node-agent-oriented lifecycle contract above that boundary for
   hosted operation.
3. Port needs an x86_64 PVM host-kit plan covering kernel, Firecracker, and
   artifact variants.
4. Port needs an AVF-specific implementation plan for macOS operators.
5. Port should stop implying that arm64 Firecracker/PVM is merely "one story
   away" from shipping.

## Proposed Next Planning Unit

Create an epic focused on productized execution backends and hosted runtime
ownership, then decompose a first voyage with these stories:

- introduce a substrate driver interface in `port-runtime`,
- define a hosted node-agent API and local/remote machine inventory model,
- plan and scaffold the x86_64 PVM host kit and artifact variants,
- plan the AVF macOS driver lane and guest transport mapping,
- expand the CLI toward local-or-hosted lifecycle surfaces rather than
  runtime-root-only commands.

## Keep / Drop Summary

- Keep:
  x86_64 PVM, AVF, hosted node-agent architecture, native arm execution on real
  arm hosts
- Drop from near-term promises:
  arm64 Firecracker/PVM implementation claims
- Reframe:
  cloud cost control as a combination of prepared x86 PVM hosts, native-capable
  KVM hosts, and arm hardware lanes rather than one universal virtualization
  trick
