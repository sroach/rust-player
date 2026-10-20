#!/bin/bash
# Build and deploy the Dioxus frontend to frontend/dist/
# This script handles copying WASM/JS assets and injecting the CSS link into index.html

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
FRONTEND_DIR="$SCRIPT_DIR/frontend"
DIST_DIR="$FRONTEND_DIR/dist"
BUILD_DIR="$SCRIPT_DIR/target/dx/frontend/release/web/public"

echo "🔨 Building Dioxus frontend..."
cd "$FRONTEND_DIR"
dx build --platform web --release

echo "📦 Preparing dist directory..."
# Clean and recreate dist/assets to avoid stale files from previous builds
rm -rf "$DIST_DIR/assets"
mkdir -p "$DIST_DIR/assets"

# Extract the JS filename referenced in the generated index.html
JS_FILE=$(sed -n 's/.*src="\/\.\/assets\/\([^"]*\)".*/\1/p' "$BUILD_DIR/index.html" | head -1)

# Extract the WASM filename from the JS file content
WASM_FILE=$(grep -o 'frontend_bg[^"]*' "$BUILD_DIR/assets/$JS_FILE" | grep '\.wasm' | tail -1)

echo "📄 Current JS: $JS_FILE"
echo "📦 Current WASM: $WASM_FILE"

# Copy only the current build's JS and WASM files
cp "$BUILD_DIR/assets/$JS_FILE" "$DIST_DIR/assets/"
cp "$BUILD_DIR/assets/$WASM_FILE" "$DIST_DIR/assets/"

# Copy index.html
cp "$BUILD_DIR/index.html" "$DIST_DIR/index.html"

# Inject CSS link and update title if not already present
# The Dioxus-generated index.html doesn't include our custom CSS, so we inject it
if ! grep -q 'style.css' "$DIST_DIR/index.html"; then
    sed -i '' 's|<meta charset="UTF-8">|<meta charset="UTF-8">\n        <link rel="stylesheet" href="/style.css">|' "$DIST_DIR/index.html"
fi

# Update title if it's still the default Dioxus title
if grep -q 'dioxus' "$DIST_DIR/index.html"; then
    sed -i '' 's/<title>[^<]*<\/title>/<title>Caribbean Radio Player<\/title>/' "$DIST_DIR/index.html"
fi

# Add lang="en" to html tag if missing
sed -i '' 's|<html>|<html lang="en">|' "$DIST_DIR/index.html"

echo "✅ Frontend deployed to $DIST_DIR"
echo "📋 Files in dist/assets:"
ls -lh "$DIST_DIR/assets/"
