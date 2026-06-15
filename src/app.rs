use yew::{component, html, use_effect_with, use_reducer, use_state, Callback, Html};

use crate::{
    components::{editor::Editor, entry_list::EntryList},
    model::{DiaryEntry, EntriesAction, EntriesState},
    storage,
    util::{new_id, now_iso},
};

#[cfg(target_arch = "wasm32")]
use crate::proofread::ProofreadPanel;

/// Root application component.
///
/// Integration seams for later features:
///   - Reducer dispatch handle: `dispatch` (type `yew::UseReducerDispatcher<EntriesState>`)
///   - Current entries accessor: `(*entries).entries.clone()` or `entries.entries.clone()`
///   - Feature slot placeholder: see comment below marked "=== feature slots ==="
#[component]
pub fn App() -> Html {
    // --- State ---

    // Primary diary state, initialized from localStorage.
    let entries = use_reducer(|| EntriesState::new(storage::load()));

    // Id of the entry currently loaded into the editor for editing (None = new entry mode).
    let editing_id: yew::UseStateHandle<Option<String>> = use_state(|| None);

    // Body pre-filled in the editor when the user clicks an existing entry.
    let prefill_body: yew::UseStateHandle<String> = use_state(String::new);

    // A key that forces the Editor to remount when we switch entries (clearing its draft).
    let editor_key: yew::UseStateHandle<u32> = use_state(|| 0u32);

    // --- Persistence side-effect ---

    // Whenever entries change, persist to localStorage.
    {
        let entries_snap = (*entries).clone();
        use_effect_with(entries_snap, move |state| {
            storage::save(&state.entries);
        });
    }

    // --- Callbacks ---

    let dispatch_add = {
        let entries = entries.clone();
        let editing_id = editing_id.clone();
        let prefill_body = prefill_body.clone();
        let editor_key = editor_key.clone();
        Callback::from(move |body: String| {
            let now = now_iso();
            let id = new_id();
            let entry = DiaryEntry::new(id, now, body);
            entries.dispatch(EntriesAction::Add(entry));
            editing_id.set(None);
            prefill_body.set(String::new());
            editor_key.set(*editor_key + 1);
        })
    };

    let dispatch_update = {
        let entries = entries.clone();
        let editing_id = editing_id.clone();
        let prefill_body = prefill_body.clone();
        let editor_key = editor_key.clone();
        Callback::from(move |body: String| {
            if let Some(id) = (*editing_id).clone() {
                let updated_at = now_iso();
                entries.dispatch(EntriesAction::Update {
                    id,
                    body,
                    updated_at,
                });
                editing_id.set(None);
                prefill_body.set(String::new());
                editor_key.set(*editor_key + 1);
            }
        })
    };

    let on_save: Callback<String> = {
        let editing_id = editing_id.clone();
        Callback::from(move |body: String| {
            if (*editing_id).is_some() {
                dispatch_update.emit(body);
            } else {
                dispatch_add.emit(body);
            }
        })
    };

    let on_select: Callback<String> = {
        let entries = entries.clone();
        let editing_id = editing_id.clone();
        let prefill_body = prefill_body.clone();
        let editor_key = editor_key.clone();
        Callback::from(move |id: String| {
            if let Some(entry) = entries.entries.iter().find(|e| e.id == id) {
                prefill_body.set(entry.body.clone());
                editing_id.set(Some(id));
                editor_key.set(*editor_key + 1);
            }
        })
    };

    let on_delete: Callback<String> = {
        let entries = entries.clone();
        let editing_id = editing_id.clone();
        let prefill_body = prefill_body.clone();
        let editor_key = editor_key.clone();
        Callback::from(move |id: String| {
            // If we're editing the entry being deleted, clear the editor too.
            if (*editing_id).as_deref() == Some(id.as_str()) {
                editing_id.set(None);
                prefill_body.set(String::new());
                editor_key.set(*editor_key + 1);
            }
            entries.dispatch(EntriesAction::Delete(id));
        })
    };

    let editor_label = if (*editing_id).is_some() {
        "Edit entry"
    } else {
        "New entry"
    };

    // === feature slots: import/export controls (#5) and proofread panel (#4) mount here ===
    // To wire in feature #5 (import/export):
    //   - Add an `<ImportExport entries={entries.entries.clone()} on_import={...} />`
    //     component here; its on_import callback should dispatch EntriesAction::ReplaceAll.

    // Feature #4: proofread panel — pass current editor draft or selected entry body.
    // Computed outside html! so #[cfg] attributes work on plain Rust expressions.
    #[cfg(target_arch = "wasm32")]
    let proofread_panel = {
        let proofread_text = (*prefill_body).clone();
        html! { <ProofreadPanel text={proofread_text} /> }
    };
    #[cfg(not(target_arch = "wasm32"))]
    let proofread_panel = html! {};

    html! {
        <div class="container">
            <header class="app-header">
                <h1>{ "Mainichi Eigo Diary" }</h1>
                <p class="app-subtitle">{ "Write your English diary — every day a little better." }</p>

                { proofread_panel }
            </header>

            <main class="app-main">
                <section class="editor-section">
                    <h2>{ editor_label }</h2>
                    <Editor
                        key={*editor_key}
                        on_save={on_save}
                        initial_body={(*prefill_body).clone()}
                    />
                </section>

                <section class="entries-section">
                    <h2>{ "Past entries" }</h2>
                    <EntryList
                        entries={entries.entries.clone()}
                        on_select={on_select}
                        on_delete={on_delete}
                    />
                </section>
            </main>
        </div>
    }
}
