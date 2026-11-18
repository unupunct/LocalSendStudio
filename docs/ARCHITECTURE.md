# Architecture

## Overview

LocalSendStudio is a Tauri v2 desktop application with a Rust networking core and a React/TypeScript frontend.

## Modules

- identity: device identity persistence
- db: SQLite storage (shared folders, history, groups, settings)
- discovery: UDP multicast + broadcast announcements
- protocol: length-prefixed framed TCP protocol
- server: TCP file server (browse, transfer, manage)
- client: TCP client helpers
- transfer: transfer orchestration and cancellation
- commands: Tauri command bridge
- util: helpers, interface enumeration

## Data flow

UI (React) -> Tauri commands -> Rust core -> TCP/UDP -> remote device.

Discovery runs in background; device list pushed via `devices-updated` event. Transfers emit `transfer-progress` and `transfer-done`.

## Storage

SQLite at OS data dir (`LocalSendStudio/localsend.db`). Identity at `identity.json`. Shared folders stored in DB.
