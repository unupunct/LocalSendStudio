# Security

This describes what LocalSendStudio protects against, what it doesn't, and how to report problems.

## Model

LocalSendStudio is meant for a **home or small-office network you mostly trust**. It listens on the local network (TCP 47821+, UDP 47822) and never contacts the internet.

### What is enforced

| Protection | How |
|---|---|
| Unknown devices are refused | Every request except `info` requires the peer to be approved. In *Ask first* mode (default) the owner sees an Allow/Deny prompt; no answer in 45 s counts as Deny, and a denied device isn't asked again for 30 s. |
| Only shared folders are reachable | Shares are an explicit list; a request names a share and a relative path inside it. |
| Per-folder permissions | *Read*, *Upload* and *Manage* are checked on the server for each request. New folders default to read-only. |
| Path safety | Names containing `..`, separators, `:` (drive letters, NTFS streams), `<>"|?*`, control characters, trailing dots/spaces or Windows device names (`CON`, `NUL`, …) are rejected. The resolved path's deepest existing ancestor is canonicalised and must remain inside the share, which blocks symlink escapes. |
| No overwrite by uploads | An upload to an existing name is stored as `name (1).ext`. |
| Rename is a name, not a path | The new name goes through the same name check, so it can't move a file out of its folder. |
| Share root can't be deleted | Even with *Full control*. |
| Integrity | Every transfer is SHA-256 verified. Data is written to `*.lsspart` and renamed only after the hash and size match; failures delete the partial file. |
| Bounded resources | Frames larger than 4 MiB are rejected before allocation; idle and stalled connections time out; speed-test size is capped. |

All of the above are covered by automated tests (`cargo test --lib`), which run against a real server on a loopback port.

### What is **not** protected

1. **No encryption.** Anyone who can capture packets on your network (shared Wi-Fi, a compromised device on the LAN) can read transfers. Only use it on networks you trust.
2. **No proof of identity.** Devices are recognised by a random ID they announce. A hostile device on the LAN that learns an approved device's ID could impersonate it. IDs are not secret on the wire.
3. **Discovery is unauthenticated.** Anyone can announce a fake device or name. The approval prompt shows the device name and IP address, but names are self-declared.
4. **Trust everyone mode** disables approval entirely. Settings warns before enabling it.
5. **Metadata is visible.** Device name, OS and the *names* of your enabled shared folders are broadcast on the LAN every 3 seconds.
6. **A denial of service from the LAN is possible** (many connections). Connections are cheap but not rate-limited.
7. **Unsigned installers.** Verify downloads with `SHA256SUMS.txt`.

### Planned

Encrypted transport with device pairing (mutual key pinning), which would fix items 1 and 2.

## Android

The app requests *All files access* so it can read files you pick and write to `Download/LocalSendStudio`. It uses a multicast lock to receive discovery packets. It makes no connections other than to devices on your local network.

## Reporting a vulnerability

Please open a **private** security advisory: <https://github.com/unupunct/LocalSendStudio/security/advisories/new>. Don't post exploit details in a public issue. Include the version, platform and steps to reproduce.
