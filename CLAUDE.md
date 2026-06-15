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

## Versioning policy
Pin toolchain and library versions wherever possible for reproducible builds; let
**Renovate** propose upgrades. Specifics:
- Rust toolchain pinned in `rust-toolchain.toml` (channel + `wasm32-unknown-unknown` target).
- Crates pinned with exact `=` requirements in `Cargo.toml`; `Cargo.lock` is committed.
- Trunk version pinned in the deploy workflow; GitHub Actions pinned by tag.

## Common commands
- `trunk serve --open` — run the dev server with hot reload.
- `trunk build --release` — produce the production bundle in `dist/`.
- `cargo test` — run native unit tests (the host-runnable, non-DOM logic).
- `cargo fmt` / `cargo clippy --all-targets -- -D warnings` — format and lint (TDD + lint required before done).

## Layout
- `src/main.rs` — app entry point and root `App` component.
- `index.html` — Trunk entry (`<link data-trunk rel="rust" />`).
- `Trunk.toml` — build config. `.nasustim-documents/` — per-task TODO/plan notes.

## Notes for contributors
- GitHub Pages must be set to **Source: GitHub Actions** in repo Settings (one-time, manual).
- Tests run on the host target; keep browser/DOM-dependent tests out of `cargo test` (add
  `wasm-bindgen-test` later if real-DOM testing is needed).
