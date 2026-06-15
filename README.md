# mainichi-eigo-diary

A web-based **English-learning diary**. Write a diary entry in English and get
**LLM proofreading** that runs entirely in your browser — no backend, your data stays local.

## Features
- ✍️ **Diary editor** — write, edit, and delete entries; stored in the browser (`localStorage`).
- 🤖 **In-browser proofreading** (#4) — corrects grammar/spelling/naturalness via
  [WebLLM](https://github.com/mlc-ai/web-llm) running on WebGPU. The model downloads on
  first use; nothing is sent to a server.
- 💾 **Import / Export** (#5) — export all entries to a JSON file and re-import it, so you
  own and can back up your data.

## Tech stack
Rust + [Yew](https://yew.rs) 0.23 (CSR) → WebAssembly, bundled with
[Trunk](https://trunkrs.dev) 0.21.14, hosted on GitHub Pages.

## Development
All tasks run through the `Makefile`:

| Command | What it does |
|---|---|
| `make dev` | dev server with hot reload (opens the browser) |
| `make build` | production bundle into `dist/` |
| `make test` | host-runnable unit tests |
| `make check` | `fmt-check` + `lint` + `test` (the pre-commit/CI suite) |
| `make clean` | remove `dist/` and `target/` |

## Requirements
- **Proofreading needs a WebGPU-capable browser** (Chrome/Edge 113+). The default model
  (`SmolLM2-360M-Instruct-q4f16_1-MLC`) also requires `shader-f16`. Where WebGPU is
  unavailable the rest of the app still works and the proofread panel shows a notice.

See [CLAUDE.md](CLAUDE.md) for contributor rules (exact dependency pinning, Makefile-driven
tasks, TDD) and the product roadmap.
