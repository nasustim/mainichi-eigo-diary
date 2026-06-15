// ---------------------------------------------------------------------------
// Pure helpers (host-testable — no DOM / JS / WebGPU calls)
// ---------------------------------------------------------------------------

/// Build the system prompt used when asking the model to proofread a diary entry.
/// Exposed for unit testing.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub fn build_system_prompt() -> String {
    "You are an English writing assistant. \
     The user will give you a diary entry written in English. \
     Correct any grammar, spelling, or naturalness issues. \
     Return ONLY the corrected text with no explanation, \
     no markdown code fences, and no commentary."
        .to_string()
}

/// Strip leading/trailing whitespace and any markdown code fences that a model
/// might accidentally prepend/append (e.g. "```\n...\n```").
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub fn clean_model_output(raw: &str) -> String {
    let trimmed = raw.trim();

    // Strip a leading code fence line (``` or ```text, ```english, etc.)
    let after_open = if let Some(rest) = trimmed.strip_prefix("```") {
        // Skip to the end of the fence line
        if let Some(nl) = rest.find('\n') {
            rest[nl + 1..].trim_start()
        } else {
            // The entire string was just a fence opener with no newline — return empty.
            return String::new();
        }
    } else {
        trimmed
    };

    // Strip a trailing closing fence.
    let cleaned = if let Some(body) = after_open.strip_suffix("```") {
        body.trim_end()
    } else {
        after_open
    };

    cleaned.to_string()
}

// ---------------------------------------------------------------------------
// Yew component (wasm32-only — uses JS/DOM)
// ---------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
mod component {
    use js_sys::{Function, Reflect};
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;
    use yew::{component, html, use_mut_ref, use_state, Callback, Html, Properties};

    use crate::web_llm::{create_engine, is_web_gpu_available, proofread_js};

    use super::clean_model_output;

    /// Available model options for the selector.
    const MODELS: &[(&str, &str)] = &[
        (
            "SmolLM2-360M-Instruct-q4f16_1-MLC",
            "SmolLM2 360M (fast, recommended)",
        ),
        (
            "Llama-3.2-1B-Instruct-q4f32_1-MLC",
            "Llama 3.2 1B (balanced)",
        ),
        ("Qwen3-0.6B-q4f16_1-MLC", "Qwen3 0.6B (small)"),
        ("gemma3-1b-it-q4f16_1-MLC", "Gemma 3 1B (Google)"),
    ];

    #[derive(Properties, PartialEq)]
    pub struct ProofreadPanelProps {
        /// The diary text to proofread.
        pub text: String,
        /// Called with the corrected text when the user clicks "Apply".
        #[prop_or_default]
        pub on_apply: Option<Callback<String>>,
    }

