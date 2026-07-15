#!/bin/sh
# Layer 3 of the no-egress proof, macOS: run real scans over synthetic
# fixtures in a deny-all-network sandbox, verify the JSON/diagnostic contract,
# and use sandbox telemetry to catch even handled or ignored denied attempts.
set -eu
umask 077
cd "$(dirname "$0")/../.."

resolve_cargo() {
    if [ -n "${CARGO:-}" ] && [ -x "$CARGO" ]; then
        printf '%s\n' "$CARGO"
        return
    fi
    candidate=$(command -v cargo 2>/dev/null || true)
    if [ -n "$candidate" ] && [ -x "$candidate" ]; then
        printf '%s\n' "$candidate"
        return
    fi
    for candidate in /usr/local/opt/rustup/bin/cargo "${HOME:-}/.cargo/bin/cargo"; do
        if [ -n "$candidate" ] && [ -x "$candidate" ]; then
            printf '%s\n' "$candidate"
            return
        fi
    done
    echo "FAIL: cargo not found; set CARGO to an executable cargo path" >&2
    exit 1
}

resolve_jq() {
    if [ -n "${JQ:-}" ] && [ -x "$JQ" ]; then
        printf '%s\n' "$JQ"
        return
    fi
    candidate=$(command -v jq 2>/dev/null || true)
    if [ -n "$candidate" ] && [ -x "$candidate" ]; then
        printf '%s\n' "$candidate"
        return
    fi
    for candidate in /usr/local/bin/jq /opt/homebrew/bin/jq; do
        if [ -x "$candidate" ]; then
            printf '%s\n' "$candidate"
            return
        fi
    done
    echo "FAIL: jq not found; set JQ to an executable jq path" >&2
    exit 1
}

