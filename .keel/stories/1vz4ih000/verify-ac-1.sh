#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

echo "== hosted runtime and service expansion voyage =="
sed -n '1,220p' .keel/epics/1vz4Yn000/voyages/1vz5mg000/README.md

echo
echo "== hosted runtime and service expansion requirements =="
rg -n 'SRS-0[1-6]|SRS-NFR-03|Story Coverage Plan|Story Order' \
  .keel/epics/1vz4Yn000/voyages/1vz5mg000/SRS.md \
  .keel/epics/1vz4Yn000/voyages/1vz5mg000/SDD.md

echo
echo "== follow-on story scopes =="
for story in 1vz5nU000 1vz5nk000 1vz5o6000 1vz5nx000 1vz5nl000 1vz5nm000; do
  echo "-- $story --"
  sed -n '1,24p' ".keel/stories/$story/README.md"
done
