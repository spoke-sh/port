# Export AWS PVM Host Kit As A First-Class Nix Module - Decision Log

<!-- Append entries below. Each entry is an H2 with ISO timestamp. -->
<!-- Use `keel mission digest` to compress older entries when this file grows large. -->

## 2026-04-02T21:37:45

Exported the AWS x86_64 PVM host-kit contract as flake surfaces: nixosModules.aws-pvm-host plus packages.<system>.firecracker-pvm-host-kit; verified module evaluation, package metadata/path alignment, and downstream AMI handoff docs in README, docs/aws.md, docs/pvm.md, and website/docs/path-to-production/aws.mdx.

## 2026-04-02T21:37:45

Mission achieved by local system user 'alex'

## 2026-04-06T20:22:36

Re-ran the downstream module evaluation, inspected the derived host config and exported firecracker-pvm host-kit metadata, reviewed the downstream AMI handoff and explicit scope boundaries in docs/planning artifacts, and recorded mission-level verification.cast plus verification.gif proof.
