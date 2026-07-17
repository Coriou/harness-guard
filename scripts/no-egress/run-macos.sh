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

assert_parse_diagnostic() {
    label=$1
    stderr=$2
    if ! /usr/bin/grep -Fq 'harness_guard::config_parse' "$stderr" ||
        ! /usr/bin/grep -Fq 'config not safely parseable' "$stderr"; then
        echo "FAIL: $label lacked the structural miette diagnostic" >&2
        /bin/cat "$stderr" >&2
        exit 1
    fi
    # Refuse raw config-key / value leaks that appear in codex persistence
    # snippets or similar. The diagnostic must stay structural only.
    if /usr/bin/grep -Fq 'persistence' "$stderr"; then
        echo "FAIL: $label diagnostic leaked a raw persistence snippet" >&2
        /bin/cat "$stderr" >&2
        exit 1
    fi
}

run_case() {
    case_name=$1
    want=$2
    summary_marker=$3
    case_dir="$PWD/fixtures/codex/$case_name/files"
    stdout="$PROOF_DIR/codex-$case_name.stdout"
    stderr="$PROOF_DIR/codex-$case_name.stderr"

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
        echo "FAIL: case codex/$case_name exited $got, expected $want" >&2
        /bin/cat "$stdout" >&2
        /bin/cat "$stderr" >&2
        exit 1
    fi
    # >= 1, not == 1 (Task 17 heads-up): the codex ruleset now has more than
    # one rule, so a case can produce more than one finding of the same
    # summary marker. The proof only needs at least one, plus zero network.
    # shellcheck disable=SC2016 # jq, not the shell, expands $marker.
    if ! "$JQ_BIN" -e --arg marker "$summary_marker" \
        'type == "object" and .network_requests_made == 0 and .summary[$marker] >= 1' \
        "$stdout" >/dev/null; then
        echo "FAIL: case codex/$case_name did not emit valid zero-network JSON with summary.$summary_marker >= 1" >&2
        /bin/cat "$stdout" >&2
        exit 1
    fi

    if [ "$case_name" = malformed-toml ]; then
        assert_parse_diagnostic "codex/malformed-toml" "$stderr"
    elif [ -s "$stderr" ]; then
        echo "FAIL: case codex/$case_name unexpectedly wrote to stderr" >&2
        /bin/cat "$stderr" >&2
        exit 1
    fi

    marker_count=$("$JQ_BIN" -r --arg marker "$summary_marker" '.summary[$marker]' "$stdout")
    echo "ok: codex/$case_name (exit $got; summary.$summary_marker=$marker_count; network_requests_made=0)"
}

# Claude Code / Grok Build fixtures: HOME is the committed synthetic home
# (containing .claude/ or .grok/); CODEX_HOME points at an absent dir so only
# the fixture's harness is detected; PATH is the fixture path dir only.
run_harness_case() {
    tool=$1
    case_name=$2
    want=$3
    summary_marker=$4
    case_dir="$PWD/fixtures/$tool/$case_name/files"
    label="$tool/$case_name"
    stdout="$PROOF_DIR/scan-$tool-$case_name.json"
    stderr="$PROOF_DIR/scan-$tool-$case_name.stderr"

    set +e
    /usr/bin/env -i \
        HOME="$case_dir/home" \
        CODEX_HOME="$case_dir/home/absent-codex-home" \
        PATH="$case_dir/path" \
        NO_COLOR=1 \
        /usr/bin/sandbox-exec -f "$SB" "$BIN" scan --json \
        >"$stdout" 2>"$stderr"
    got=$?
    set -e

    if [ "$got" -ne "$want" ]; then
        echo "FAIL: case $label exited $got, expected $want" >&2
        /bin/cat "$stdout" >&2
        /bin/cat "$stderr" >&2
        exit 1
    fi
    # shellcheck disable=SC2016 # jq, not the shell, expands $marker.
    if ! "$JQ_BIN" -e --arg marker "$summary_marker" \
        'type == "object" and .network_requests_made == 0 and .summary[$marker] >= 1' \
        "$stdout" >/dev/null; then
        echo "FAIL: case $label did not emit valid zero-network JSON with summary.$summary_marker >= 1" >&2
        /bin/cat "$stdout" >&2
        exit 1
    fi

    case "$case_name" in
        malformed-json|malformed-toml)
            assert_parse_diagnostic "$label" "$stderr"
            ;;
        *)
            if [ -s "$stderr" ]; then
                echo "FAIL: case $label unexpectedly wrote to stderr" >&2
                /bin/cat "$stderr" >&2
                exit 1
            fi
            ;;
    esac

    marker_count=$("$JQ_BIN" -r --arg marker "$summary_marker" '.summary[$marker]' "$stdout")
    echo "ok: $label (exit $got; summary.$summary_marker=$marker_count; network_requests_made=0)"
}

