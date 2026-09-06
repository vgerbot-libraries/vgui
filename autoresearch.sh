#!/usr/bin/env bash
set -euo pipefail

# Keyboard event system benchmark harness.
#
# Compiles the bench_keyboard example in release mode and runs it.
# The benchmark exercises KeyboardEvent construction, stop_propagation,
# handler dispatch, and the public new() constructor.
#
# Prints METRIC lines parsed by the autoresearch framework.

cd "$(dirname "$0")"

# Build the benchmark binary in release mode for stable timings.
cargo build --release --example bench_keyboard -p vgui 2>&1

# Run the benchmark.
./target/release/examples/bench_keyboard
