#!/bin/sh
# fetch-cited.sh — fetch every cited rule URL, hash via normalize.sh, compare
# to url-hashes.json. Maintainer-only. Does not write the freshness tree or
# the rules tree.
# Usage: fetch-cited.sh [--save-wayback]
# Known limitation (same as doc-drift.yml / normalize.sh): regex tag stripping
# is approximate; JS-rendered pages may need a Playwright fallback later.
# Exit 0 all match; 1 any drift or missing entry; 2 fetch/hash failure.
set -eu
cd "$(dirname "$0")/../.."

SAVE_WAYBACK=0
if [ "${1:-}" = "--save-wayback" ]; then
  SAVE_WAYBACK=1
elif [ -n "${1:-}" ]; then
  echo "Usage: $0 [--save-wayback]" >&2
  exit 2
fi

HASHES="freshness/url-hashes.json"
if ! jq -e 'type == "object" and (.hashes | type == "object")' \
  "$HASHES" >/dev/null; then
  echo "FAIL: url-hashes schema missing hashes object" >&2
  exit 2
fi

tmp_dir=$(mktemp -d)
trap 'rm -rf "$tmp_dir"' EXIT
scripts/freshness/extract-urls.sh > "$tmp_dir/urls.txt"
if [ ! -s "$tmp_dir/urls.txt" ]; then
  echo "FAIL: no cited URLs" >&2
  exit 2
fi

status=0
printf '%s\t%s\t%s\t%s\n' "url" "old" "new" "status"

while IFS= read -r url; do
  [ -n "$url" ] || continue
  document=$(mktemp "$tmp_dir/document.XXXXXX")
  if ! curl --fail --silent --show-error --location "$url" --output "$document"; then
    echo "FAIL: fetch failed: $url" >&2
    exit 2
  fi
  if [ ! -s "$document" ]; then
    echo "FAIL: empty document: $url" >&2
    exit 2
  fi
  hex=$(scripts/freshness/normalize.sh "$document") || {
    echo "FAIL: normalize failed: $url" >&2
    exit 2
  }
  case "$hex" in
    *[!0-9a-f]* | "")
      echo "FAIL: invalid hash for $url" >&2
      exit 2
      ;;
  esac
  if [ "${#hex}" -ne 64 ]; then
    echo "FAIL: invalid hash length for $url" >&2
    exit 2
  fi
  new="sha256:$hex"
  old=$(jq -r --arg url "$url" '.hashes[$url] // empty' "$HASHES")
  if [ -z "$old" ]; then
    row_status="missing-from-freshness"
    old="-"
    status=1
  elif [ "$old" = "$new" ]; then
    row_status="match"
  else
    row_status="drift"
    status=1
  fi
  printf '%s\t%s\t%s\t%s\n' "$url" "$old" "$new" "$row_status"

  if [ "$SAVE_WAYBACK" -eq 1 ]; then
    snapshot=$(curl --fail --silent --show-error \
      "https://web.archive.org/save/$url" \
      --output /dev/null --write-out '%{redirect_url}' || true)
    if [ -z "$snapshot" ]; then
      snapshot="unavailable"
    fi
    printf 'wayback\t%s\t%s\n' "$url" "$snapshot"
  fi
done < "$tmp_dir/urls.txt"

exit "$status"
