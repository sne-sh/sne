#!/usr/bin/env bash
# Measures the commands that run in every shell and on every `cd`.
# Needs hyperfine. Compares against fnm when fnm is on PATH.
#
#   benches/bench.sh              print timings
#   benches/bench.sh --check 25   also fail if any sne command has a median above 25 ms
set -euo pipefail

cd "$(dirname "$0")/.."
cargo build --release --quiet
export PATH="$PWD/target/release:$PATH"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
export SNE_DIR="$work/sne"
mkdir -p "$work/project"
echo "20" > "$work/project/.nvmrc"

sne install 20 22 > /dev/null
eval "$(sne env --shell bash)"

commands=(
  -n "sne use 22" "sne use 22"
  -n "sne use --auto" "cd '$work/project' && sne use --auto"
  -n "sne env" "sne env --shell bash"
  -n "sne current" "sne current"
)
if command -v fnm > /dev/null; then
  export FNM_DIR="$work/fnm"
  fnm install 20 > /dev/null 2>&1
  fnm install 22 > /dev/null 2>&1
  eval "$(fnm env --shell bash)"
  commands+=(-n "fnm use 22" "fnm use 22" -n "fnm env" "fnm env --shell bash")
fi

hyperfine --warmup 10 --shell=bash --export-json "$work/results.json" "${commands[@]}"

if [[ "${1:-}" == "--check" ]]; then
  limit_ms="${2:?usage: bench.sh --check <milliseconds>}"
  # hyperfine reports seconds; pull out each sne command's median without needing jq. The median,
  # not the mean: on a shared runner one run can stall for most of a second (an antivirus scan, a
  # busy neighbour), which drags the mean over the limit while every other run is fast.
  node -e '
    const { results } = JSON.parse(require("fs").readFileSync(process.argv[1], "utf8"));
    const limit = Number(process.argv[2]);
    const slow = results.filter(r => r.command.startsWith("sne") && r.median * 1000 > limit);
    for (const r of slow) console.error(`too slow: ${r.command} median ${(r.median * 1000).toFixed(1)} ms > ${limit} ms`);
    process.exit(slow.length ? 1 : 0);
  ' "$work/results.json" "$limit_ms"
fi
