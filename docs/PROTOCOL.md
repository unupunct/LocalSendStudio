# Network protocol (version 2)

Version 2 is **not** compatible with 0.1.x clients (they lack the `from` field and checksums).

## Discovery

UDP port **47822**. Each device sends a JSON datagram every 3 seconds to

- multicast group `239.255.42.99` on each IPv4 interface, and
- the directed broadcast address of each IPv4 interface.

```json
{ "type": "announce", "id": "<uuid>", "name": "Maya's MacBook", "os": "macOS",
  "version": "0.2.0", "port": 47821, "shared_folders": ["Inbox", "Photos"] }
```

A device is *online* if heard in the last 12 s and forgotten after 3 min. If the file server's default port is taken, the next free port up to +19 is used and announced. Devices can also be added by IP; those are polled with `info` every 5 s.

## Framing

TCP, default port **47821**. Every frame is

```
[u32 big-endian: length of kind+payload][u8 kind][payload]
```

| kind | meaning |
|---|---|
| 1 | JSON |
| 2 | file chunk (up to 256 KiB) |
| 3 | end of stream (payload may carry a checksum) |

Payloads over **4 MiB** are rejected before allocation. Reads time out after 60 s (120 s idle between requests).

## Requests and responses

```json
{ "id": "<uuid>", "method": "list_dir", "params": { "share": "Photos", "path": "2025" },
  "from": { "id": "<device id>", "name": "Studio-PC" } }
```
```json
{ "id": "<uuid>", "ok": true, "data": { }, "error": null }
```

`from` is required for everything except `info`. If the peer isn't approved the server answers `{ "ok": false, "error": "not authorized" }` and closes. The first reply can take up to 45 s while the owner decides.

## Methods

| method | params | permission | notes |
|---|---|---|---|
| `info` | – | none | `{id,name,os,version,protocol,shared_folders}` |
| `list_dir` | `share`, `path` | read | `{entries:[{name,is_dir,size,modified}], can_upload, can_manage}`; folders first |
| `download` | `share`, `path`, `offset?` | read | see below |
| `upload` | `share`, `path`, `size`, `sha256` | upload | see below |
| `mkdir` | `share`, `path` | manage | |
| `delete` | `share`, `path` | manage | share root can't be deleted |
| `rename` | `share`, `path`, `new_name` | manage | `new_name` must be a plain name; fails if it exists |
| `speedtest` | `bytes?` (≤256 MiB) | approved peer | zero-filled chunks |

*read* is satisfied by `read` or `manage`; *upload* by `upload` or `manage`.

### download

1. Client → `download {share, path, offset}` (`offset` = bytes it already has in its `.lsspart` file, else 0).
2. Server → JSON `{size, offset}`.
3. Server → chunk frames for bytes `offset..size`.
4. Server → **end** frame whose payload is the lowercase hex **SHA-256 of the whole file** (the server hashes the skipped prefix too).
5. Client compares the digest with what it computed over prefix + received data. On mismatch with a resumed file it discards the partial file and retries once from 0.

The client writes to `<name>.lsspart` and renames only after size and digest match.

### upload

1. Client → `upload {share, path, size, sha256}`.
2. Server → JSON `{ready:true}` (or an error such as `uploading is not allowed on this folder`). The server picks a non-existing name (`name (1).ext`) and opens `<name>.lsspart`.
3. Client → chunk frames, then an **end** frame.
4. Server verifies byte count and SHA-256, renames the file, and replies `{path, size}` with the final relative path. On failure it deletes the partial file and replies with an error.

## Compatibility

`info.protocol` carries the protocol version (2). Unknown methods return `unknown method`.
