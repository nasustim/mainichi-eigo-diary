# Source layout

The app is a single-page Rust + Yew (CSR) WebAssembly app bundled by Trunk. All UI logic
lives under `src/`. Pure, non-DOM logic is kept in functions that are unit-tested on the
host target; DOM / WebGPU / WebLLM code is kept behind thin `wasm32`-only wrappers.

## Modules (`src/`)
| File | Responsibility |
|---|---|
| `main.rs` | Entry point: `mod` declarations + `yew::Renderer::<app::App>::new().render()`. |
| `app.rs` | Root `App` component. Owns diary state via `use_reducer`, persists to storage on change, and wires the editor, entry list, import/export controls, and proofread panel. |
| `model.rs` | `DiaryEntry { id, created_at, updated_at, body }` + `EntriesState` / `EntriesAction` (`Add` / `Update` / `Delete` / `ReplaceAll`) reducer. **Pure, host-tested.** The reducer takes timestamps in via the action so it stays free of JS. |
| `storage.rs` | `localStorage` persistence (`gloo-storage`, key `diary-entries`). Pure `serialize` / `deserialize` (serde_json) are host-tested; `load` / `save` are the thin untested wrappers. |
| `util.rs` | `wasm32`-only helpers `now_iso()` / `new_id()` via `js_sys::Date`. |
| `components/editor.rs` | Textarea + Save button; emits the body via `Callback<String>`. |
| `components/entry_list.rs` | Renders entries with select / delete callbacks. |
| `portability.rs` (#5) | JSON import/export — see below. |
| `proofread.rs`, `web_llm.rs`, `web_llm.js` (#4) | In-browser LLM proofreading — see [`02_proofread_system.md`](02_proofread_system.md). |

## Import / export (#5)
`portability.rs` defines an `ExportFile { version, exported_at, entries }`.
- `to_json(entries, exported_at)` / `from_json(&str) -> Result<Vec<DiaryEntry>, String>` are
  pure and unit-tested (round-trip + malformed / wrong-version inputs).
- The `ImportExport` component: **Export** builds JSON and triggers a `Blob` download via a
  temporary `<a download>`; **Import** reads a file with `FileReader`, parses with
  `from_json`, confirms, then dispatches `EntriesAction::ReplaceAll`.

## State & integration seams
- The reducer handle lives in `App`; clone it before moving into callbacks.
  `entries.entries` is the `Vec<DiaryEntry>`.
- Feature components are built outside the `html!` macro (so `#[cfg]` applies to plain Rust
  expressions) and mounted at the "feature slots" marker in `app.rs`.

## Build entry points
- `src/index.html` — Trunk target (`Trunk.toml` sets `target = "src/index.html"`).
  `rel="rust"` → `../Cargo.toml`; `rel="css"` → `styles.css`.
- `src/styles.css` — the frontend styles. Built on CSS custom-property design tokens
  (`:root` color / spacing / radius / shadow / typography scales) with reusable `.card`
  and `.btn-*` classes; light theme, responsive. The diary body and result text use a
  serif face for a "diary" feel. `app.rs` renders a header (title + import/export toolbar),
  an editor card, the proofread panel section, and the past-entries list.

See the repo root `CLAUDE.md` for the hard rules (exact dependency pinning, Makefile-driven
tasks, TDD). Local per-task notes live in `.nasustim-documents/` (git-ignored).
