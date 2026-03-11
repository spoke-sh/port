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

## Context

This bearing started after the first hosted-control and substrate foundations
had landed, but before Port had any real executable PVM or macOS runtime lane.
The board needed a decision on how to split the next delivery program so Linux
cost-control work and macOS operator work could advance without diluting each
other.

## Objectives

- Decide whether the next execution work should split into separate Linux PVM
  and AVF programs.
- Define the smallest credible `x86_64` PVM host-kit and hosted-launch program.
- Define the smallest real AVF runtime program that still preserves Port's
  canonical lifecycle and guest verbs.

## Scope

- In scope: `x86_64` PVM host-kit delivery, prepared-node hosted launch, AVF
  runtime delivery on macOS, and arm64 cost-control positioning relative to
  those lanes.
- Out of scope: implementing the full host kit, claiming immediate arm64
  Firecracker/PVM support, or collapsing Linux and macOS runtime work back into
  one generic queue.

## Success Criteria

How will we know if this research was valuable?

- [x] The research ends with a concrete keep/split recommendation for x86_64
  PVM, arm64 cost-control execution, and AVF.
- [x] The outcome names the next epics or voyages needed to resume execution
  immediately instead of leaving the board empty.

## Research Questions

- Should Port pursue arm64 cost control through Firecracker/PVM, or through
  standard virtualization on native arm64 hosts?
- What is the smallest shippable `x86_64` PVM program beyond the current
  admission and docs foundation?
- What is the smallest real AVF program that preserves Port's canonical
  lifecycle and guest verbs on macOS?

## Open Questions

- What packaging boundary will make the first PVM host kit portable across
  prepared Linux nodes?
- How much hosted-control durability must land before prepared-node launch is
  worth productizing?
- Should the first AVF implementation use direct Rust bindings or a narrower
  helper boundary?
