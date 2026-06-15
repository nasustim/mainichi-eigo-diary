// ---------------------------------------------------------------------------
// Export file format — pure, host-testable.
//
// Gate with `any(test, target_arch = "wasm32")` so the host-target `clippy`
// invocation (which compiles for the native target, not wasm32) does not emit
// dead-code / unused-import warnings.  The functions are exercised by unit
// tests on the host and called at runtime on wasm32.
// ---------------------------------------------------------------------------

#[cfg(any(test, target_arch = "wasm32"))]
use crate::model::DiaryEntry;

#[cfg(any(test, target_arch = "wasm32"))]
const EXPORT_VERSION: u32 = 1;

/// The envelope written to / read from an export JSON file.
#[cfg(any(test, target_arch = "wasm32"))]
#[derive(serde::Serialize, serde::Deserialize)]
struct ExportFile {
    version: u32,
    exported_at: String,
    entries: Vec<DiaryEntry>,
}

/// Serialise entries into the export JSON envelope.
///
/// `exported_at` should be an ISO-8601 string (e.g. from `util::now_iso()`).
#[cfg(any(test, target_arch = "wasm32"))]
pub fn to_json(entries: &[DiaryEntry], exported_at: &str) -> String {
    let payload = ExportFile {
        version: EXPORT_VERSION,
        exported_at: exported_at.to_string(),
        entries: entries.to_vec(),
    };
    serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
}

/// Parse an export JSON envelope and return the contained entries.
///
/// Returns `Err(message)` if the JSON is malformed or the `version` field is
/// absent / does not match `EXPORT_VERSION`.
#[cfg(any(test, target_arch = "wasm32"))]
pub fn from_json(json: &str) -> Result<Vec<DiaryEntry>, String> {
    let payload: ExportFile =
        serde_json::from_str(json).map_err(|e| format!("Invalid JSON: {}", e))?;

    if payload.version != EXPORT_VERSION {
        return Err(format!(
            "Unsupported export version {} (expected {}). \
             Please export from the same app version.",
            payload.version, EXPORT_VERSION
        ));
    }

    Ok(payload.entries)
}

// ---------------------------------------------------------------------------
// Thin wasm-only wrapper — Blob + anchor download. NOT host-tested.
// ---------------------------------------------------------------------------

/// Trigger a file download in the browser.
///
/// Creates an in-memory `Blob` from `contents`, builds a temporary `<a
/// download>` element, clicks it programmatically, then revokes the object URL.
///
/// This is intentionally thin so all real logic stays in the pure functions
/// above and can be tested on the host target.
#[cfg(target_arch = "wasm32")]
pub fn download_json(filename: &str, contents: &str) {
    use wasm_bindgen::JsCast;

    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let document = match window.document() {
        Some(d) => d,
        None => return,
    };

    // Build the Blob.
    let parts = js_sys::Array::new();
    parts.push(&wasm_bindgen::JsValue::from_str(contents));

    let options = web_sys::BlobPropertyBag::new();
    options.set_type("application/json");

    let blob = match web_sys::Blob::new_with_str_sequence_and_options(&parts, &options) {
        Ok(b) => b,
        Err(_) => return,
    };

    // Create an object URL for the blob.
    let url = match web_sys::Url::create_object_url_with_blob(&blob) {
        Ok(u) => u,
        Err(_) => return,
    };

    // Build a temporary <a download> element.
    let anchor = match document.create_element("a") {
        Ok(el) => el,
        Err(_) => {
            let _ = web_sys::Url::revoke_object_url(&url);
            return;
        }
    };
    let anchor: web_sys::HtmlAnchorElement = match anchor.dyn_into() {
        Ok(a) => a,
        Err(_) => {
            let _ = web_sys::Url::revoke_object_url(&url);
            return;
        }
    };

    anchor.set_href(&url);
    anchor.set_download(filename);

    // The element must be in the DOM for Firefox; append, click, remove.
    let body = match document.body() {
        Some(b) => b,
        None => {
            let _ = web_sys::Url::revoke_object_url(&url);
            return;
        }
    };
    let _ = body.append_child(&anchor);
    anchor.click();
    let _ = body.remove_child(&anchor);

    let _ = web_sys::Url::revoke_object_url(&url);
}

// ---------------------------------------------------------------------------
// ImportExport Yew component — wasm32 only (uses FileReader / confirm).
// ---------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
mod component {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;
    use yew::{component, html, use_state, Callback, Html, Properties};

