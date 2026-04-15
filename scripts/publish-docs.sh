#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd -- "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
docs_root="$repo_root/website"

app_name="${DOCS_APP_NAME:-port}"
site_url="${DOCS_SITE_URL:-https://www.spoke.sh}"
preview_bucket="${DOCS_PREVIEW_BUCKET:-spoke-previews}"
publish_stable="${DOCS_PUBLISH_STABLE:-false}"
skip_sync="${DOCS_SKIP_SYNC:-false}"

default_branch() {
  local branch
  branch="$(git -C "$repo_root" rev-parse --abbrev-ref HEAD)"
  if [[ -z "$branch" || "$branch" == "HEAD" ]]; then
    git -C "$repo_root" rev-parse --short=12 HEAD
  else
    printf '%s\n' "$branch"
  fi
}

sanitize_branch() {
  local branch="$1"
  branch="${branch//\//-}"
  branch="${branch// /-}"
  printf '%s\n' "$branch"
}

run_build() {
  local base_url="$1"
  echo "Building docs with DOCS_BASE_URL=$base_url"
  (
    cd "$repo_root"
    DOCS_SITE_URL="$site_url" \
    DOCS_BASE_URL="$base_url" \
    npm --prefix "$docs_root" run build
  )
}

sync_build() {
  local prefix="$1"
  if [[ "$skip_sync" == "true" ]]; then
    echo "Skipping sync to s3://$preview_bucket/$prefix because DOCS_SKIP_SYNC=true"
    return 0
  fi

  local destination="s3://$preview_bucket/$prefix"
  echo "Uploading docs build to $destination with Cache-Control: no-cache"
  aws s3 cp "$docs_root/build/" "$destination" --recursive --cache-control "no-cache"
  echo "Removing stale docs objects from $destination"
  aws s3 sync "$docs_root/build/" "$destination" --delete
}

if [[ ! -f "$docs_root/package.json" ]]; then
  echo "missing docs package.json at $docs_root/package.json" >&2
  exit 1
fi

branch="$(sanitize_branch "${DOCS_BRANCH:-$(default_branch)}")"

echo "Installing Port docs dependencies..."
(
  cd "$repo_root"
  npm --prefix "$docs_root" ci
)

if [[ "$publish_stable" == "true" ]]; then
  run_build "/$app_name/"
  sync_build "stable/$app_name/"
fi

run_build "/previews/$app_name/$branch/"
sync_build "$app_name/$branch/"

echo
echo "Published Port docs"
echo "Bucket:        $preview_bucket"
if [[ "$publish_stable" == "true" ]]; then
  echo "Stable route:  https://www.spoke.sh/$app_name/docs"
fi
echo "Preview route: https://www.spoke.sh/previews/$app_name/$branch/docs"
