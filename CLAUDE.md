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
**This is a hard rule. Agents must comply.** Every dependency (Rust crates, GitHub Actions,
CI tools, JS CDN imports) is pinned to a single exact version — no ranges, no floating major
tags, no branch refs. Renovate proposes upgrades; do not loosen a pin to "fix" a build.
Full how-to-pin recipe: [`docs/03_versioning_policy.md`](docs/03_versioning_policy.md).

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
`src/main.rs` (entry), `src/index.html` (Trunk target; `rel="rust"` → `../Cargo.toml`,
`rel="css"` → `styles.css`), `Trunk.toml` (build config), `.nasustim-documents/` (local
per-task notes). Full module map: [`docs/01_layout.md`](docs/01_layout.md). The in-browser
proofreading subsystem: [`docs/02_proofread_system.md`](docs/02_proofread_system.md).

## Notes for contributors
- GitHub Pages must be set to **Source: GitHub Actions** in repo Settings (one-time, manual).
- Tests run on the host target; keep DOM/WebGPU/WebLLM code behind thin `wasm32`-only
  wrappers and unit-test only pure helpers (add `wasm-bindgen-test` later for real-DOM tests).
- **Proofreading needs a WebGPU browser** — see [`docs/02_proofread_system.md`](docs/02_proofread_system.md).
