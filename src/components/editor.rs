use wasm_bindgen::JsCast;
use web_sys::HtmlTextAreaElement;
use yew::{component, html, use_state, Callback, Html, Properties};

/// Props for the `Editor` component.
#[derive(Properties, PartialEq)]
pub struct EditorProps {
    /// Called with the textarea body text when the user clicks "Save".
    pub on_save: Callback<String>,
    /// Optional initial body text (e.g. when editing an existing entry).
    #[prop_or_default]
    pub initial_body: String,
}

/// A textarea + "Save" button component.
/// Maintains its own local draft state; clears after save.
#[component]
pub fn Editor(props: &EditorProps) -> Html {
    let body = use_state(|| props.initial_body.clone());

    // When the parent changes `initial_body` (e.g. user selects a different entry),
    // sync the local state to the new value.
    // We do a simple approach: we track initial_body changes by re-rendering.
    // For simplicity, use a key-based reset at the call site (App sets key=entry_id).

    let oninput = {
        let body = body.clone();
        Callback::from(move |e: yew::events::InputEvent| {
            if let Some(target) = e.target() {
                if let Ok(textarea) = target.dyn_into::<HtmlTextAreaElement>() {
                    body.set(textarea.value());
                }
            }
        })
    };

    let on_save = props.on_save.clone();
    let onclick = {
        let body = body.clone();
        Callback::from(move |_: yew::events::MouseEvent| {
            let text = (*body).clone();
            if !text.trim().is_empty() {
                on_save.emit(text);
                body.set(String::new());
            }
        })
    };

    html! {
        <div class="editor">
            <textarea
                class="editor-textarea"
                placeholder="Write your diary entry in English…"
                value={(*body).clone()}
                oninput={oninput}
                rows="8"
            />
            <button class="btn-primary" onclick={onclick}>
                { "Save" }
            </button>
        </div>
    }
}