    /// A self-contained panel that loads a WebLLM model in-browser and
    /// proofreads the supplied diary text.
    #[component]
    pub fn ProofreadPanel(props: &ProofreadPanelProps) -> Html {
        // --- Feature gate: WebGPU availability ---
        let webgpu_ok = is_web_gpu_available();

        // Selected model ID.
        let selected_model = use_state(|| MODELS[0].0.to_string());

        // Engine handle (None until loaded).
        let engine_ref = use_mut_ref(|| Option::<JsValue>::None);

        // Loading progress 0.0..=1.0.
        let load_progress: yew::UseStateHandle<f64> = use_state(|| 0.0_f64);
        // Human-readable loading status.
        let load_status: yew::UseStateHandle<String> = use_state(String::new);
        // Whether loading is in progress.
        let loading: yew::UseStateHandle<bool> = use_state(|| false);
        // Whether the engine has been loaded successfully.
        let engine_ready: yew::UseStateHandle<bool> = use_state(|| false);

        // Proofread result text.
        let result_text: yew::UseStateHandle<String> = use_state(String::new);
        // Whether a proofread request is in flight.
        let proofreading: yew::UseStateHandle<bool> = use_state(|| false);
        // Error message, if any.
        let error_msg: yew::UseStateHandle<String> = use_state(String::new);

        // --- "Load model" handler ---
        let on_load = {
            let selected_model = selected_model.clone();
            let load_progress = load_progress.clone();
            let load_status = load_status.clone();
            let loading = loading.clone();
            let engine_ready = engine_ready.clone();
            let engine_ref = engine_ref.clone();
            let error_msg = error_msg.clone();

            Callback::from(move |_: yew::events::MouseEvent| {
                let model_id = (*selected_model).clone();
                let load_progress = load_progress.clone();
                let load_status = load_status.clone();
                let loading = loading.clone();
                let engine_ready = engine_ready.clone();
                let engine_ref = engine_ref.clone();
                let error_msg = error_msg.clone();

                loading.set(true);
                engine_ready.set(false);
                error_msg.set(String::new());
                load_progress.set(0.0);
                load_status.set("Initialising…".to_string());

                wasm_bindgen_futures::spawn_local(async move {
                    // Build a JS closure for the progress callback.
                    let lp = load_progress.clone();
                    let ls = load_status.clone();
                    let progress_closure =
                        Closure::<dyn Fn(f64, String)>::new(move |progress: f64, text: String| {
                            lp.set(progress);
                            ls.set(text);
                        });
                    let progress_fn: &Function = progress_closure.as_ref().unchecked_ref();

                    let promise = create_engine(&model_id, progress_fn);
                    match JsFuture::from(promise).await {
                        Ok(engine_val) => {
                            *engine_ref.borrow_mut() = Some(engine_val);
                            engine_ready.set(true);
                            load_status.set("Model loaded — ready to proofread.".to_string());
                            load_progress.set(1.0);
                        }
                        Err(err) => {
                            let msg = format_js_error(&err);
                            error_msg.set(format!("Failed to load model: {msg}"));
                            load_status.set(String::new());
                        }
                    }
                    loading.set(false);
                    // Keep closure alive until the future resolves.
                    drop(progress_closure);
                });
            })
        };

        // --- "Proofread" handler ---
        let on_proofread = {
            let text = props.text.clone();
            let engine_ref = engine_ref.clone();
            let proofreading = proofreading.clone();
            let result_text = result_text.clone();
            let error_msg = error_msg.clone();

            Callback::from(move |_: yew::events::MouseEvent| {
                let engine_val = engine_ref.borrow().clone();
                if let Some(engine) = engine_val {
                    let text = text.clone();
                    let proofreading = proofreading.clone();
                    let result_text = result_text.clone();
                    let error_msg = error_msg.clone();

                    proofreading.set(true);
                    error_msg.set(String::new());
                    result_text.set(String::new());

                    wasm_bindgen_futures::spawn_local(async move {
                        let promise = proofread_js(&engine, &text);
                        match JsFuture::from(promise).await {
                            Ok(val) => {
                                let raw = val.as_string().unwrap_or_default();
                                let cleaned = clean_model_output(&raw);
                                result_text.set(cleaned);
                            }
                            Err(err) => {
                                let msg = format_js_error(&err);
                                error_msg.set(format!("Proofread error: {msg}"));
                            }
                        }
                        proofreading.set(false);
                    });
                }
            })
        };

        // --- "Apply" handler ---
        let on_apply = {
            let result_text = result_text.clone();
            let on_apply_cb = props.on_apply.clone();
            Callback::from(move |_: yew::events::MouseEvent| {
                if let Some(cb) = &on_apply_cb {
                    cb.emit((*result_text).clone());
                }
            })
        };

        // --- Model selector change ---
        let on_model_change = {
            let selected_model = selected_model.clone();
            let engine_ready = engine_ready.clone();
            let engine_ref = engine_ref.clone();
            let result_text = result_text.clone();
            let error_msg = error_msg.clone();
            let load_status = load_status.clone();
            let load_progress = load_progress.clone();
            Callback::from(move |e: yew::events::Event| {
                use wasm_bindgen::JsCast;
                if let Some(select) = e
                    .target()
                    .and_then(|t| t.dyn_into::<web_sys::HtmlSelectElement>().ok())
                {
                    selected_model.set(select.value());
                    // Reset engine state when model changes.
                    engine_ready.set(false);
                    *engine_ref.borrow_mut() = None;
                    result_text.set(String::new());
                    error_msg.set(String::new());
                    load_status.set(String::new());
                    load_progress.set(0.0);
                }
            })
        };

        // --- Render ---

        if !webgpu_ok {
            return html! {
                <aside class="proofread-panel proofread-panel--unavailable">
                    <h3>{ "AI Proofreading" }</h3>
                    <p class="proofread-notice">
                        { "Proofreading requires a WebGPU-capable browser (Chrome 113+ or Edge 113+). \
                           Your current browser does not support WebGPU. \
                           Please try again in a supported browser." }
                    </p>
                </aside>
            };
        }

        let progress_pct = (*load_progress * 100.0) as u32;

        html! {
            <aside class="proofread-panel">
                <h3>{ "AI Proofreading" }</h3>

                <div class="proofread-model-row">
                    <label for="proofread-model-select">{ "Model: " }</label>
                    <select
                        id="proofread-model-select"
                        onchange={on_model_change}
                        disabled={*loading}
                    >
                        { for MODELS.iter().map(|(id, label)| html! {
                            <option
                                value={*id}
                                selected={(*selected_model).as_str() == *id}
                            >
                                { *label }
                            </option>
                        }) }
                    </select>
                </div>

                <div class="proofread-load-row">
                    <button
                        class="btn-secondary"
                        onclick={on_load}
                        disabled={*loading}
                    >
                        { if *engine_ready { "Reload model" } else { "Load model" } }
                    </button>

                    if *loading {
                        <div class="proofread-progress">
                            <progress value={progress_pct.to_string()} max="100" />
                            <span class="proofread-status">{ (*load_status).clone() }</span>
                        </div>
                    } else if !(*load_status).is_empty() {
                        <span class="proofread-status">{ (*load_status).clone() }</span>
                    }
                </div>

                if !(*error_msg).is_empty() {
                    <p class="proofread-error">{ (*error_msg).clone() }</p>
                }

                if *engine_ready {
                    <div class="proofread-action-row">
                        <button
                            class="btn-primary"
                            onclick={on_proofread}
                            disabled={*proofreading || props.text.trim().is_empty()}
                        >
                            { if *proofreading { "Proofreading…" } else { "Proofread" } }
                        </button>
                    </div>
                }

                if !(*result_text).is_empty() {
                    <div class="proofread-result">
                        <h4>{ "Corrected text:" }</h4>
                        <pre class="proofread-result-text">{ (*result_text).clone() }</pre>
                        if props.on_apply.is_some() {
                            <button
                                class="btn-secondary"
                                onclick={on_apply}
                            >
                                { "Apply" }
                            </button>
                        }
                    </div>
                }
            </aside>
        }
    }