CARGO_BIN=$(resolve_cargo)
JQ_BIN=$(resolve_jq)
# Rustup's cargo proxy locates its sibling rustc proxy through PATH. This PATH
# is build-only; every scan below receives a fixture-only PATH via env -i.
CARGO_DIR=${CARGO_BIN%/*}
PATH="$CARGO_DIR:${PATH:-/usr/bin:/bin}" "$CARGO_BIN" build -p harness-guard-cli

BIN="$PWD/target/debug/harness-guard"
SB="$PWD/scripts/no-egress/scan.sb"
PROOF_DIR=$(/usr/bin/mktemp -d /tmp/harness-guard-noegress.XXXXXX)
cleanup() {
    /bin/rm -rf "$PROOF_DIR"
}
trap cleanup EXIT
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM

LOG_PREDICATE='subsystem == "com.apple.sandbox.reporting" AND eventMessage CONTAINS "HARNESS_GUARD_NETWORK_DENIED"'

query_denials() {
    since=$1
    destination=$2
    if ! /usr/bin/log show --start "$since" --style compact \
        --predicate "$LOG_PREDICATE" >"$destination" 2>&1; then
        echo "FAIL: could not query sandbox denial telemetry" >&2
        /bin/cat "$destination" >&2
        exit 1
    fi
}

run_case() {
    case_name=$1
    want=$2
    summary_marker=$3
    case_dir="$PWD/fixtures/codex/$case_name/files"
    stdout="$PROOF_DIR/$case_name.stdout"
    stderr="$PROOF_DIR/$case_name.stderr"

    set +e
    /usr/bin/env -i \
        HOME="$case_dir" \
        CODEX_HOME="$case_dir/codex-home" \
        PATH="$case_dir/path" \
        NO_COLOR=1 \
        /usr/bin/sandbox-exec -f "$SB" "$BIN" scan --json \
        >"$stdout" 2>"$stderr"
    got=$?
    set -e

    if [ "$got" -ne "$want" ]; then
        echo "FAIL: case $case_name exited $got, expected $want" >&2
        /bin/cat "$stdout" >&2
        /bin/cat "$stderr" >&2
        exit 1
    fi
    # shellcheck disable=SC2016 # jq, not the shell, expands $marker.
    if ! "$JQ_BIN" -e --arg marker "$summary_marker" \
        'type == "object" and .network_requests_made == 0 and .summary[$marker] == 1' \
        "$stdout" >/dev/null; then
        echo "FAIL: case $case_name did not emit valid zero-network JSON with summary.$summary_marker = 1" >&2
        /bin/cat "$stdout" >&2
        exit 1
    fi

    if [ "$case_name" = malformed-toml ]; then
        if ! /usr/bin/grep -Fq 'harness_guard::config_parse' "$stderr" ||
            ! /usr/bin/grep -Fq 'config not safely parseable' "$stderr"; then
            echo "FAIL: malformed-toml lacked the structural miette diagnostic" >&2
            /bin/cat "$stderr" >&2
            exit 1
        fi
        if /usr/bin/grep -Fq 'persistence' "$stderr"; then
            echo "FAIL: malformed-toml diagnostic leaked a raw persistence snippet" >&2
            /bin/cat "$stderr" >&2
            exit 1
        fi
    elif [ -s "$stderr" ]; then
        echo "FAIL: case $case_name unexpectedly wrote to stderr" >&2
        /bin/cat "$stderr" >&2
        exit 1
    fi

    echo "ok: $case_name (exit $got; summary.$summary_marker=1; network_requests_made=0)"
}

# Record the start before any scans. A tagged denial in this interval means a
# scan attempted egress, even if the process caught the resulting error.
SCAN_LOG_START=$(/bin/date '+%Y-%m-%d %H:%M:%S')
run_case hardened 0 passed
run_case risky-unset 1 warning
run_case malformed-toml 2 unknown
run_case unknown-version 0 stale

/bin/sleep 1
query_denials "$SCAN_LOG_START" "$PROOF_DIR/scan-denials.log"
if /usr/bin/grep -Fq 'HARNESS_GUARD_NETWORK_DENIED' "$PROOF_DIR/scan-denials.log"; then
    echo "FAIL: a sandboxed scan attempted network egress" >&2
    /bin/cat "$PROOF_DIR/scan-denials.log" >&2
    exit 1
fi
echo "ok: scan telemetry contains no denied network attempts"

# Sanity-prove both enforcement and telemetry without public DNS or internet:
# a loopback TCP connect must fail with EPERM and produce our tagged denial.
SANITY_LOG_START=$(/bin/date '+%Y-%m-%d %H:%M:%S')
set +e
/usr/bin/sandbox-exec -f "$SB" /usr/bin/curl \
    --silent --show-error --verbose --max-time 2 http://127.0.0.1:9 \
    >"$PROOF_DIR/curl.stdout" 2>"$PROOF_DIR/curl.stderr"
curl_status=$?
set -e
if [ "$curl_status" -eq 0 ]; then
    echo "FAIL: sandbox profile allowed the loopback network probe" >&2
    exit 1
fi
if ! /usr/bin/grep -Fq 'Operation not permitted' "$PROOF_DIR/curl.stderr"; then
    echo "FAIL: loopback probe did not fail with Operation not permitted" >&2
    /bin/cat "$PROOF_DIR/curl.stderr" >&2
    exit 1
fi

sanity_denial_seen=0
attempt=1
while [ "$attempt" -le 5 ]; do
    query_denials "$SANITY_LOG_START" "$PROOF_DIR/sanity-denials.log"
    if /usr/bin/grep -Fq 'HARNESS_GUARD_NETWORK_DENIED' "$PROOF_DIR/sanity-denials.log"; then
        sanity_denial_seen=1
        break
    fi
    /bin/sleep 1
    attempt=$((attempt + 1))
done
if [ "$sanity_denial_seen" -ne 1 ]; then
    echo "FAIL: sandbox denied loopback but emitted no tagged telemetry" >&2
    /bin/cat "$PROOF_DIR/sanity-denials.log" >&2
    exit 1
fi

echo "ok: sandbox profile denied loopback egress with EPERM and tagged telemetry"
echo "NO-EGRESS PROOF PASSED"
