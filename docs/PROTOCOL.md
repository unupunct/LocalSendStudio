# Network Protocol

## Discovery

UDP port 47822. Multicast group 239.255.42.99 plus broadcast 255.255.255.255. Payload JSON announce:
`{ type, id, name, os, version, port, shared_folders }`.

## Transfer control

TCP port 47821 (default). Frames: `[u32 length][u8 kind][payload]`.
Kinds: 1 JSON, 2 chunk, 3 end.

Requests: `{ id, method, params }`. Responses: `{ id, ok, data, error }`.

Methods: info, list_dir, download, upload, mkdir, delete, rename.

Download flow: request -> metadata JSON (size, sha256) -> chunk frames -> end.
Upload flow: request -> ready JSON -> client sends chunk frames -> end.