    use crate::model::DiaryEntry;
    use crate::util::now_iso;

    use super::{download_json, from_json, to_json};

    #[derive(Properties, PartialEq)]
    pub struct ImportExportProps {
        pub entries: Vec<DiaryEntry>,
        pub on_import: Callback<Vec<DiaryEntry>>,
    }

    /// A small toolbar component that provides Export and Import buttons.
    ///
    /// - **Export**: serialises current entries to a JSON file and triggers a
    ///   browser download.
    /// - **Import**: opens a file picker; on file selection, reads the JSON
    ///   with `FileReader`, asks the user to confirm (window.confirm), then
    ///   fires `on_import` with the parsed entries.
    #[component]
    pub fn ImportExport(props: &ImportExportProps) -> Html {
        // Status / error message shown below the buttons.
        let status: yew::UseStateHandle<Option<String>> = use_state(|| None);

        // ---- Export --------------------------------------------------------

        let on_export = {
            let entries = props.entries.clone();
            let status = status.clone();
            Callback::from(move |_: yew::events::MouseEvent| {
                let json = to_json(&entries, &now_iso());
                download_json("mainichi-eigo-diary-export.json", &json);
                status.set(Some(format!("Exported {} entries.", entries.len())));
            })
        };

        // ---- Import --------------------------------------------------------

        let on_import_cb = props.on_import.clone();
        let status_for_change = status.clone();

        let on_file_change = Callback::from(move |e: yew::events::Event| {
            use web_sys::{FileReader, HtmlInputElement, ProgressEvent};

            let input: HtmlInputElement = match e.target() {
                Some(t) => match t.dyn_into() {
                    Ok(i) => i,
                    Err(_) => return,
                },
                None => return,
            };

            let file_list = match input.files() {
                Some(fl) => fl,
                None => return,
            };
            let file = match file_list.get(0) {
                Some(f) => f,
                None => return,
            };

            let reader = match FileReader::new() {
                Ok(r) => r,
                Err(_) => return,
            };

            // Clone handles needed inside the closure.
            let on_import_inner = on_import_cb.clone();
            let status_inner = status_for_change.clone();

            let onload = {
                let reader_clone = reader.clone();
                Closure::once(Box::new(move |_evt: ProgressEvent| {
                    let result = reader_clone.result();
                    let text = match result {
                        Ok(v) => match v.as_string() {
                            Some(s) => s,
                            None => {
                                status_inner.set(Some("Failed to read file as text.".to_string()));
                                return;
                            }
                        },
                        Err(_) => {
                            status_inner.set(Some("FileReader error.".to_string()));
                            return;
                        }
                    };

                    match from_json(&text) {
                        Err(msg) => {
                            status_inner.set(Some(format!("Import error: {}", msg)));
                        }
                        Ok(entries) => {
                            // Ask for confirmation before replacing all entries.
                            let confirmed = web_sys::window()
                                .and_then(|w| {
                                    w.confirm_with_message(&format!(
                                        "This will replace all {} current entries with {} \
                                         entries from the file. Continue?",
                                        "your",
                                        entries.len()
                                    ))
                                    .ok()
                                })
                                .unwrap_or(false);

                            if confirmed {
                                let count = entries.len();
                                on_import_inner.emit(entries);
                                status_inner
                                    .set(Some(format!("Imported {} entries successfully.", count)));
                            } else {
                                status_inner.set(Some("Import cancelled.".to_string()));
                            }
                        }
                    }
                }) as Box<dyn FnOnce(ProgressEvent)>)
            };

            reader.set_onload(Some(onload.as_ref().unchecked_ref()));
            // `onload` must not be dropped until after the async FileReader fires.
            // We intentionally forget it here — it is a one-shot closure and will
            // be cleaned up by the browser after it fires.
            onload.forget();

            let _ = reader.read_as_text(&file);
        });

        // Status message element.
        let status_el = (*status).as_ref().map(|msg| {
            html! { <span class="import-export-status">{ msg }</span> }
        });

        html! {
            <div class="import-export-controls">
                <button class="btn-secondary" onclick={on_export}>
                    { "Export" }
                </button>
                <label class="btn-secondary import-label">
                    { "Import" }
                    <input
                        type="file"
                        accept="application/json,.json"
                        style="display:none"
                        onchange={on_file_change}
                    />
                </label>
                { for status_el }
            </div>
        }
    }
}

