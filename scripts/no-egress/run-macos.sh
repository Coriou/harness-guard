#!/bin/sh
# Instrumented no-egress proof, macOS. Runs real scans over synthetic
# fixtures inside a deny-all-network sandbox and asserts the exact §6 exit
# codes — a blocked network call would surface as exit 2 / error output.
set -eu
cd "$(dirname "$0")/../.."

cargo build -p harness-guard-cli
BIN=target/debug/harness-guard
SB=scripts/no-egress/scan.sb

# NB: sandbox-exec/curl are invoked by absolute path — the per-command PATH
# override (needed so version detection sees only the fixture path dir)
# would otherwise break command lookup.
run_case() {
    case_dir="fixtures/codex/$1/files"
    want="$2"
    set +e
    CODEX_HOME="$PWD/$case_dir/codex-home" PATH="$PWD/$case_dir/path" NO_COLOR=1 \
        /usr/bin/sandbox-exec -f "$SB" "$PWD/$BIN" scan --json \
        > /tmp/harness-guard-noegress.json 2>&1
    got=$?
    set -e
    if [ "$got" -ne "$want" ]; then
        echo "FAIL: case $1 exited $got, expected $want" >&2
        cat /tmp/harness-guard-noegress.json >&2
        exit 1
    fi
    echo "ok: $1 (exit $got under deny-all-network sandbox)"
}

run_case hardened 0
run_case risky-unset 1
run_case malformed-toml 2
run_case unknown-version 0

# Sanity check that the sandbox profile actually blocks network: curl must fail.
if /usr/bin/sandbox-exec -f "$SB" /usr/bin/curl -s --max-time 5 https://example.com >/dev/null 2>&1; then
    echo "FAIL: sandbox profile did not block network — proof is void" >&2
    exit 1
fi
echo "ok: sandbox profile verified to block egress"
echo "NO-EGRESS PROOF PASSED"
