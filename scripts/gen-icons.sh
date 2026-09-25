#!/usr/bin/env bash
# Regenerates the Oxidean favicon / app-icon set from brand/oxidean-mark.png.
# Squircle silhouette comes from the same mask the DOM mark uses (D-02, D-20).
set -euo pipefail

cd "$(dirname "$0")/.."

SRC=brand/oxidean-mark.png
MASK=apps/web/public/brand/squircle.svg
OUT=apps/web/public
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# Brand ground is charcoal (see brand/README.md). Transparent corners outside the
# squircle read as white in browser chrome / light OS surfaces — flatten onto
# opaque charcoal while keeping the squircle-clipped mark.
BRAND_BLACK='#121212'

command -v magick >/dev/null || { echo "ImageMagick 'magick' is required" >&2; exit 1; }
test -f "$MASK" || { echo "missing $MASK — run: node scripts/gen-squircle.mjs" >&2; exit 1; }

mkdir -p "$OUT/icons"

# 1024 square source → squircle clip → opaque black flatten
magick "$SRC" -resize 1024x1024^ -gravity center -extent 1024x1024 "$WORK/square.png"
magick -background none "$MASK" -resize 1024x1024 "$WORK/mask.png"
magick "$WORK/square.png" "$WORK/mask.png" -alpha set -compose DstIn -composite "$WORK/squircle-alpha.png"
magick "$WORK/squircle-alpha.png" -background "$BRAND_BLACK" -alpha remove -alpha off "$WORK/squircle.png"

# Favicons + apple touch icon
magick "$WORK/squircle.png" -resize 16x16   "$OUT/favicon-16.png"
magick "$WORK/squircle.png" -resize 32x32   "$OUT/favicon-32.png"
magick "$WORK/squircle.png" -resize 180x180 "$OUT/apple-touch-icon.png"
magick "$WORK/squircle.png" -resize 48x48   "$WORK/favicon-48.png"
magick "$OUT/favicon-16.png" "$OUT/favicon-32.png" "$WORK/favicon-48.png" "$OUT/favicon.ico"

# Manifest icons
magick "$WORK/squircle.png" -resize 192x192 "$OUT/icons/icon-192.png"
magick "$WORK/squircle.png" -resize 512x512 "$OUT/icons/icon-512.png"

# Maskable icon: full-bleed on brand black with a safe zone, so the
# platform applies its own shape instead of double-rounding our squircle.
magick "$WORK/square.png" -resize 410x410 \
  -background "$BRAND_BLACK" -gravity center -extent 512x512 \
  "$OUT/icons/icon-512-maskable.png"

echo "icons written to $OUT"
