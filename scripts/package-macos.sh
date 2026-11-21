#!/bin/zsh
# Builds Kernship for release: an .app, a disk image and a zip for each chosen architecture,
# into dist/<arch>/ (the app) and dist/ (the files to hand out).
#
#   scripts/package-macos.sh              Apple Silicon and Intel, one package each
#   scripts/package-macos.sh universal    one package that runs natively on both
#   scripts/package-macos.sh arm64 universal x86_64 …   any mix of arm64, x86_64, universal
#
#   SIGN_ID="Developer ID Application: …" scripts/package-macos.sh
#                                         signs with that identity instead of ad-hoc
#                                         (ad-hoc runs on this Mac; other Macs ask the user
#                                         to allow it once, see README)
#
# The version comes from Cargo.toml. Apple Silicon and Intel builds need rustup with the
# aarch64-apple-darwin and x86_64-apple-darwin targets; without rustup only the installed
# Rust's own architecture can be built.
set -euo pipefail
cd "${0:A:h}/.."

NAME=Kernship
BUNDLE_ID=com.brainupstudio.kernship
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
DIST=dist
WORK="$DIST/.work"
VARIANTS=("$@")
(( ${#VARIANTS} )) || VARIANTS=(arm64 x86_64)

# rustup's toolchain, when installed, comes before any other Rust on the PATH.
[[ -x "$HOME/.cargo/bin/rustup" ]] && export PATH="$HOME/.cargo/bin:$PATH"
export CARGO_RESOLVER_INCOMPATIBLE_RUST_VERSIONS=fallback

triple() {
    case "$1" in
        arm64) echo aarch64-apple-darwin ;;
        x86_64) echo x86_64-apple-darwin ;;
        *) echo "Unknown architecture: $1 (use arm64, x86_64 or universal)" >&2; exit 2 ;;
    esac
}

# The program for one architecture; prints the path of the binary.
build() {
    local t=$(triple "$1")
    if command -v rustup >/dev/null && rustup target list --installed | grep -qx "$t"; then
        cargo build --release --target "$t" >&2
        echo "target/$t/release/kernship"
    elif [[ "$(rustc -vV | sed -n 's/^host: //p')" == "$t" ]]; then
        cargo build --release >&2
        echo "target/release/kernship"
    else
        echo "Cannot build for $1: install rustup and run: rustup target add $t" >&2
        exit 2
    fi
}

rm -rf "$WORK"
mkdir -p "$WORK"

# The icon, made once: the same Dock icon the app draws (src/brand.rs `icon`), at 1024 px,
# turned into every size macOS asks for.
KERNSHIP_ICON_PREVIEW="$PWD/$WORK" cargo test --release --quiet brand::tests::dock_icon >/dev/null
ICONSET="$WORK/$NAME.iconset"
mkdir -p "$ICONSET"
for s in 16 32 128 256 512; do
    sips -z $s $s "$WORK/dock-icon.png" --out "$ICONSET/icon_${s}x${s}.png" >/dev/null
    sips -z $((s * 2)) $((s * 2)) "$WORK/dock-icon.png" --out "$ICONSET/icon_${s}x${s}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$WORK/$NAME.icns"

# Entitlements for a real signing identity: the hardened runtime (needed for notarization)
# with the right to send Apple events to design apps.
ENT="$WORK/entitlements.plist"
cat > "$ENT" <<ENTPLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict><key>com.apple.security.automation.apple-events</key><true/></dict></plist>
ENTPLIST

OUTPUTS=()
for VARIANT in $VARIANTS; do
    APP="$DIST/$VARIANT/$NAME.app"
    rm -rf "$APP"
    mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"

    # 1. The program.
    if [[ "$VARIANT" == universal ]]; then
        lipo -create "$(build arm64)" "$(build x86_64)" -output "$APP/Contents/MacOS/$NAME"
    else
        cp "$(build "$VARIANT")" "$APP/Contents/MacOS/$NAME"
    fi
    ARCHS=$(lipo -archs "$APP/Contents/MacOS/$NAME")
    echo "Kernship $VERSION, $VARIANT ($ARCHS)"

    # 2. The icon and what macOS needs to know about the app.
    cp "$WORK/$NAME.icns" "$APP/Contents/Resources/$NAME.icns"
    cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key><string>$NAME</string>
    <key>CFBundleDisplayName</key><string>$NAME</string>
    <key>CFBundleExecutable</key><string>$NAME</string>
    <key>CFBundleIdentifier</key><string>$BUNDLE_ID</string>
    <key>CFBundleIconFile</key><string>$NAME</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleShortVersionString</key><string>$VERSION</string>
    <key>CFBundleVersion</key><string>$VERSION</string>
    <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
    <key>LSApplicationCategoryType</key><string>public.app-category.graphics-design</string>
    <key>LSMinimumSystemVersion</key><string>11.0</string>
    <key>NSHighResolutionCapable</key><true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key><true/>
    <key>NSAppleEventsUsageDescription</key>
    <string>Kernship asks design apps which fonts their open documents use, so it can activate the missing ones, and opens file and folder choosers.</string>
    <key>NSHumanReadableCopyright</key><string>© $(date +%Y) Luca Pagot</string>
</dict>
</plist>
PLIST
    printf 'APPL????' > "$APP/Contents/PkgInfo"

    # 3. Signature: ad-hoc unless an identity is given.
    if [[ -n "${SIGN_ID:-}" ]]; then
        codesign --force --options runtime --timestamp --entitlements "$ENT" --sign "$SIGN_ID" "$APP"
    else
        codesign --force --sign - "$APP"
    fi
    codesign --verify --strict "$APP"

    # 4. Disk image (drag to Applications) and a zip of the app.
    DMG="$DIST/$NAME-$VERSION-$VARIANT.dmg"
    ZIP="$DIST/$NAME-$VERSION-$VARIANT.zip"
    STAGE="$WORK/dmg-$VARIANT"
    mkdir -p "$STAGE"
    cp -R "$APP" "$STAGE/"
    ln -s /Applications "$STAGE/Applications"
    rm -f "$DMG" "$ZIP"
    hdiutil create -quiet -volname "$NAME $VERSION" -srcfolder "$STAGE" -fs HFS+ -format UDZO "$DMG"
    ditto -c -k --keepParent "$APP" "$ZIP"
    OUTPUTS+=("$APP" "$DMG  ($(du -h "$DMG" | cut -f1 | tr -d ' '))" "$ZIP  ($(du -h "$ZIP" | cut -f1 | tr -d ' '))")
done
rm -rf "$WORK"

# Checksums of every package of this version, for install.sh and anyone who checks by hand.
(cd "$DIST" && shasum -a 256 "$NAME-$VERSION"-*.dmg "$NAME-$VERSION"-*.zip > SHA256SUMS.txt)
OUTPUTS+=("$DIST/SHA256SUMS.txt")

echo
for o in $OUTPUTS; do echo "  $o"; done