// Re-export the component publicly on wasm32.
#[cfg(target_arch = "wasm32")]
pub use component::ImportExport;

// ---------------------------------------------------------------------------
// Unit tests — pure, host-runnable.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{from_json, to_json, EXPORT_VERSION};
    use crate::model::DiaryEntry;

    fn sample_entries() -> Vec<DiaryEntry> {
        vec![
            DiaryEntry::new("id-1", "2026-01-01T00:00:00.000Z", "Hello world"),
            DiaryEntry::new("id-2", "2026-01-02T00:00:00.000Z", "Second entry"),
        ]
    }

    // ---- round-trip --------------------------------------------------------

    #[test]
    fn round_trip_preserves_all_entries() {
        let entries = sample_entries();
        let json = to_json(&entries, "2026-06-15T00:00:00.000Z");
        let restored = from_json(&json).expect("round-trip should succeed");
        assert_eq!(restored.len(), 2);
        assert_eq!(restored[0].id, "id-1");
        assert_eq!(restored[0].body, "Hello world");
        assert_eq!(restored[0].created_at, "2026-01-01T00:00:00.000Z");
        assert_eq!(restored[1].id, "id-2");
    }

    #[test]
    fn round_trip_empty_entries() {
        let json = to_json(&[], "2026-06-15T00:00:00.000Z");
        let restored = from_json(&json).expect("empty round-trip should succeed");
        assert!(restored.is_empty());
    }

    // ---- valid parse -------------------------------------------------------

    #[test]
    fn valid_json_with_correct_version_succeeds() {
        let json = serde_json::json!({
            "version": EXPORT_VERSION,
            "exported_at": "2026-06-15T00:00:00.000Z",
            "entries": [
                {
                    "id": "x",
                    "created_at": "2026-01-01T00:00:00.000Z",
                    "updated_at": "2026-01-01T00:00:00.000Z",
                    "body": "test"
                }
            ]
        })
        .to_string();

        let entries = from_json(&json).expect("valid JSON should parse");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "x");
    }

    // ---- malformed JSON ----------------------------------------------------

    #[test]
    fn malformed_json_returns_err() {
        let result = from_json("not valid json {{{{");
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("Invalid JSON"), "got: {}", msg);
    }

    #[test]
    fn empty_string_returns_err() {
        let result = from_json("");
        assert!(result.is_err());
    }

    #[test]
    fn empty_object_returns_err() {
        let result = from_json("{}");
        assert!(result.is_err());
    }

    // ---- wrong version -----------------------------------------------------

    #[test]
    fn wrong_version_returns_err_with_helpful_message() {
        let json = serde_json::json!({
            "version": 99,
            "exported_at": "2026-06-15T00:00:00.000Z",
            "entries": []
        })
        .to_string();

        let result = from_json(&json);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        // Should mention both the found version and the expected version.
        assert!(msg.contains("99"), "got: {}", msg);
        assert!(msg.contains(&EXPORT_VERSION.to_string()), "got: {}", msg);
    }

    #[test]
    fn version_zero_returns_err() {
        let json = serde_json::json!({
            "version": 0,
            "exported_at": "2026-06-15T00:00:00.000Z",
            "entries": []
        })
        .to_string();

        let result = from_json(&json);
        assert!(result.is_err());
    }

    // ---- garbage / missing fields ------------------------------------------

    #[test]
    fn missing_version_field_returns_err() {
        let json = serde_json::json!({
            "exported_at": "2026-06-15T00:00:00.000Z",
            "entries": []
        })
        .to_string();

        let result = from_json(&json);
        assert!(result.is_err());
    }

    #[test]
    fn missing_entries_field_returns_err() {
        let json = serde_json::json!({
            "version": EXPORT_VERSION,
            "exported_at": "2026-06-15T00:00:00.000Z"
        })
        .to_string();

        let result = from_json(&json);
        assert!(result.is_err());
    }

    #[test]
    fn garbage_entries_array_returns_err() {
        let json = serde_json::json!({
            "version": EXPORT_VERSION,
            "exported_at": "2026-06-15T00:00:00.000Z",
            "entries": ["not", "diary", "objects"]
        })
        .to_string();

        let result = from_json(&json);
        assert!(result.is_err());
    }
}
