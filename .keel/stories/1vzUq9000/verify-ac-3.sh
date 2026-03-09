#!/usr/bin/env bash
set -euo pipefail

cd /home/alex/workspace/spoke-sh/port

flow_output="$(keel flow)"
printf '%s\n' "$flow_output"

grep -F "ready             0" <<<"$flow_output"
grep -F "blocked           0" <<<"$flow_output"
grep -F "in-flight         1" <<<"$flow_output"
grep -F "1vzUnI000" <<<"$flow_output"

voyage_readme=".keel/epics/1vzUnI000/voyages/1vzUoK000/README.md"
rg -n '\[Define Durable Hosted Registry Contract\].*\| feat \| done \|' "$voyage_readme"
rg -n '\[Persist Hosted Registration And Freshness\].*\| feat \| done \|' "$voyage_readme"
rg -n '\[Materialize Imported Fleet Inventory\].*\| feat \| done \|' "$voyage_readme"
rg -n '\[Surface Durable Hosted Fleet State\].*\| feat \| done \|' "$voyage_readme"
rg -n '\[Publish Durable Hosted Fleet Workflow\].*\| feat \| in-progress \|' "$voyage_readme"

for id in 1vzUq5000 1vzUq6000 1vzUq7000 1vzUq8000; do
  test -d ".keel/stories/$id/EVIDENCE"
done
