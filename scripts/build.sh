#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
NAVIDROME_VERSION="${NAVIDROME_VERSION:-v0.64.2}"
NAVIDROME_DIR="$ROOT/.deps/navidrome"

echo "==> Building navidrome-delete"
echo "==> Navidrome SDK: $NAVIDROME_VERSION"

mkdir -p "$ROOT/.deps"

if [ ! -d "$NAVIDROME_DIR/.git" ]; then
    echo "==> Downloading Navidrome SDK..."
    git clone \
        --depth 1 \
        --branch "$NAVIDROME_VERSION" \
        https://github.com/navidrome/navidrome.git \
        "$NAVIDROME_DIR"
else
    echo "==> Navidrome SDK already present"
fi

echo "==> Building WebAssembly..."

docker run --rm \
    -v "$ROOT:/src" \
    -w /src \
    rust:1.90 \
    bash -c '
        /usr/local/cargo/bin/rustup target add wasm32-wasip1 &&
        /usr/local/cargo/bin/cargo build \
            --release \
            --target wasm32-wasip1
    '

echo "==> Packaging .ndp..."

rm -rf "$ROOT/dist"
mkdir -p "$ROOT/dist/package"

cp \
    "$ROOT/target/wasm32-wasip1/release/navidrome_delete.wasm" \
    "$ROOT/dist/package/plugin.wasm"

cp \
    "$ROOT/manifest.json" \
    "$ROOT/dist/package/manifest.json"

(
    cd "$ROOT/dist/package"
    zip -q -j \
        "$ROOT/dist/navidrome-delete.ndp" \
        manifest.json \
        plugin.wasm
)

rm -rf "$ROOT/dist/package"

echo
echo "Build complete:"
ls -lh "$ROOT/dist/navidrome-delete.ndp"
