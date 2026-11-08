# Orbsmith

Desktop app that launches stand-in processes named after game executables, so that software detecting running games sees them as launched.

> [!IMPORTANT]
> Orbsmith is not affiliated with any software or game publisher.

## Tech stack

- [Tauri 2](https://tauri.app) (Rust backend, system WebView)
- [Svelte 5](https://svelte.dev) and [Vite](https://vite.dev)
- [Tailwind CSS 4](https://tailwindcss.com) and [shadcn-svelte](https://shadcn-svelte.com)
- [Oxlint](https://oxc.rs/docs/guide/usage/linter) and [Oxfmt](https://oxc.rs/docs/guide/usage/formatter)

## Development

### Prerequisites

- Node.js 24+ and pnpm
- Rust stable (1.90+)
- [Tauri system prerequisites](https://tauri.app/start/prerequisites/) for your platform

### Commands

```sh
pnpm install       # install dependencies
pnpm tauri dev     # run the app in development mode
pnpm tauri build   # build the installer
pnpm check         # type-check
pnpm lint          # lint the code
pnpm format        # format the code
pnpm format:check  # check formatting
```

## License

Released under the [MIT License](LICENSE).
