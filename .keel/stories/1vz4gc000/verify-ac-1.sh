#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

echo "== cargo test -q -p port-model =="
cargo test -q -p port-model

echo
echo "== cargo test -q -p port-model sample_config_derives_hosted_guest_attach_contract =="
cargo test -q -p port-model sample_config_derives_hosted_guest_attach_contract

echo
echo "== hosted guest attach contracts in port-model =="
rg -n "hosted_guest_attach_contract|HostedGuestAttachContract|HostedGuestProtocolContract|HostedGuestAttachHop|HostedGuestAttachActor|GuestCommandVerb|sample_config_derives_hosted_guest_attach_contract" crates/port-model/src/lib.rs
