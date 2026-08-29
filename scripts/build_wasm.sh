#!/usr/bin/env bash
# Build the counter WASM demo and copy assets into the mdBook source tree.
# Usage: scripts/build_wasm.sh [example-name]
set -euo pipefail

EXAMPLE="${1:-counter}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PKG="vgui-${EXAMPLE}"

cd "$ROOT_DIR"

echo "==> Building ${EXAMPLE} for wasm32-unknown-unknown (release)..."
cargo +nightly build --target wasm32-unknown-unknown -p "$PKG" --release

echo "==> Generating wasm-bindgen bindings..."
rm -rf "examples/${EXAMPLE}/dist"
wasm-bindgen --target web --out-dir "examples/${EXAMPLE}/dist" --no-typescript \
    "target/wasm32-unknown-unknown/release/${EXAMPLE}.wasm"

echo "==> Copying assets to book/src/wasm/${EXAMPLE}/..."
mkdir -p "book/src/wasm/${EXAMPLE}"
cp "examples/${EXAMPLE}/dist/${EXAMPLE}.js" "book/src/wasm/${EXAMPLE}/"
cp "examples/${EXAMPLE}/dist/${EXAMPLE}_bg.wasm" "book/src/wasm/${EXAMPLE}/"

# Copy the standalone HTML if it doesn't exist yet
if [ ! -f "book/src/wasm/${EXAMPLE}/index.html" ]; then
    cat > "book/src/wasm/${EXAMPLE}/index.html" << HTMLEOF
<!doctype html>
<html lang="en">
    <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, height=device-height, initial-scale=1.0, user-scalable=0" />
        <title>vgui ${EXAMPLE}</title>
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
            import init from "./${EXAMPLE}.js";
            init();
        </script>
    </body>
</html>
HTMLEOF
fi

echo "==> Done. Rebuild the book with: mdbook build book"
