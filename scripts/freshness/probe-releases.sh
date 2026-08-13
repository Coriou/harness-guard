#!/bin/sh
# probe-releases.sh — compare live npm dist-tags and the Grok CLI pointer
# against last-seen.json. Maintainer-only. Does not write the freshness
# tree or the rules tree.
# Exit 0 all match; 1 any drift; 2 fetch/parse failure.
set -eu
cd "$(dirname "$0")/../.."

LAST_SEEN="freshness/last-seen.json"

if ! jq -e \
  'type == "object" and (.packages | type == "object") and (.channels["grok-build"] | type == "object")' \
  "$LAST_SEEN" >/dev/null; then
  echo "FAIL: last-seen schema missing packages or channels.grok-build" >&2
  exit 2
fi

status=0

probe_npm() {
  pkg="$1"
  tag="$2"
  encoded=$(printf '%s' "$pkg" | sed 's|/|%2F|')
  tmp=$(mktemp)
  if ! curl --fail --silent --show-error \
    -H 'Accept: application/vnd.npm.install-v1+json' \
    "https://registry.npmjs.org/$encoded" \
    --output "$tmp"; then
    echo "FAIL: npm fetch failed for $pkg" >&2
    rm -f "$tmp"
    exit 2
  fi
  if ! jq -e 'type == "object" and (."dist-tags" | type == "object")' "$tmp" >/dev/null; then
    echo "FAIL: npm document shape wrong for $pkg" >&2
    rm -f "$tmp"
    exit 2
  fi
  live=$(jq -er --arg tag "$tag" \
    '."dist-tags"[$tag] | strings | select(length > 0)' "$tmp") || {
    echo "FAIL: dist-tag $tag missing for $pkg" >&2
    rm -f "$tmp"
    exit 2
  }
  seen=$(jq -er --arg pkg "$pkg" \
    '.packages[$pkg].version | strings | select(length > 0)' "$LAST_SEEN") || {
    echo "FAIL: last-seen missing version for $pkg" >&2
    rm -f "$tmp"
    exit 2
  }
  rm -f "$tmp"
  if [ "$seen" = "$live" ]; then
    row_status="match"
  else
    row_status="drift"
    status=1
  fi
  printf '%s\t%s\t%s\t%s\t%s\n' "$pkg" "$tag" "$seen" "$live" "$row_status"
}

printf '%s\t%s\t%s\t%s\t%s\n' "package" "tag" "seen" "live" "status"
probe_npm "@anthropic-ai/claude-code" "stable"
probe_npm "@openai/codex" "latest"
probe_npm "@github/copilot" "latest"
probe_npm "@xai-official/grok" "latest"

tmp=$(mktemp)
if ! curl --fail --silent --show-error --location \
  "https://x.ai/cli/stable" --output "$tmp"; then
  echo "FAIL: fetch of the Grok CLI pointer failed" >&2
  rm -f "$tmp"
  exit 2
fi
live=$(sed -n '1p' "$tmp" | tr -d ' \t\r\n')
rm -f "$tmp"
if [ -z "$live" ]; then
  echo "FAIL: empty Grok channel pointer" >&2
  exit 2
fi
seen=$(jq -er \
  '.channels["grok-build"].version | strings | select(length > 0)' \
  "$LAST_SEEN") || {
  echo "FAIL: last-seen missing channels.grok-build.version" >&2
  exit 2
}
if [ "$seen" = "$live" ]; then
  row_status="match"
else
  row_status="drift"
  status=1
fi
printf '%s\t%s\t%s\t%s\t%s\n' "grok-build" "cli-pointer" "$seen" "$live" "$row_status"

exit "$status"
