# LocalSendStudio

Universal local network file sharing, synchronization and device management.

**Install. Open. Discover. Share.**

LocalSendStudio turns computers and mobile devices on the same LAN into one interconnected file-sharing environment. No cloud accounts, no mandatory internet connection, no manual IP configuration, no pairing procedures.

## Features

- Open Network Mode by default: automatic discovery, automatic connections, no pairing.
- mDNS/UDP multicast discovery with fallback.
- Device dashboard with cards and filters.
- Shared folder management with per-folder permissions.
- Remote file browser (browse, upload, download, mkdir, rename, delete).
- High-performance chunked transfers with SHA-256 verification.
- Transfer queue, progress, speed, ETA, cancel.
- Network topology visualization (map + list).
- Quick Share to one/all/group of devices.
- Device groups.
- Global Explorer aggregating all shared folders.
- Network search, diagnostics, analytics.
- Clipboard sharing toggle, duplicate detection (configurable).
- System tray, background sharing service.
- Optional synchronization (one-way copy).
- Storage monitoring and transfer history (CSV export).

## Platforms

- Windows 10/11 (installer provided)
- macOS, Linux (build from source)
- Android, iOS (planned; core reusable)

## Install

### Windows

1. Download `LocalSendStudio_0.1.0_x64-setup.exe` (NSIS) or `.msi` from [Releases](https://github.com/unupunct/LocalSendStudio/releases).
2. Run the installer. Windows SmartScreen may warn; click "More info" → "Run anyway" (unsigned build).
3. Launch LocalSendStudio. It will discover other devices on your LAN automatically.

### Build from source

Prerequisites:
- Rust (stable), Node.js 20+, npm
- Tauri v2 prerequisites for your OS (see https://tauri.app/start/prerequisites/)

```bash
git clone https://github.com/unupunct/LocalSendStudio.git
cd LocalSendStudio
npm install
npm run tauri build
```

The installer will be in `src-tauri/target/release/bundle/`.

## Usage

1. Open the app on each device on the same LAN.
2. Devices appear on the Dashboard and Network Map within seconds.
3. Add shared folders in **Shared Folders**.
4. Browse any device in **Devices** or **Global Explorer**.
5. Drag files onto a device card or use **Quick Share**.

## Security

Open Network Mode trusts any device that can reach the app on the LAN. Only explicitly configured folders are exposed. Directory traversal is prevented. The service is not exposed to the internet by default. Use Restricted Mode for authorization/encryption.

## Documentation

- `docs/ARCHITECTURE.md`
- `docs/PROTOCOL.md`
- `docs/INSTALL.md`
- `docs/TROUBLESHOOTING.md`

## License

MIT


