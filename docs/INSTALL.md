# Install

Download files from the [latest release](https://github.com/unupunct/LocalSendStudio/releases/latest). Each release also has a `SHA256SUMS.txt` to verify downloads:

```bash
sha256sum -c SHA256SUMS.txt --ignore-missing      # Linux / macOS (shasum -a 256 -c on macOS)
```
```powershell
Get-FileHash .\LocalSendStudio_*_x64-setup.exe -Algorithm SHA256   # Windows, compare with SHA256SUMS.txt
```

After installing, open the app on **two devices on the same Wi-Fi or LAN**. They appear on each other's Home screen within a few seconds. The first time one connects to the other, you'll be asked to allow it.

## Windows

1. Download `LocalSendStudio_<version>_x64-setup.exe` (per-user installer, no admin rights needed) or the `.msi`.
2. Run it. The builds are **not code-signed**, so Windows SmartScreen says *"Windows protected your PC"*. Click **More info → Run anyway**.
3. When Windows Firewall asks, allow **private networks**. If it doesn't ask and devices can't see each other, see [Troubleshooting](TROUBLESHOOTING.md#windows-firewall).

Uninstall from *Settings → Apps*.

## macOS

1. Download `LocalSendStudio_<version>_universal.dmg` (one file for Intel and Apple silicon). Needs macOS 10.15 or newer.
2. Open the dmg and drag **LocalSendStudio** to **Applications**.
3. The app is **not notarized by Apple**, so the first launch is blocked. Do one of:
   - In Finder, **right-click the app → Open → Open**, or
   - Run once in Terminal:
     ```bash
     xattr -dr com.apple.quarantine /Applications/LocalSendStudio.app
     ```
4. Allow **incoming connections** when macOS asks, and allow **Local Network** access if prompted (System Settings → Privacy & Security → Local Network).

## Linux

The short version:

```bash
curl -fsSL https://raw.githubusercontent.com/unupunct/LocalSendStudio/main/scripts/install-linux.sh | bash
```

It picks `.deb`, `.rpm` or AppImage for your system, checks the SHA-256, installs, and offers to open the firewall ports. Full walkthrough, manual installs, firewall and uninstall: **[INSTALL-LINUX.md](INSTALL-LINUX.md)**.

## Android

Requires Android 7.0 or newer.

1. On the phone, open the release page in a browser and download `LocalSendStudio_<version>_android.apk`.
2. Open the file. Android asks to allow installs from that browser (*Install unknown apps*) — allow it, then install.
3. On first launch the app opens a system screen asking for **All files access**. Turn it on for LocalSendStudio. This is how it can read files you choose to send and save received files to `Download/LocalSendStudio`. The app has no other use for it.
4. Keep the app open while you expect to receive files: Android stops background apps from listening on the network.

Notes:
- The APK is signed with the project's release key but isn't distributed through Google Play, so Play Protect may show *"unrecognized developer"*. Choose **Install anyway**.
- Android picks files through the app's own file browser (not the system picker), because the system picker doesn't give apps real file paths.
- To update, install the new APK over the old one.

## iOS

Not available.

## Updating

Download and run the newer installer; it replaces the old version and keeps your settings. On Linux, re-run the install script.

## Where settings are stored

| OS | Folder |
|---|---|
| Windows | `%APPDATA%\LocalSendStudio` |
| macOS | `~/Library/Application Support/LocalSendStudio` |
| Linux | `~/.local/share/LocalSendStudio` |
| Android | app-private storage |

Received files go to your **Downloads/LocalSendStudio** folder unless you change it in Settings.
