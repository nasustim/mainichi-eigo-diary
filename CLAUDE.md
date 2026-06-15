# mainichi-eigo-diary

A web-based **English-learning diary** app. Users write diary entries in English and get
**LLM proofreading** to support their language learning.

## Product vision (future plan — issue #2)
- **LLM proofreading in the browser.** Proofreading must run client-side (no backend), so
  the plan is to use an in-browser LLM such as [WebLLM](https://github.com/mlc-ai/web-llm).
- **Backend-less, hosted on GitHub Pages.** Users can try the app with zero setup.
- **Portable data.** Diaries live in the browser; provide import/export so users own their data.

Roadmap issues are tracked on GitHub. Architype: issue #2. Check `gh issue list` for current work.

## Tech stack
- **Rust + [Yew](https://yew.rs)** (`0.23`, client-side rendering) compiled to WebAssembly.
- **[Trunk](https://trunkrs.dev)** (`0.21.14`) as the wasm bundler / dev server.
- Hosting: **GitHub Pages** via `.github/workflows/deploy.yml` (builds with
  `--public-url /mainichi-eigo-diary/` since the site serves from a project subpath).

## Versioning policy — MUST pin every dependency to an exact version
**This is a hard rule. Agents must comply.** Every dependency — Rust crates AND GitHub
Actions — is pinned to a single exact version. No ranges (`^`, `~`, `*`), no floating
major tags (`@v4`), no branch refs (`@master`, `@stable`). **Renovate** proposes upgrades;
do not loosen a pin to "fix" a build.

How to pin each kind:
- **Rust toolchain** — exact `channel` in `rust-toolchain.toml` (plus the
  `wasm32-unknown-unknown` target).
- **Rust crates** — exact `=x.y.z` requirement in `Cargo.toml` (e.g. `yew = "=0.23.0"`);
  `Cargo.lock` is committed.
- **GitHub Actions** — exact release tag `@vX.Y.Z` (e.g. `actions/checkout@v6.0.3`). If an
  action publishes no semver release (e.g. `dtolnay/rust-toolchain`, which only tags `v1`),
  pin to the full commit **SHA** with a trailing `# <tag>` comment instead.
- **Tools installed in CI** — exact version (e.g. `taiki-e/install-action` with
  `tool: trunk@0.21.14`).

When adding any dependency, look up its current exact version (`gh api`, crates.io) and pin it.

## Common commands
- `trunk serve --open` — run the dev server with hot reload.
- `trunk build --release` — produce the production bundle in `dist/`.
- `cargo test` — run native unit tests (the host-runnable, non-DOM logic).
- `cargo fmt` / `cargo clippy --all-targets -- -D warnings` — format and lint (TDD + lint required before done).

## Layout
- `src/main.rs` — app entry point and root `App` component.
- `src/index.html` — Trunk entry; its `rel="rust"` link points at `../Cargo.toml`
  (`Trunk.toml` sets `target = "src/index.html"`).
- `Trunk.toml` — build config. `.nasustim-documents/` — per-task TODO/plan notes.

## Notes for contributors
- GitHub Pages must be set to **Source: GitHub Actions** in repo Settings (one-time, manual).
- Tests run on the host target; keep browser/DOM-dependent tests out of `cargo test` (add
  `wasm-bindgen-test` later if real-DOM testing is needed).
