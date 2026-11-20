#!/bin/sh
# Installs or updates Kernship on macOS:
#
#   curl -fsSL https://github.com/brainupstudio/kernship/releases/latest/download/install.sh | sh
#
# It downloads the latest release from github.com/brainupstudio/kernship (the Apple Silicon or
# the Intel build, whichever this Mac is), checks it against the published SHA-256 checksums,
# and puts Kernship.app in /Applications (KERNSHIP_DIR to choose another folder). A file
# fetched by curl is not marked as downloaded from the internet, so macOS opens the app
# without the "cannot verify" warning shown for downloads from a browser.
# Fonts, libraries and settings are never touched: they live in their own folders.
#
# Everything is inside main(), so a download cut short runs nothing. Plain ASCII only: with
# the C locale, sh would read a multibyte character after $name as part of the name.

main() {
    set -eu
    REPO="brainupstudio/kernship"

    say() { printf '%s\n' "$*"; }
    fail() { printf 'Kernship: %s\n' "$*" >&2; exit 1; }
    need() { command -v "$1" >/dev/null 2>&1 || fail "this needs '$1', which is not installed."; }

    [ "$(uname -s)" = Darwin ] || fail "Kernship is a Mac app: this script runs on macOS only."
    need curl
    need mktemp
    need hdiutil

    # Apple Silicon or Intel. A shell running under Rosetta says x86_64, so ask the hardware.
    if [ "$(sysctl -n hw.optional.arm64 2>/dev/null || echo 0)" = 1 ]; then
        arch=arm64
    else
        arch=x86_64
    fi

    # the latest release: GitHub redirects .../releases/latest to .../releases/tag/v<version>
    tag_url=$(curl -fsSLI -o /dev/null -w '%{url_effective}' "https://github.com/${REPO}/releases/latest") || fail "no connection to GitHub."
    tag=${tag_url##*/}
    version=${tag#v}
    case "${version}" in
        [0-9]*.[0-9]*) ;;
        *) fail "could not find the latest version." ;;
    esac
    file="Kernship-${version}-${arch}.dmg"
    base="https://github.com/${REPO}/releases/download/${tag}"

    tmp=$(mktemp -d)
    mnt=""
    cleanup() {
        if [ -n "${mnt}" ]; then hdiutil detach "${mnt}" -quiet >/dev/null 2>&1 || true; fi
        rm -rf "${tmp}"
    }
    trap cleanup EXIT INT TERM

    say "Downloading Kernship ${version} for ${arch}..."
    curl -fL --progress-bar -o "${tmp}/${file}" "${base}/${file}" || fail "the download did not finish."
    curl -fsSL -o "${tmp}/SHA256SUMS.txt" "${base}/SHA256SUMS.txt" || fail "the checksums of version ${version} are not published, so the download cannot be checked."

    expected=$(awk -v f="${file}" '$2 == f || $2 == "*"f { print $1 }' "${tmp}/SHA256SUMS.txt")
    [ -n "${expected}" ] || fail "${file} is not in the published checksums."
    actual=$(shasum -a 256 "${tmp}/${file}" | awk '{ print $1 }')
    [ "${actual}" = "${expected}" ] || fail "the downloaded file does not match its checksum. Nothing was installed; try again."
    say "Checked: the file is the published one."

    dir="${KERNSHIP_DIR:-/Applications}"
    app="${dir}/Kernship.app"
    mnt="${tmp}/volume"
    mkdir -p "${mnt}"
    hdiutil attach "${tmp}/${file}" -nobrowse -readonly -quiet -mountpoint "${mnt}" || { mnt=""; fail "could not open the disk image."; }
    [ -d "${mnt}/Kernship.app" ] || fail "the disk image does not contain Kernship.app."

    # the copy being replaced must not be running (only that copy, found by its path)
    running() { pgrep -f "${app}/Contents/MacOS/Kernship" >/dev/null 2>&1; }
    if running; then
        say "Closing Kernship to replace it..."
        # by path, not by bundle id: another Kernship elsewhere must keep running
        pkill -TERM -f "${app}/Contents/MacOS/Kernship" >/dev/null 2>&1 || true
        i=0
        while running && [ "${i}" -lt 20 ]; do sleep 0.5; i=$((i + 1)); done
        if running; then fail "Kernship is still open: quit it (right-click its menu bar icon > Quit Kernship) and run this again."; fi
    fi

    mkdir -p "${dir}"
    sudo=""
    if [ ! -w "${dir}" ] || { [ -e "${app}" ] && [ ! -w "${app}" ]; }; then
        say "${dir} needs your password to write Kernship there."
        sudo="sudo"
    fi
    ${sudo} rm -rf "${app}"
    ${sudo} ditto "${mnt}/Kernship.app" "${app}"
    # in case the copy came marked anyway: the app is the published one, checked above
    ${sudo} xattr -dr com.apple.quarantine "${app}" 2>/dev/null || true
    say "Kernship ${version} is in ${dir}."
    if [ "${dir}" = /Applications ]; then
        open "${app}" || true
    fi
}

main "$@"
