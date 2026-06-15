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

## Tasks — MUST run through the Makefile
**This is a hard rule. Agents must comply.** Every repeatable task (build, dev, test,
format, lint, clean, CI steps) runs through a `Makefile` target — do not invoke the raw
`trunk`/`cargo` commands directly, and keep CI (`.github/workflows/`) calling `make`
targets too. When you add a new task, add a `Makefile` target for it instead of
documenting a bare command.

- `make dev` — dev server with hot reload (opens the browser).
- `make build` — production bundle into `dist/`. Override the served path with
  `make build PUBLIC_URL=/mainichi-eigo-diary/` (CI uses this for the Pages subpath).
- `make test` — native unit tests (host-runnable, non-DOM logic).
- `make fmt` / `make fmt-check` — format / verify formatting.
- `make lint` — Clippy with warnings as errors.
- `make check` — full pre-commit/CI suite (`fmt-check` + `lint` + `test`).
- `make clean` — remove `dist/` and `target/`.

TDD + lint are required before a task is considered done: run `make check`.

## Layout
- `src/main.rs` — entry point: `mod` declarations + `yew::Renderer::<app::App>`.
- `src/app.rs` — root `App` component; owns diary state via `use_reducer`, wires the
  editor, entry list, import/export controls, and the proofread panel.
- `src/model.rs` — `DiaryEntry` + `EntriesState`/`EntriesAction` reducer (pure, host-tested).
- `src/storage.rs` — `localStorage` persistence; pure `serialize`/`deserialize` are tested.
- `src/util.rs` — wasm-only `now_iso()` / `new_id()` (via `js_sys::Date`).
- `src/components/` — `editor.rs`, `entry_list.rs`.
- `src/portability.rs` (#5) — JSON import/export (`Blob` download + `FileReader` upload);
  `to_json`/`from_json` are pure and tested.
- `src/proofread.rs` + `src/web_llm.rs` + `src/web_llm.js` (#4) — in-browser LLM
  proofreading. The Rust layer owns the system prompt (`build_system_prompt`); `web_llm.js`
  is a wasm-bindgen snippet importing `@mlc-ai/web-llm@0.2.84` from the esm.run CDN.
- `src/index.html` — Trunk entry; `rel="rust"` → `../Cargo.toml`, `rel="css"` → `styles.css`
  (`Trunk.toml` sets `target = "src/index.html"`).
- `Trunk.toml` — build config. `.nasustim-documents/` — per-task TODO/plan notes.

## Notes for contributors
- GitHub Pages must be set to **Source: GitHub Actions** in repo Settings (one-time, manual).
- Tests run on the host target; keep browser/DOM-dependent tests out of `cargo test` (add
  `wasm-bindgen-test` later if real-DOM testing is needed). DOM/WebGPU/WebLLM code is kept
  behind thin wasm32-only wrappers; only pure helpers are unit-tested.
- **Proofreading needs a WebGPU browser** (Chrome/Edge 113+); the default model
  `SmolLM2-360M-Instruct-q4f16_1-MLC` also needs `shader-f16`. The panel degrades gracefully
  with a message when WebGPU is unavailable. The model downloads in-browser on first use.
