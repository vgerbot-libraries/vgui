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

EXAMPLES=(counter todolist styling theming variants inputs elements forms context refs focus overlays animation canvas router dashboard)
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

# 1a. Build all examples in a single cargo invocation — cargo parallelizes
# internally across crates sharing one target dir. This avoids lock
# contention from multiple concurrent cargo processes.
PKGS=()
for ex in "${EXAMPLES[@]}"; do
    PKGS+=("-p" "vgui-${ex}")
done
cargo +nightly build --target wasm32-unknown-unknown --release "${PKGS[@]}"

# 1b. Run wasm-bindgen + asset copy for each example in parallel.
# These steps are fully independent (separate dist/ and book/src/wasm/ dirs).
pids=()
for ex in "${EXAMPLES[@]}"; do
    (
        set -e
        rm -rf "examples/${ex}/dist"
        wasm-bindgen --target web --out-dir "examples/${ex}/dist" --no-typescript \
            "target/wasm32-unknown-unknown/release/${ex}.wasm"
        mkdir -p "book/src/wasm/${ex}"
        cp "examples/${ex}/dist/${ex}.js" "book/src/wasm/${ex}/"
        cp "examples/${ex}/dist/${ex}_bg.wasm" "book/src/wasm/${ex}/"
        if [ ! -f "book/src/wasm/${ex}/index.html" ]; then
            cat > "book/src/wasm/${ex}/index.html" << HTMLEOF
<!doctype html>
<html lang="en">
    <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, height=device-height, initial-scale=1.0, user-scalable=0" />
        <title>vgui ${ex}</title>
        <style>
            * { margin: 0; padding: 0; box-sizing: border-box; }
            html, body { margin: 0; height: 100%; background: #1e1e1e; }
            canvas {
                display: block;
                width: 100%;
                height: 100%;
                touch-action: none;
                outline: none;
                -webkit-user-select: none;
                user-select: none;
            }
        </style>
    </head>
    <body>
        <script type="module">
            import init from "./${ex}.js";
            init();
        </script>
    </body>
</html>
HTMLEOF
        fi
    ) &
    pids+=($!)
done

# Wait for all parallel jobs; fail if any failed.
fail=0
for pid in "${pids[@]}"; do
    wait "$pid" || fail=1
done
[ "$fail" -eq 0 ] || { echo "WASM bindgen/copy failed" >&2; exit 1; }

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
