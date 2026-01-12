#!/usr/bin/env sh
set -eu

if [ $# -ne 1 ]; then
    echo "Usage: $0 <tag>"
    echo "Example: $0 1.6.0"
    exit 1
fi

TAG="$1"
REPO_URL="https://github.com/braiins/bos-plus-api.git"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
DEST_DIR="$ROOT_DIR/meta/braiins/proto/$TAG"

if [ -e "$DEST_DIR" ]; then
    echo "Error: $DEST_DIR already exists"
    exit 1
fi

echo "Loading proto files for braiins/bos-plus-api @ $TAG"
mkdir -p "$DEST_DIR"

# Clone full repo
git clone "$REPO_URL" "$DEST_DIR"

cd "$DEST_DIR"
git checkout "tags/$TAG"

# Remove git metadata to make snapshot immutable
rm -rf .git

# Remove unneeded files
rm -rf .gitignore README.md

cd "$ROOT_DIR"
git add "meta/braiins/proto/$TAG"

# Commit ONLY this directory, regardless of working tree state
git commit "meta/braiins/proto/$TAG" \
    -m "proto: add braiins proto files for version $TAG"


echo "Done."
echo "Proto files are in: meta/braiins/proto/$TAG/proto"
