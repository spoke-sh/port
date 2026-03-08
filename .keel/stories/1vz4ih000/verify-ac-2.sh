#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

echo "== hosted follow-on boundary docs =="
rg -n 'follow-on order after this foundation|downstream of the authenticated API|not already shipped' \
  docs/hosted.md

echo
echo "== hosted guest boundary context =="
sed -n '284,296p' docs/hosted.md

echo
echo "== board root summary =="
sed -n '1,120p' .keel/README.md
