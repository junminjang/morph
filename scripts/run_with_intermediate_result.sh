#!/usr/bin/env bash

set -euo pipefail

if [ "$#" -lt 1 ]; then
    echo "Usage: $0 IMAGE [TIME_SECS=3600] [CHAINS=4] [CHECKPOINT_SECS...]" >&2
    exit 2
fi

input_image="$1"
shift

if [[ "$input_image" = /* ]]; then
    image_path="$input_image"
else
    image_path="$(pwd)/$input_image"
fi

time_secs="${1:-3600}"
if [ "$#" -gt 0 ]; then
    shift
fi

chains="${1:-4}"
if [ "$#" -gt 0 ]; then
    shift
fi

if [ "$#" -gt 0 ]; then
    checkpoints=("$@")
else
    checkpoints=(60 600 1200 2400 3600)
fi

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "$script_dir/.." && pwd)"
cd "$repo_root"

if [ ! -f "$image_path" ]; then
    echo "Image not found: $image_path" >&2
    exit 1
fi

base="$(basename -- "$image_path")"
stem="${base%.*}"
log_file="${stem}_intermediate_search.log"
checkpoint_csv="$(IFS=,; echo "${checkpoints[*]}")"

cargo build --release

echo "Running $image_path for ${time_secs}s with ${chains} chain(s)"
echo "Checkpoints: $checkpoint_csv"
echo "Log: $log_file"

MORPH_CHECKPOINT_SECS="$checkpoint_csv" \
    target/release/morph "$image_path" "$time_secs" "$chains" > "$log_file" 2>&1

echo "Done"
echo "Final result: ${stem}_result.png"
echo "Checkpoint results: ${stem}_result_<seconds>s.png"
