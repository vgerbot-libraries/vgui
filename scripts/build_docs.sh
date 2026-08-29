#!/usr/bin/env bash
# Build the full vgui documentation site in one command:
#   1. WASM demos for every example  → book/src/wasm/<example>/
#   2. mdBook                         → book/book/
#   3. Rust API docs (cargo doc)      → book/book/api/
#
# Usage:
#   scripts/build_docs.sh            # build everything
#   scripts/build_docs.sh --serve    # build, then serve on http://127.0.0.1:8080
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT_DIR"

EXAMPLES=(counter todolist inputs tags-demo)
SERVE=0

for arg in "$@"; do
    case "$arg" in
        --serve) SERVE=1 ;;
        -h|--help)
            sed -n '2,11p' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *) echo "unknown option: $arg" >&2; exit 2 ;;
    esac
done

# 1. WASM demos
echo "==> [1/3] Building WASM demos..."
for ex in "${EXAMPLES[@]}"; do
    "$SCRIPT_DIR/build_wasm.sh" "$ex"
done

# 2. mdBook
echo "==> [2/3] Building mdBook..."
mdbook build book

# 3. API docs
echo "==> [3/3] Building API docs..."
cargo doc --workspace --no-deps
mkdir -p book/book/api
cp -r target/doc/* book/book/api/

# Disable Jekyll so GitHub Pages serves _-prefixed rustdoc assets.
touch book/book/.nojekyll

echo "==> Done. Site output in book/book/"

if [ "$SERVE" -eq 1 ]; then
    exec python3 "$SCRIPT_DIR/serve_wasm.py" 8080 "$ROOT_DIR/book/book"
fi
