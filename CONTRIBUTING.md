# Contributing

Thanks for helping. Bug reports with steps to reproduce are as valuable as code.

## Set up

```bash
npm install
npm run tauri dev
```

See [docs/BUILD.md](docs/BUILD.md) for platform prerequisites and [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for how it fits together.

## Before you open a PR

```bash
npx tsc --noEmit
cd src-tauri && cargo test --lib && cargo clippy --lib -- -D warnings
```

CI runs the same on Windows, macOS and Linux.

## Guidelines

- **Anything that touches paths, permissions, or the protocol needs a test** in `src-tauri/src/tests.rs` or `util.rs`. Those tests run a real server and a real client.
- Keep the UI usable at 360 px wide and in both themes. Use the CSS variables in `src/styles.css`, not fixed colors.
- Don't use `alert`, `prompt` or `confirm` (the app window doesn't support them): use `useApp().confirm/prompt/toast`.
- Be honest in docs and UI copy. If something isn't encrypted or isn't implemented, say so.
- Screenshots: use the demo mode (`VITE_DEMO=1 npm run dev`) so no real device names, IPs or files appear.

## Reporting security problems

Privately, please: see [docs/SECURITY.md](docs/SECURITY.md#reporting-a-vulnerability).
