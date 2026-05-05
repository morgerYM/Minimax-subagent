#!/usr/bin/env bash
set -euo pipefail

BUMP="${1:-patch}"
VERSION_FILE="${2:-VERSION}"

CURRENT=$(cat "$VERSION_FILE")
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT"

case "$BUMP" in
  major)
    MAJOR=$((MAJOR + 1))
    MINOR=0
    PATCH=0
    ;;
  minor)
    MINOR=$((MINOR + 1))
    PATCH=0
    ;;
  patch)
    PATCH=$((PATCH + 1))
    ;;
  *)
    echo "Usage: $0 {major|minor|patch} [version_file]"
    exit 1
    ;;
esac

NEW="$MAJOR.$MINOR.$PATCH"
echo "$NEW" > "$VERSION_FILE"
echo "Version bumped: $CURRENT → $NEW"
