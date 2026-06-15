# Proofreading system (#4)

Proofreading runs **entirely in the browser** — no backend. Diary text is corrected by an
LLM executed on WebGPU via [WebLLM](https://github.com/mlc-ai/web-llm). Nothing is sent to
a server; the model is downloaded into the browser cache on first use.

## Components
| File | Role |
|---|---|
| `src/web_llm.js` | ES module (a wasm-bindgen snippet). Imports `@mlc-ai/web-llm@0.2.84` from the `esm.run` CDN (exact-pinned). Exports `createEngine(modelId, progressCb)`, `proofread(engine, text, systemPrompt)`, and `isWebGpuAvailable()`. |
| `src/web_llm.rs` | `#[wasm_bindgen(module = "/src/web_llm.js")]` extern bindings (`wasm32`-only) returning `js_sys::Promise`. |
| `src/proofread.rs` | Pure helpers (`build_system_prompt`, `clean_model_output`) + the `ProofreadPanel` Yew component (`wasm32`-only). |

## Data flow
1. On mount, `ProofreadPanel` calls `isWebGpuAvailable()`. If false it renders a notice and
   disables all controls (graceful fallback).
2. **Load model**: `createEngine(modelId, progressCb)` → `CreateMLCEngine`. The progress
   callback drives a `<progress>` bar. The returned engine handle is held in a
   `use_mut_ref`.
3. **Proofread**: `proofread(engine, text, systemPrompt)` calls the OpenAI-style
   `engine.chat.completions.create({ messages: [system, user] })`. The result is passed
   through `clean_model_output` (trims whitespace and strips stray markdown code fences).
4. **Apply** (optional): emits the corrected text back via an `on_apply` callback.

Promises are driven from Rust with `wasm_bindgen_futures::JsFuture` inside `spawn_local`.
All JS errors are converted to human-readable strings — the panel never panics
(e.g. `ShaderF16SupportError`, device-lost, or WebGPU-unavailable surface as messages).

## System prompt — single source of truth
The system prompt is owned by **Rust** (`build_system_prompt` in `proofread.rs`) and passed
into `web_llm.js`'s `proofread(engine, text, systemPrompt)`. The JS layer does not hardcode
a prompt. This keeps the prompt unit-testable on the host and avoids drift between the two
layers. To change the proofreading behaviour, edit `build_system_prompt` (and its tests).

## Models
A `<select>` lists low-resource prebuilt models; the default is
`SmolLM2-360M-Instruct-q4f16_1-MLC` (~380 MB). Others offered: `Llama-3.2-1B-Instruct-q4f32_1-MLC`,
`Qwen3-0.6B-q4f16_1-MLC`, `gemma3-1b-it-q4f16_1-MLC`. To add a model, append to the `MODELS`
table in `proofread.rs` (id, label). Changing the selection resets the loaded engine.

## Requirements & caveats
- Needs a **WebGPU** browser (Chrome / Edge 113+). The default model also requires the
  `shader-f16` GPU feature.
- First load downloads the model weights (hundreds of MB) from the HuggingFace/MLC CDN and
  caches them in the browser.
- The WebLLM CDN import is resolved by the **browser at runtime**, so `make build` needs no
  network for it; the snippet is emitted under `dist/snippets/…/src/web_llm.js`.

## Testing
Only pure helpers are unit-tested on the host (`build_system_prompt`, `clean_model_output`).
The engine/DOM/WebGPU path cannot run in `cargo test`; verify it manually in a WebGPU
browser via `make dev` (load a model, then proofread). The WebGPU-unavailable fallback can
be checked in a non-WebGPU browser.
