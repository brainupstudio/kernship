# Kernship

*Kerning + Spaceship: a command centre for your font galaxy.*

A font manager for macOS: libraries and sets, Google Fonts, icon fonts, activation for design
apps (with auto-activation), and web-font conversion (WOFF2, WOFF, TTF/OTF, EOT, @font-face CSS).

## Install

In Terminal, paste:

```sh
curl -fsSL https://github.com/brainupstudio/kernship/releases/latest/download/install.sh | sh
```

It installs the latest version in Applications, for Apple Silicon or Intel, whichever your Mac
is, checks the download against its published checksum, and opens Kernship. macOS shows no
warning this way. Run the same command to update: your fonts, libraries and settings stay
where they are.

### Or download it

From [Releases](https://github.com/brainupstudio/kernship/releases/latest):
`Kernship-<version>-arm64.dmg` for Apple Silicon (M1 and later), `Kernship-<version>-x86_64.dmg`
for Intel. Open the disk image and drag Kernship to Applications.

The first time, macOS says it cannot verify Kernship: press **Done**, open
**System Settings › Privacy & Security** and, at the bottom, press **Open Anyway**. You need to
do this only once. (Right-click › Open no longer works for this since macOS 15.)

Kernship needs macOS 11 or later.

## Updates

Kernship checks once a day whether a newer version is out and says so in the sidebar and in
**Settings › General › Updates**. The check asks GitHub only for the number of the latest
version; nothing about you or your fonts is sent. It can be turned off there.

`SHA256SUMS.txt` in each release lists the checksums of its packages.
