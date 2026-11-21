#!/bin/zsh
# Publishes the packages built by scripts/package-macos.sh as a release of
# brainupstudio/kernship:
#
#   scripts/publish-release.sh notes.md        release v<version> with these notes
#
# Attached: the disk images and zips of this version, SHA256SUMS.txt and install.sh, so that
# .../releases/latest/download/install.sh always installs the newest version.
# Needs a GitHub token for the personal account: GITHUB_TOKEN, or the one git keeps for
# github.com (git credential).
set -euo pipefail
cd "${0:A:h}/.."

REPO=brainupstudio/kernship
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
TAG="v$VERSION"
NOTES=${1:?"usage: scripts/publish-release.sh notes.md"}
DIST=dist

FILES=("$DIST/Kernship-$VERSION-arm64.dmg" "$DIST/Kernship-$VERSION-x86_64.dmg"
       "$DIST/Kernship-$VERSION-arm64.zip" "$DIST/Kernship-$VERSION-x86_64.zip"
       "$DIST/SHA256SUMS.txt" "scripts/install.sh")
for f in $FILES; do [[ -f "$f" ]] || { echo "Missing $f: run scripts/package-macos.sh first." >&2; exit 1; }; done
(cd "$DIST" && shasum -a 256 -c SHA256SUMS.txt >/dev/null) || { echo "SHA256SUMS.txt does not match the packages." >&2; exit 1; }

TOKEN=${GITHUB_TOKEN:-$(printf 'protocol=https\nhost=github.com\nusername=brainupstudio\n\n' | git credential fill 2>/dev/null | sed -n 's/^password=//p')}
[[ -n "$TOKEN" ]] || { echo "No GitHub token: set GITHUB_TOKEN." >&2; exit 1; }
api() { curl -fsS -H "Authorization: Bearer $TOKEN" -H "Accept: application/vnd.github+json" "$@"; }

# A draft first: "latest", install.sh and the test workflow see the release only once every
# file is attached (published straight away, it would be "latest" with nothing to download).
BODY=$(python3 -c 'import json,sys; print(json.dumps({"tag_name": sys.argv[1], "name": "Kernship " + sys.argv[2], "body": open(sys.argv[3]).read(), "draft": True}))' "$TAG" "$VERSION" "$NOTES")
ID=$(api -X POST "https://api.github.com/repos/$REPO/releases" --data "$BODY" | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
echo "Release $TAG drafted ($ID)"
for f in $FILES; do
    name=${f:t}
    case $name in
        *.dmg) type=application/x-apple-diskimage ;;
        *.zip) type=application/zip ;;
        *) type=text/plain ;;
    esac
    api -X POST -H "Content-Type: $type" --data-binary "@$f" \
        "https://uploads.github.com/repos/$REPO/releases/$ID/assets?name=$name" -o /dev/null
    echo "  $name"
done
api -X PATCH "https://api.github.com/repos/$REPO/releases/$ID" --data '{"draft": false}' -o /dev/null
echo "Published: https://github.com/$REPO/releases/tag/$TAG"
