#!/usr/bin/env bash
# Component-capability gap benchmark: vgui vs longbridge/gpui-kit.
#
# Deterministic static capability audit. A frozen reference catalog of
# gpui-kit's user-facing components/framework capabilities (82 entries,
# captured 2026-10-17 from gpui-component 0.7) is matched against evidence in
# the local vgui crate sources. Each capability is classified:
#   full    - a real implementation exists
#   partial - only a primitive / narrower equivalent exists
#   absent  - no equivalent
#
# Primary metric: component_coverage_pct = 100 * (full + 0.5*partial) / N.
# Higher is better; 100 = full parity with the gpui-kit catalog.
#
# No network, no clock, no randomness: same sources -> same output.
set -euo pipefail

cd "$(dirname "$0")"

python3 bench/component_gap/compare.py
