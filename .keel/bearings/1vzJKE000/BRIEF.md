# Executable Pvm And Avf Lanes — Brief

## Hypothesis

Port now has enough control-plane, artifact, and guest-agent foundation to stop
treating executable PVM and AVF lanes as distant futures. The next meaningful
step is to split them into two delivery programs:

- an x86_64 Firecracker/PVM host-kit and hosted-launch program for
  cost-controlled Linux fleets
- a first real Apple Virtualization Framework runtime for macOS operators

That matters because the user objective is broader than the finished board:
Port still lacks a real hosted launch path on prepared Linux nodes and still
lacks a first-class executable macOS substrate.

## Problem Space

The current board exhausted the first PVM and hosted-control slices, but the
product goal is still incomplete. Port can now:

- model x86_64 PVM host-kit requirements
- gate hosted PVM placement by node readiness
- document AVF as a first-class planned lane

Port still cannot:

- build or ship the x86_64 PVM host kit itself
- launch hosted Linux VMs through node agents on prepared hosts
- run a real AVF-backed `machine launch` and guest workflow on macOS

We need the next executable split, not more placeholder scope.

## Success Criteria

How will we know if this research was valuable?

- [x] The research ends with a concrete keep/split recommendation for x86_64
  PVM, arm64 cost-control execution, and AVF.
- [x] The outcome names the next epics or voyages needed to resume execution
  immediately instead of leaving the board empty.

## Open Questions

- Should Port pursue arm64 cost control through Firecracker/PVM, or through
  standard virtualization on native arm64 hosts?
- What is the smallest shippable x86_64 PVM program beyond the current
  admission and docs foundation?
- What is the smallest real AVF program that preserves Port's canonical
  lifecycle and guest verbs on macOS?
