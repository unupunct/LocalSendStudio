# Changelog

## 0.2.0

A security and reliability rewrite of the core, a new design, and builds for every platform.

### Security
- New devices must be **approved** before they can access anything (Ask first is the default). Approved devices are remembered and can be revoked.
- **Permissions are enforced.** 0.1.x let any device list and download from a share even when *Read* was off, and `rename` could move files outside the share.
- Path checks now block `..`, drive letters, NTFS streams, Windows reserved names, and symlinks that leave a shared folder.
- New shared folders default to **read-only** (were full control).
- Network frames are capped at 4 MiB (a single bad packet could previously allocate 4 GiB). Stalled connections time out.
- Uploads never overwrite existing files, and the share root can't be deleted remotely.

### Transfers
- **SHA-256 verified end to end** (0.1.x computed a hash but never checked it).
- Files are written to `.lsspart` and moved into place only when complete and correct.
- **Interrupted downloads resume.**
- Uploads are confirmed by the receiver, appear in the transfer list and history, and show speed and time left.
- Streaming hash: no more reading the whole file before sending it.

### Discovery
- Announces on every network interface (multicast + directed broadcast), not just one.
- Devices now go **offline** and disappear when they stop responding (0.1.x showed them forever).
- **Add by IP** for networks that block discovery.
- If the port is busy, the next free one is used.

### New
- **Inbox**: a receive-only folder created on first run so sending works immediately.
- Four access levels per shared folder.
- Network checks with plain-language fixes, and a real speed test.
- Recursive one-way sync that skips files that already match.
- System tray with close-to-tray, single-instance, light/dark theme, device rename that works.
- Disk space readout (was a placeholder).

### Design
- Complete UI redesign: responsive (desktop sidebar, phone tab bar), light and dark themes, empty states, toasts and dialogs instead of browser `alert/prompt/confirm` (which don't work in the app window).

### Platforms
- Windows installer (.exe and .msi), macOS universal .dmg, Linux .deb / .rpm / .AppImage with an install script, and an Android APK.
- GitHub Actions: CI on three operating systems, tagged releases with checksums.

### Removed
- Claims for features that never existed (clipboard sharing, duplicate detection, "Restricted Mode" encryption). The settings switch for them did nothing.
- The recharts dependency (charts are now plain CSS).

### Compatibility
- Protocol version 2. 0.1.x and 0.2.x devices can't talk to each other; update both.

## 0.1.0

First release.
