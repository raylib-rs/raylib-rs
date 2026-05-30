#!/usr/bin/env bash
# UBSAN findings report generator for sanitizers.yml.
#
# Reads ubsan.log* files produced by libubsan (with UBSAN_OPTIONS log_path=ubsan.log,
# libubsan appends .pid per-process). Parses runtime-error diagnostics into kind/
# count/first-occurrence rows, then writes a markdown table to $GITHUB_STEP_SUMMARY
# followed by the raw log inside a <details> collapse.
#
# Always exits 0 — the surrounding job has continue-on-error: true, but explicit
# success here keeps the script's contract clear.
#
# See docs/superpowers/specs/2026-05-30-ubsan-through-ffi-design.md §4.2.

set -u

SUMMARY="${GITHUB_STEP_SUMMARY:-/dev/stdout}"

# libubsan appends .pid; tests may spawn several. Match both `ubsan.log` and `ubsan.log.*`.
shopt -s nullglob
logs=(ubsan.log ubsan.log.*)

if (( ${#logs[@]} == 0 )); then
    echo "## UBSAN: no findings (no log files produced)" >> "$SUMMARY"
    exit 0
fi

combined=$(cat "${logs[@]}" 2>/dev/null || true)

# Whitespace-only check.
if [[ -z "${combined//[[:space:]]/}" ]]; then
    echo "## UBSAN: no findings (logs present but empty)" >> "$SUMMARY"
    exit 0
fi

# Parse runtime-error lines. libubsan's diagnostic format is:
#   <file>:<line>:<col>: runtime error: <kind>: <details...>
# gawk's match() with capture-array is GNU-specific; ubuntu-latest has gawk by default.
parsed=$(echo "$combined" | awk '
    /runtime error:/ {
        if (match($0, /([^ :]+):([0-9]+):([0-9]+): runtime error: ([a-zA-Z0-9_-]+)/, arr)) {
            kind = arr[4];
            loc = arr[1] ":" arr[2] ":" arr[3];
            count[kind]++;
            if (!(kind in first)) first[kind] = loc;
        }
    }
    END {
        n = 0; total = 0;
        for (k in count) { n++; total += count[k]; }
        printf "TOTAL\t%d\t%d\n", total, n;
        for (k in count) printf "ROW\t%s\t%d\t%s\n", k, count[k], first[k];
    }
')

total=$(printf "%s\n" "$parsed" | awk -F'\t' '/^TOTAL/ {print $2}')
kinds=$(printf "%s\n" "$parsed" | awk -F'\t' '/^TOTAL/ {print $3}')

emit_raw_log() {
    {
        echo ""
        echo "<details><summary>Raw log</summary>"
        echo ""
        echo '```'
        echo "$combined"
        echo '```'
        echo ""
        echo "</details>"
    } >> "$SUMMARY"
}

if [[ -z "$total" || "$total" == "0" ]]; then
    echo "## UBSAN: no findings (logs present, no parseable runtime-error lines)" >> "$SUMMARY"
    emit_raw_log
    exit 0
fi

{
    echo "## UBSAN findings ($total hits across $kinds kinds)"
    echo ""
    echo "| kind | count | first occurrence |"
    echo "|------|-------|------------------|"
    printf "%s\n" "$parsed" | awk -F'\t' '/^ROW/ {
        printf "| %s | %s | %s |\n", $2, $3, $4;
    }'
} >> "$SUMMARY"

emit_raw_log
exit 0
