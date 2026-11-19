# Installing on Linux

Three ways, from easiest to most manual. Supported: x86-64 and arm64.

| Distribution family | Best option |
|---|---|
| Ubuntu, Debian, Linux Mint, Pop!_OS, elementary | `.deb` |
| Fedora, RHEL, Rocky, Alma, openSUSE | `.rpm` |
| Arch, Manjaro, NixOS, anything else | AppImage |

## 1. Install script (recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/unupunct/LocalSendStudio/main/scripts/install-linux.sh | bash
```

What it does, in order:

1. Finds the latest release for your CPU.
2. Chooses `.deb` (apt), `.rpm` (dnf, zypper or yum) or the AppImage, depending on your system.
3. Downloads it and **verifies the SHA-256** against the release's `SHA256SUMS.txt`. A mismatch aborts.
4. Installs it. Package installs ask for `sudo`; the AppImage installs into `~/.local/bin` with a menu entry and needs no root.
5. If a firewall (ufw or firewalld) is active, offers to open the ports the app needs.

Read it first if you prefer: [scripts/install-linux.sh](../scripts/install-linux.sh).

Options (append `-s -- <option>` when piping):

```bash
curl -fsSL https://raw.githubusercontent.com/unupunct/LocalSendStudio/main/scripts/install-linux.sh | bash -s -- --appimage --open-firewall
```

| Option | Meaning |
|---|---|
| `--version v0.2.0` | Install that release instead of the latest |
| `--appimage` | Use the AppImage even on Debian/Fedora systems |
| `--open-firewall` | Open the ports without asking |
| `-y` | No questions |
| `--uninstall` | Remove LocalSendStudio |

## 2. Manual install

Download the file for your system from the [releases page](https://github.com/unupunct/LocalSendStudio/releases/latest).

**Debian, Ubuntu and derivatives**

```bash
sudo apt install ./LocalSendStudio_0.2.0_amd64.deb
```

**Fedora, RHEL and derivatives**

```bash
sudo dnf install ./LocalSendStudio-0.2.0-1.x86_64.rpm
```

**openSUSE**

```bash
sudo zypper --no-gpg-checks install ./LocalSendStudio-0.2.0-1.x86_64.rpm
```

**AppImage (any distribution)**

```bash
chmod +x LocalSendStudio_0.2.0_amd64.AppImage
./LocalSendStudio_0.2.0_amd64.AppImage
```

AppImages need FUSE 2. If it complains: `sudo apt install libfuse2` (Ubuntu 22.04+: `libfuse2t64`) or `sudo dnf install fuse fuse-libs`. Without FUSE you can still run it with `--appimage-extract-and-run`.

Package installs need the WebKitGTK 4.1 runtime. It is pulled in automatically on Ubuntu 22.04+, Debian 12+, Fedora 38+. On older systems use the AppImage, which bundles what it needs.

## 3. Open the firewall

LocalSendStudio needs **TCP 47821–47840** (file transfer) and **UDP 47822** (finding devices) open on your **local network only**.

```bash
# ufw (Ubuntu, Debian, Mint)
sudo ufw allow from 192.168.0.0/16 to any port 47821:47840 proto tcp
sudo ufw allow from 192.168.0.0/16 to any port 47822 proto udp
```

```bash
# firewalld (Fedora, RHEL, openSUSE)
sudo firewall-cmd --permanent --add-port=47821-47840/tcp --add-port=47822/udp
sudo firewall-cmd --reload
```

Change `192.168.0.0/16` if your network uses another range (`10.0.0.0/8`, `172.16.0.0/12`). The install script's `--open-firewall` does exactly this for ufw (all three private ranges). For firewalld it opens the ports in your default zone.

## 4. Start it

Open **LocalSendStudio** from your applications menu, or run `localsendstudio` (package installs) / `~/.local/bin/LocalSendStudio.AppImage`.

Closing the window keeps it running in the system tray so you can still receive files. On GNOME you need the *AppIndicator and KStatusNotifierItem Support* extension to see the tray icon. Without it, turn off *Keep running in the background* in Settings.

## Check it works

1. Install on a second device (any platform), same Wi-Fi.
2. Each should list the other under **Home → Nearby devices** within ~5 seconds.
3. If not, open **Network** in the app. It tells you what's wrong. Common causes are in [Troubleshooting](TROUBLESHOOTING.md).

## Uninstall

```bash
curl -fsSL https://raw.githubusercontent.com/unupunct/LocalSendStudio/main/scripts/install-linux.sh | bash -s -- --uninstall
```

or manually: `sudo apt remove localsendstudio` / `sudo dnf remove localsendstudio` / delete `~/.local/bin/LocalSendStudio.AppImage` and `~/.local/share/applications/localsendstudio.desktop`. Settings live in `~/.local/share/LocalSendStudio`; delete that folder to remove them.

## Building the packages yourself

See [BUILD.md](BUILD.md#linux).
