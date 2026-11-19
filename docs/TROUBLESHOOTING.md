# Troubleshooting

Start with the app's **Network** page. It runs the checks below and says what failed.

## I can't see the other device

1. Both devices must be on the **same network**. Guest Wi-Fi, "AP isolation" and hotspots often stop devices talking to each other.
2. **VPNs, Docker, Hyper-V and VM adapters** can send discovery packets out the wrong interface. Disconnect the VPN briefly.
3. **Firewall**: allow TCP 47821–47840 and UDP 47822 on the local network (see below).
4. **Add by IP**: on the Devices page enter the other device's IP (shown on its Home screen). This works even when discovery is blocked; the TCP port must still be reachable.
5. A router that blocks multicast/broadcast (some mesh systems) only affects discovery; Add by IP still works.

## Windows firewall

On first start Windows asks whether to allow the app. Tick **Private networks**. If you dismissed it:

*Windows Security → Firewall & network protection → Allow an app through firewall → Change settings → find LocalSendStudio → tick Private.*

Or, in an **administrator** PowerShell:

```powershell
New-NetFirewallRule -DisplayName "LocalSendStudio" -Direction Inbound -Protocol TCP -LocalPort 47821-47840 -Profile Private -Action Allow
New-NetFirewallRule -DisplayName "LocalSendStudio discovery" -Direction Inbound -Protocol UDP -LocalPort 47822 -Profile Private -Action Allow
```

If your network is marked **Public**, change it to Private (*Settings → Network & internet → Wi-Fi → your network → Private*).

## Linux firewall

```bash
sudo ufw allow from 192.168.0.0/16 to any port 47821:47840 proto tcp
sudo ufw allow from 192.168.0.0/16 to any port 47822 proto udp
# firewalld:
sudo firewall-cmd --permanent --add-port=47821-47840/tcp --add-port=47822/udp && sudo firewall-cmd --reload
```

## macOS

Allow **incoming connections** when asked, and enable the app under *System Settings → Privacy & Security → Local Network*. If the app is blocked at launch, see the [macOS steps](INSTALL.md#macos).

## "not authorized" / the other side never answers

The receiving device must tap **Allow**. The prompt shows on that device and expires after 45 seconds. If it was denied, the sender is ignored for 30 seconds, then asked again. To trust a device permanently, tick *Always allow*; to undo it, use *Settings → Approved devices → Revoke*.

## "uploading is not allowed on this folder"

The folder's access level doesn't include uploads. On the owning device: *Shared folders →* choose **View & upload**, **Drop box** or **Full control**. The automatic **Inbox** already allows uploads.

## A transfer fails with "checksum mismatch"

The file changed on the sender during the transfer, or the network corrupted it. Nothing is saved when this happens. Try again; if it keeps failing, run the speed test on the Network page and check for a flaky Wi-Fi link.

## An interrupted download

Start the same download again. If a `<name>.lsspart` file is in the destination folder, it continues where it stopped (and restarts from zero if the source file changed).

## Android

- Turn on **All files access** for the app (Settings → Apps → LocalSendStudio → Permissions), or it can't read files to send or save received ones.
- **Keep the app open** to receive. Android pauses background network use.
- No devices found: make sure Wi-Fi (not mobile data) is on, and that the router doesn't isolate clients. Add by IP as a fallback.

## Linux AppImage won't start

Install FUSE 2 (`sudo apt install libfuse2` / `libfuse2t64` on Ubuntu 24.04), or run `./LocalSendStudio_*.AppImage --appimage-extract-and-run`.

## No tray icon on GNOME

GNOME hides tray icons by default. Install the *AppIndicator and KStatusNotifierItem Support* extension, or turn off *Keep running in the background* in Settings.

## Logs

Run from a terminal with `RUST_LOG=debug` to see what the core is doing:

```bash
RUST_LOG=debug localsendstudio          # Linux package
RUST_LOG=debug ./LocalSendStudio.AppImage
```

## Still stuck?

Open an issue at <https://github.com/unupunct/LocalSendStudio/issues> with your OS, app version (Settings → bottom of page) and the result of the **Network** checks. Don't include file names or private IP addresses you'd rather keep private.
