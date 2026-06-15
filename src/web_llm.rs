// Rust bindings for src/web_llm.js (bundled as a wasm-bindgen snippet by Trunk).
// These bindings are wasm32-only; all functions here call into JS.

#[cfg(target_arch = "wasm32")]
mod bindings {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "/src/web_llm.js")]
    extern "C" {
        /// Create an MLC engine for the given model.
        /// `progress_cb` is called with `(progress: f64, text: String)` during loading.
        #[wasm_bindgen(js_name = createEngine)]
        pub fn create_engine(model_id: &str, progress_cb: &js_sys::Function) -> js_sys::Promise;

        /// Proofread the given text using the loaded engine.
        /// Returns a Promise that resolves to the corrected text string.
        #[wasm_bindgen(js_name = proofread)]
        pub fn proofread_js(engine: &JsValue, text: &str) -> js_sys::Promise;

        /// Returns true if WebGPU is available in the current browser.
        #[wasm_bindgen(js_name = isWebGpuAvailable)]
        pub fn is_web_gpu_available() -> bool;
    }
}

#[cfg(target_arch = "wasm32")]
pub use bindings::{create_engine, is_web_gpu_available, proofread_js};