# Mixed two-store case (§11.2): codex AND claude-code detected in one scan;
# CODEX_HOME points INTO the fixture home, at the committed synthetic store.
run_mixed_case() {
    case_name=$1
    want=$2
    summary_marker=$3
    mixed_dir="$PWD/fixtures/mixed/$case_name/files"
    label="mixed/$case_name"
    stdout="$PROOF_DIR/scan-mixed-$case_name.json"
    stderr="$PROOF_DIR/scan-mixed-$case_name.stderr"

    set +e
    /usr/bin/env -i \
        HOME="$mixed_dir/home" \
        CODEX_HOME="$mixed_dir/home/.codex" \
        PATH="$mixed_dir/path" \
        NO_COLOR=1 \
        /usr/bin/sandbox-exec -f "$SB" "$BIN" scan --json \
        >"$stdout" 2>"$stderr"
    got=$?
    set -e

    if [ "$got" -ne "$want" ]; then
        echo "FAIL: case $label exited $got, expected $want" >&2
        /bin/cat "$stdout" >&2
        /bin/cat "$stderr" >&2
        exit 1
    fi
    # Multi-detected proof: tools[] must hold exactly two harnesses, plus the
    # zero-network contract and a non-zero summary marker (claude-code unknown).
    # shellcheck disable=SC2016 # jq, not the shell, expands $marker.
    if ! "$JQ_BIN" -e --arg marker "$summary_marker" \
        'type == "object" and .network_requests_made == 0 and (.tools | length) == 2 and .summary[$marker] >= 1' \
        "$stdout" >/dev/null; then
        echo "FAIL: case $label did not emit valid zero-network JSON with tools|length==2 and summary.$summary_marker >= 1" >&2
        /bin/cat "$stdout" >&2
        exit 1
    fi

    # Claude side is malformed JSON, so the structural parse diagnostic is expected.
    assert_parse_diagnostic "$label" "$stderr"

    marker_count=$("$JQ_BIN" -r --arg marker "$summary_marker" '.summary[$marker]' "$stdout")
    tools_count=$("$JQ_BIN" -r '.tools | length' "$stdout")
    echo "ok: $label (exit $got; tools=$tools_count; summary.$summary_marker=$marker_count; network_requests_made=0)"
}

# Record the start before any scans. A tagged denial in this interval means a
# scan attempted egress, even if the process caught the resulting error.
SCAN_LOG_START=$(/bin/date '+%Y-%m-%d %H:%M:%S')

# Codex matrix (original single-harness cases).
run_case hardened 0 passed
run_case risky-explicit 1 warning
run_case malformed-toml 2 unknown
run_case unknown-version 0 stale

# Multi-harness matrix (Task 22): representative claude-code + grok-build
# fixtures plus the mixed two-store case so sandboxed proof covers two
# harnesses detected in the same run.
run_harness_case claude-code hardened 0 passed
run_harness_case claude-code duplicate-keys 1 warning
run_harness_case claude-code malformed-json 2 unknown
run_harness_case grok-build hardened 0 passed
run_harness_case grok-build malformed-toml 2 unknown
run_mixed_case codex-pass-claude-degraded 2 unknown

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
