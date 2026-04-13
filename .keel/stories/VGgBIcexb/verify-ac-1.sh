#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

expected_rev="65af71bb72f871fcd7249913a9580d8cfb1fbf2b"

echo "== locked keel revision =="
nix eval --impure --raw --expr '
  let
    lock = builtins.fromJSON (builtins.readFile /home/alex/workspace/spoke-sh/port/flake.lock);
  in
    lock.nodes.keel.locked.rev
'

echo
echo "== verify keel follow edges =="
nix eval --impure --raw --expr '
  let
    lock = builtins.fromJSON (builtins.readFile /home/alex/workspace/spoke-sh/port/flake.lock);
  in
    if lock.nodes.root.inputs.keel == "keel"
      && lock.nodes.atxt.inputs.keel == [ "keel" ]
      && lock.nodes.paddles.inputs.keel == [ "keel" ]
    then
      "follow-edges-ok"
    else
      throw "keel follow edges drifted"
'

actual_rev="$(
  nix eval --impure --raw --expr '
    let
      lock = builtins.fromJSON (builtins.readFile /home/alex/workspace/spoke-sh/port/flake.lock);
    in
      lock.nodes.keel.locked.rev
  '
)"

test "$actual_rev" = "$expected_rev"