    /// Convert a JS error value to a human-readable string.
    fn format_js_error(err: &JsValue) -> String {
        // Try err.message first (standard Error objects).
        if let Ok(msg) = Reflect::get(err, &JsValue::from_str("message")) {
            if let Some(s) = msg.as_string() {
                return s;
            }
        }
        // Fall back to toString.
        err.as_string()
            .or_else(|| err.as_f64().map(|n| n.to_string()))
            .unwrap_or_else(|| "Unknown error".to_string())
    }
}

#[cfg(target_arch = "wasm32")]
pub use component::ProofreadPanel;

// ---------------------------------------------------------------------------
// Unit tests (host-runnable — pure logic only)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{build_system_prompt, clean_model_output};

    #[test]
    fn system_prompt_is_not_empty() {
        let p = build_system_prompt();
        assert!(!p.is_empty());
    }

    #[test]
    fn system_prompt_contains_key_instructions() {
        let p = build_system_prompt();
        assert!(p.contains("grammar"));
        assert!(p.contains("ONLY"));
        assert!(p.contains("code fences"));
    }

    #[test]
    fn clean_plain_text_is_unchanged() {
        let input = "Today I went to the park and enjoyed the weather.";
        assert_eq!(clean_model_output(input), input);
    }

    #[test]
    fn clean_trims_whitespace() {
        assert_eq!(clean_model_output("  hello world  \n"), "hello world");
    }

    #[test]
    fn clean_strips_plain_code_fence() {
        let input = "```\nToday I went to the park.\n```";
        assert_eq!(clean_model_output(input), "Today I went to the park.");
    }

    #[test]
    fn clean_strips_labelled_code_fence() {
        let input = "```text\nHello world.\n```";
        assert_eq!(clean_model_output(input), "Hello world.");
    }

    #[test]
    fn clean_strips_english_labelled_code_fence() {
        let input = "```english\nI had a great day.\n```";
        assert_eq!(clean_model_output(input), "I had a great day.");
    }

    #[test]
    fn clean_no_trailing_fence_leaves_text_intact() {
        // If only opening fence is present (model truncated output), still strips the opener.
        let input = "```\nSome text here";
        assert_eq!(clean_model_output(input), "Some text here");
    }

    #[test]
    fn clean_empty_fence_returns_empty() {
        assert_eq!(clean_model_output("```"), "");
    }

    #[test]
    fn clean_multiline_without_fences() {
        let input = "Line one.\nLine two.\nLine three.";
        assert_eq!(clean_model_output(input), input);
    }

    #[test]
    fn clean_multiline_with_fences() {
        let input = "```\nLine one.\nLine two.\n```";
        assert_eq!(clean_model_output(input), "Line one.\nLine two.");
    }
}
