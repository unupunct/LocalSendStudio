# Architecture

Tauri v2 app: a Rust core and a React/TypeScript UI in a system WebView. The same code builds for Windows, macOS, Linux and Android.

```
┌──────────────────────────── UI (src/) ────────────────────────────┐
│ React 19 · react-router (hash) · lucide icons · plain CSS          │
│ lib/store.tsx   devices, live transfers, approvals, toasts          │
│ lib/api.ts      typed wrapper over Tauri `invoke`                   │
└───────────────────────────────┬────────────────────────────────────┘
                 invoke / events │ (transfer-progress, approval-request, devices-updated …)
┌───────────────────────────────┴────────────────────────────────────┐
│ Core (src-tauri/src/)                                              │
│  lib.rs        setup, tray, close-to-tray, single instance          │
│  commands.rs   Tauri commands exposed to the UI                     │
│  server.rs     TCP server: authorize → dispatch → permission check  │
│  client.rs     TCP client: list, download (resume), upload, speed   │
│  transfer.rs   background jobs, progress events, cancel             │
│  auth.rs       approval flow (pending/allowed/denied peers)         │
│  discovery.rs  UDP announce + listen, manual devices                │
│  protocol.rs   framing, request/response types, size limits         │
│  util.rs       path safety, names, interfaces, hostnames            │
│  db.rs         SQLite: shares, history, trusted devices, settings   │
│  tests.rs      end-to-end tests (real server on loopback)           │
└────────────────────────────────────────────────────────────────────┘
```

## Request path (server side)

```
accept → read frame → parse Request
       → method == info ? answer
       → authorize(peer): trust_mode open? trusted? session-allowed? else ask the UI and wait ≤45 s
       → find share (enabled, by name) → check permission (read/upload/manage)
       → util::safe_join(share root, path)   ← traversal / symlink / Windows-name checks
       → perform operation
```

The decision is cached for the connection, so a transfer doesn't re-prompt.

## Design decisions

- **Own TCP protocol, not HTTP.** Simple framing, easy to stream with one hash pass and no extra dependency. Documented in [PROTOCOL.md](PROTOCOL.md).
- **Inbox share.** On first run an upload-only share pointing at the download folder is created, so "send a file" needs no setup on the receiver.
- **Approval, not pairing codes.** Fast for home use. The limits are spelled out in [SECURITY.md](SECURITY.md); encryption plus pairing is the planned replacement.
- **Hash in the end frame.** The server hashes while streaming, so there is no up-front pass over a large file. The client hashes while writing.
- **`.lsspart` temp files.** Nothing half-written is ever visible under its final name; the same file enables resume.
- **UI never builds paths for the server.** It sends a share name and a relative path; the server owns the mapping.
- **Android picker.** The system picker returns `content://` URIs that native code can't read, so Android uses an in-app browser over real paths with *All files access*. See `components/LocalPicker.tsx`.

## Tests

`cd src-tauri && cargo test --lib` runs 19 tests:

- frame limits and round trip;
- path rules (traversal, drive letters, streams, reserved names, symlinks on Unix);
- upload → download round trip with checksum, no-overwrite, resume, stale partial file;
- permissions for read / upload / manage, share-root protection;
- approval allow-and-remember, deny, and refusal with no UI;
- an oversized-frame attack leaving the server alive.

The UI can be previewed without the Rust side using demo data: `VITE_DEMO=1 npm run dev` (this is how the README screenshots are made; it is not part of release builds).
