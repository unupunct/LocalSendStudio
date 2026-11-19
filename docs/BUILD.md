# Building

## Common

Rust (stable), Node.js 20+, npm.

```bash
npm install
npm run tauri dev           # development
npm run tauri build         # release bundles for the current OS
cd src-tauri && cargo test --lib && cargo clippy --lib -- -D warnings
```

Bundles are written to `src-tauri/target/release/bundle/`.

## Windows

Install the [Tauri prerequisites](https://tauri.app/start/prerequisites/#windows) (Visual Studio Build Tools with the C++ workload; WebView2 is built into Windows 11). `npm run tauri build` produces an NSIS `*-setup.exe` and a `.msi`.

## macOS

Xcode command line tools. For one universal build:

```bash
rustup target add aarch64-apple-darwin x86_64-apple-darwin
npm run tauri build -- --target universal-apple-darwin
```

Output: `…/universal-apple-darwin/release/bundle/dmg/*.dmg`. It is not signed or notarized; see [INSTALL.md](INSTALL.md#macos) for how users open it. To sign, set the `APPLE_*` variables described in the [Tauri docs](https://tauri.app/distribute/sign/macos/).

## Linux

```bash
# Debian / Ubuntu
sudo apt install libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libsoup-3.0-dev libgtk-3-dev patchelf rpm build-essential curl
npm run tauri build            # .deb, .rpm and .AppImage
```

On Fedora: `sudo dnf install webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel libsoup3-devel gtk3-devel patchelf rpm-build`.

## Android

Prerequisites: JDK 17, Android SDK (platform 36, build-tools 36, NDK 26.1.10909125) and the Rust targets.

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi
export ANDROID_HOME=/path/to/android-sdk
export NDK_HOME=$ANDROID_HOME/ndk/26.1.10909125
npx tauri android build --apk --target aarch64 --target armv7
```

The result is an **unsigned** APK at
`src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk`. Phones refuse unsigned APKs, so sign it:

```bash
# once: create a key and keep it somewhere safe and OUT of git. Losing it means users can't upgrade in place.
keytool -genkeypair -v -keystore release.jks -alias localsendstudio -keyalg RSA -keysize 4096 -validity 10000

$ANDROID_HOME/build-tools/36.0.0/zipalign -p -f 4 app-universal-release-unsigned.apk aligned.apk
$ANDROID_HOME/build-tools/36.0.0/apksigner sign --ks release.jks --ks-key-alias localsendstudio --out LocalSendStudio.apk aligned.apk
$ANDROID_HOME/build-tools/36.0.0/apksigner verify --print-certs LocalSendStudio.apk
```

`src-tauri/gen/android` is committed (manifest permissions, `MainActivity.kt` with the multicast lock). Generated build output inside it is git-ignored.

## Releases (CI)

Pushing a tag `vX.Y.Z` runs `.github/workflows/release.yml`: it builds Windows, macOS (universal) and Linux packages, creates a **draft** release, and adds `SHA256SUMS.txt`. Review the draft, attach the signed APK, then publish.

Optional: set the repository variable `BUILD_ANDROID=true` and the secrets `ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS` to let CI build and sign the APK too.

Before tagging, bump the version in `package.json`, `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json`.

## UI preview without the Rust core

```bash
VITE_DEMO=1 npm run dev     # http://localhost:1420 with fake devices and transfers
```

Add `?live` to show active transfers or `?approve` to show the approval prompt.
