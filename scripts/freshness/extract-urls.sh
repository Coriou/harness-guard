#!/bin/sh
# extract-urls.sh — list every cited source URL from the rules data package.
set -eu
cd "$(dirname "$0")/../.."
find rules -name '*.json' -not -name 'ruleset.json' -print0 \
  | xargs -0 jq -r '.sources[].url' \
  | sort -u
