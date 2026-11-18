# Installation Guide

## Windows

- Download installer from Releases.
- Run setup. If SmartScreen appears, choose "More info" → "Run anyway".
- Launch from Start Menu.

## macOS

Download the `.app` bundle or `.dmg` from Releases (when available) and drag to Applications. Or build from source:

```bash
# Install prerequisites: Xcode Command Line Tools, Rust, Node.js
xcode-select --install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# build
git clone https://github.com/unupunct/LocalSendStudio.git
cd LocalSendStudio
npm install
npm run tauri build
```

The bundle will be in `src-tauri/target/release/bundle/macos/`.

## Linux

Debian/Ubuntu:
```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://github.com/unupunct/LocalSendStudio.git
cd LocalSendStudio
npm install
npm run tauri build
```
Packages: `.deb` and `.AppImage` in `src-tauri/target/release/bundle/`.

## Android

Prerequisites: Android Studio, Android SDK, NDK, JDK 17, Rust Android targets.

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
npm install
npm run tauri android init
npm run tauri android build
```

The APK will be in `src-tauri/gen/android/app/build/outputs/apk/`. A prebuilt APK is published in Releases when available.

## Troubleshooting

- Devices not discovered: check firewall allows UDP 47822 and TCP 47821. Ensure same subnet.
- Multicast blocked: fallback broadcast may work; check router AP isolation.
- Port in use: change port in Settings.
