#!/bin/sh
# normalize.sh — semantic-text normalization + sha256 (hex to stdout).
# Usage: normalize.sh [file]   (reads stdin if no file)
# Shared definition for rule source content_hash AND doc-drift hashing (§5.1).
# 1. drop script/style/nav/header/footer blocks  2. strip tags
# 3. decode common entities  4. collapse whitespace  5. hash
# Known limitation (documented in doc-drift.yml too): regex tag stripping is
# approximate; JS-rendered pages may need a Playwright fallback later.
set -eu
INPUT="${1:-/dev/stdin}"
perl -0777 -pe '
  s/<script\b.*?<\/script>//gis;
  s/<style\b.*?<\/style>//gis;
  s/<nav\b.*?<\/nav>//gis;
  s/<header\b.*?<\/header>//gis;
  s/<footer\b.*?<\/footer>//gis;
  s/<[^>]+>/ /g;
  s/&nbsp;/ /g; s/&amp;/&/g; s/&lt;/</g; s/&gt;/>/g; s/&quot;/"/g; s/&#39;/'"'"'/g;
  s/\s+/ /g; s/^\s+|\s+$//g;
' "$INPUT" | shasum -a 256 | cut -d' ' -f1
