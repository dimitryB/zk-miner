#!/bin/sh
# Experimental one-stage hybrid build; CPU and unchanged verifier targets stay separate.
set -eu
engine=${1:?usage: build-hybrid.sh ENGINE_DIR REVIEWED_ENGINE_SHA}
revision=${2:?pass the full reviewed engine commit SHA}
HEMI_MINER_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
export HEMI_MINER_DIR
. "$HEMI_MINER_DIR/scripts/macos/env.sh"
cd "$HEMI_MINER_DIR"
[ "$(uname -m)" = arm64 ] && [ "$(uname -s)" = Darwin ] || { echo 'Apple Silicon macOS required' >&2; exit 1; }
python3 scripts/macos/local-engine.py "$engine" target/macos/hybrid-engine.toml --revision "$revision"
python3 scripts/macos/proof_provenance.py snapshot hybrid --engine "$engine"
cargo +1.93.0 build --locked --release --manifest-path tools/macos-hybrid/Cargo.toml \
    --config target/macos/hybrid-engine.toml --target-dir target/macos/hybrid
cargo +1.93.0 metadata --locked --filter-platform aarch64-apple-darwin \
    --manifest-path tools/macos-hybrid/Cargo.toml --config target/macos/hybrid-engine.toml \
    --format-version 1 > target/macos/hybrid-metadata.json
python3 scripts/macos/check-source.py target/macos/hybrid-metadata.json --engine "$engine" --metal-hybrid
cargo +1.93.0 tree --locked --target aarch64-apple-darwin --edges features \
    --manifest-path tools/macos-hybrid/Cargo.toml --config target/macos/hybrid-engine.toml \
    > target/macos/hybrid-tree.txt
python3 scripts/macos/proof_provenance.py seal hybrid --engine "$engine"
