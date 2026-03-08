#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

echo "== cargo test -q -p port-model =="
cargo test -q -p port-model

echo
echo "== cargo test -q -p port-model sample_config_derives_hosted_machine_lifecycle_contracts =="
cargo test -q -p port-model sample_config_derives_hosted_machine_lifecycle_contracts

echo
echo "== hosted lifecycle contracts in port-model =="
rg -n "hosted_machine_summary_contract|hosted_machine_status_contract|hosted_machine_stop_contract|HostedMachineSummaryContract|HostedMachineStatusContract|HostedMachineStopContract|sample_config_derives_hosted_machine_lifecycle_contracts" crates/port-model/src/lib.rs
