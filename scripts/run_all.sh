#!/usr/bin/env bash

set -uo pipefail

time_secs=3600
chains_per_image=4

args=(
    "kiwi.png"
    "psl_logo.png"
    "github_logo.png"
    "yinyang.png"
)

pids=()

cargo build --release

for arg in "${args[@]}"; do
    target/release/morph "$arg" "$time_secs" "$chains_per_image" > "${arg}_search.log" 2>&1 &
    pids+=($!)
done

status=0

for pid in "${pids[@]}"; do
    if ! wait "$pid"; then
        status=1
    fi
done


exit "$status"
